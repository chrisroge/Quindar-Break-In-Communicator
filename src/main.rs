use axum::{
    Router,
    extract::{Json, State},
    response::IntoResponse,
    routing::post,
};
use mpris::{PlayerFinder, PlaybackStatus};
use msedge_tts::tts::{SpeechConfig, client::connect_async};
use rodio::{Decoder, OutputStream, Sink, Source};
use serde::Deserialize;
use std::f32::consts::PI;
use std::io::Cursor;
use std::sync::{Arc, LazyLock};
use std::time::Duration;
use tokio::sync::mpsc;

/// Brand-term pronunciation respellings, applied to text before synthesis.
/// Neural TTS handles English orthography well but mangles novel proper nouns
/// (e.g. "Enscrive" gets pronounced "en-SKREEV"); a respelling like "en skryve"
/// nudges it onto the right phonetics. Logs preserve the original; only the
/// synthesizer sees the respelling.
static PRONUNCIATION_LEXICON: LazyLock<Vec<(&'static str, &'static str)>> = LazyLock::new(|| {
    vec![
        // "Enscrive" rhymes with "jive", begins with "N" not "ON".
        ("Enscrive", "en skryve"),
    ]
});

fn apply_pronunciation(text: &str) -> String {
    let mut result = text.to_string();
    for (term, respelling) in PRONUNCIATION_LEXICON.iter() {
        result = result.replace(term, respelling);
    }
    result
}

/// Backend-agnostic synthesis input.
#[derive(Debug, Clone)]
struct SynthesisRequest {
    text: String,
    voice: String,
    speed: f32,
}

/// TTS backend abstraction. Adding a new backend (Piper, Klatt, Parler, ...)
/// means adding a struct, a variant, and a `synthesize` arm.
#[derive(Clone)]
enum TtsBackend {
    Edge(EdgeBackend),
}

impl TtsBackend {
    fn from_env() -> Result<Self, String> {
        Ok(Self::Edge(EdgeBackend::from_env()))
    }

    fn display_name(&self) -> &'static str {
        match self {
            Self::Edge(_) => "Edge TTS (free)",
        }
    }

    async fn synthesize(&self, req: &SynthesisRequest) -> Result<Vec<u8>, String> {
        match self {
            Self::Edge(b) => b.synthesize(req).await,
        }
    }
}

#[derive(Clone)]
struct EdgeBackend {
    default_voice: String,
}

impl EdgeBackend {
    fn from_env() -> Self {
        Self {
            default_voice: std::env::var("EDGE_VOICE")
                .unwrap_or_else(|_| "en-US-AndrewNeural".to_string()),
        }
    }
}

/// Audio destination. Selected once at startup based on environment.
/// Telegram is outbound-only by design — no command handler, no inbound
/// listening — eliminating reply-injection attack surface.
#[derive(Clone)]
enum AudioSink {
    Local,
    Telegram(TelegramSink),
}

impl AudioSink {
    fn from_env() -> Self {
        match (
            std::env::var("TELEGRAM_BOT_TOKEN"),
            std::env::var("TELEGRAM_CHAT_ID"),
        ) {
            (Ok(token), Ok(chat_id)) if !token.is_empty() && !chat_id.is_empty() => {
                Self::Telegram(TelegramSink { token, chat_id })
            }
            _ => Self::Local,
        }
    }

    fn display_name(&self) -> &'static str {
        match self {
            Self::Local => "Local audio playback",
            Self::Telegram(_) => "Telegram (outbound voice message)",
        }
    }

    async fn deliver(
        &self,
        audio_bytes: Vec<u8>,
        transcript: &str,
        tone_type: ToneType,
        volume: f32,
    ) -> Result<(), String> {
        match self {
            Self::Local => {
                tokio::task::spawn_blocking(move || {
                    play_tones_and_audio(audio_bytes, volume, tone_type)
                })
                .await
                .map_err(|e| format!("Audio playback task failed: {}", e))?
            }
            Self::Telegram(s) => s.deliver(audio_bytes, transcript).await,
        }
    }
}

/// Telegram outbound sink. Currently delivers via `sendAudio` (MP3) — Edge
/// TTS's free read-aloud endpoint doesn't emit OGG/Opus, which `sendVoice`
/// would require. The transcript becomes the audio caption.
#[derive(Clone)]
struct TelegramSink {
    token: String,
    chat_id: String,
}

impl TelegramSink {
    async fn deliver(&self, audio_bytes: Vec<u8>, transcript: &str) -> Result<(), String> {
        let url = format!("https://api.telegram.org/bot{}/sendAudio", self.token);

        let audio_part = reqwest::multipart::Part::bytes(audio_bytes)
            .file_name("quindar.mp3")
            .mime_str("audio/mpeg")
            .map_err(|e| format!("Failed to build audio part: {}", e))?;

        let form = reqwest::multipart::Form::new()
            .text("chat_id", self.chat_id.clone())
            .text("caption", transcript.to_string())
            .text("title", "Quindar")
            .text("performer", "Claude Code")
            .part("audio", audio_part);

        let response = reqwest::Client::new()
            .post(&url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| format!("Failed to call Telegram API: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Telegram API error {}: {}", status, body));
        }

        println!("Telegram audio message delivered");
        Ok(())
    }
}

/// Tone Type options
#[derive(Debug, Clone, PartialEq)]
enum ToneType {
    Quindar,   // Classic NASA Quindar tones (default)
    None,      // No tones, just voice
    ThreeNote, // Three-note audience recall chime
}

impl ToneType {
    fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "NO-TONE" | "NONE" => ToneType::None,
            "THREE-NOTE" | "THREE-NOTE-CHIME" | "CHIME" => ToneType::ThreeNote,
            _ => ToneType::Quindar, // Default to Quindar
        }
    }

    fn from_env() -> Self {
        match std::env::var("DEFAULT_TONE").as_deref() {
            Ok(tone_str) => Self::from_str(tone_str),
            _ => ToneType::Quindar, // Default to Quindar
        }
    }
}


#[derive(Deserialize)]
struct PlayRequest {
    text: String,
    #[serde(default = "default_voice")]
    voice: String,
    #[serde(default = "default_speed")]
    speed: f32,
    #[serde(default = "default_volume")]
    volume: f32,
    #[serde(default)]
    tone: Option<String>,
}

fn default_voice() -> String {
    "alloy".to_string()
}

fn default_speed() -> f32 {
    1.0
}

fn default_volume() -> f32 {
    2.0 // Default to 2x volume to match Quindar tone loudness
}

#[derive(Clone)]
struct TransmissionRequest {
    text: String,
    voice: String,
    speed: f32,
    volume: f32,
    tone_type: ToneType,
}

#[derive(Clone)]
struct AppState {
    tx: mpsc::UnboundedSender<TransmissionRequest>,
}

/// Generate Quindar tone samples
fn generate_quindar_tone_samples(duration_ms: u32) -> Vec<f32> {
    let sample_rate = 48000;
    let frequency = 2500.0; // Hz - higher frequency is more audible
    let attack_ms = 50; // 50ms fade in
    let release_ms = 50; // 50ms fade out

    let total_samples = sample_rate * duration_ms / 1000;
    let attack_samples = sample_rate * attack_ms / 1000;
    let release_samples = sample_rate * release_ms / 1000;

    (0..total_samples)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            let sine_wave = (t * frequency * 2.0 * PI).sin();

            // Apply envelope to avoid clicks
            let envelope = if i < attack_samples {
                // Fade in
                i as f32 / attack_samples as f32
            } else if i > total_samples - release_samples {
                // Fade out
                (total_samples - i) as f32 / release_samples as f32
            } else {
                // Full volume
                1.0
            };

            sine_wave * envelope * 0.5 // 0.5 amplitude
        })
        .collect()
}

/// Generate three-note audience recall chime (like theater/concert hall chimes)
/// Simple ascending C-E-G pattern, xylophone-like with echo and depth
fn generate_three_note_chime() -> Vec<f32> {
    let sample_rate = 48000;
    let note_duration_ms = 800; // Longer notes for depth (800ms)
    let note_overlap_ms = 300; // Notes overlap while previous dissipates

    // Frequencies for C5-E5-G5 (middle register, clear and pleasant)
    let frequencies = [523.25, 659.25, 783.99]; // C5, E5, G5

    let note_samples = sample_rate * note_duration_ms / 1000;
    let overlap_samples = sample_rate * note_overlap_ms / 1000;
    let attack_samples = sample_rate * 10 / 1000; // 10ms soft attack

    // Total length: first note + (spacing * 2 notes) + decay tail
    let note_spacing = note_samples - overlap_samples;
    let total_length = note_samples + (note_spacing * 2) + (sample_rate / 2); // Extra 500ms decay
    let mut result = vec![0.0; total_length];

    for (idx, frequency) in frequencies.iter().enumerate() {
        let start_position = idx * note_spacing;

        // Generate one note
        for i in 0..note_samples {
            let t = i as f32 / sample_rate as f32;

            // Primary tone
            let sine_wave = (t * frequency * 2.0 * PI).sin();

            // Add harmonics for depth (octave and fifth)
            let harmonic_1 = (t * frequency * 2.0 * 2.0 * PI).sin() * 0.15; // Octave
            let harmonic_2 = (t * frequency * 1.5 * 2.0 * PI).sin() * 0.1; // Fifth

            let combined = sine_wave + harmonic_1 + harmonic_2;

            // Softer attack, longer exponential decay
            let envelope = if i < attack_samples {
                // Very soft attack
                (i as f32 / attack_samples as f32) * 0.5
            } else {
                // Slow exponential decay for depth
                let decay_t = (i - attack_samples) as f32 / note_samples as f32;
                (-decay_t * 1.8).exp() * 0.5
            };

            // Add echo/reverb (simple delay-based echo)
            let with_envelope = combined * envelope;
            let position = start_position + i;

            if position < result.len() {
                result[position] += with_envelope * 0.25; // Main signal (softer)
            }

            // First echo at 120ms
            let echo1_pos = position + (sample_rate * 120 / 1000);
            if echo1_pos < result.len() {
                result[echo1_pos] += with_envelope * 0.12; // Softer echo
            }

            // Second echo at 240ms
            let echo2_pos = position + (sample_rate * 240 / 1000);
            if echo2_pos < result.len() {
                result[echo2_pos] += with_envelope * 0.06; // Even softer echo
            }
        }
    }

    result
}

/// Check if running in headless mode (no audio output)
fn is_headless_mode() -> bool {
    std::env::var("HEADLESS_MODE")
        .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
        .unwrap_or(false)
}

/// Check if audio ducking is enabled via environment variable
fn is_audio_ducking_enabled() -> bool {
    std::env::var("ENABLE_AUDIO_DUCKING")
        .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
        .unwrap_or(false)
}

/// Manages pausing and resuming MPRIS media players for audio ducking
struct AudioDucker {
    paused_players: Vec<String>, // Bus names of players we paused
}

impl AudioDucker {
    /// Create a new AudioDucker and pause any playing media players
    fn pause_media_players() -> Self {
        let mut paused_players = Vec::new();

        if !is_audio_ducking_enabled() {
            return AudioDucker { paused_players };
        }

        println!("Audio ducking: Checking for active media players...");

        // Find all MPRIS players
        let finder = match PlayerFinder::new() {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Audio ducking: Failed to connect to D-Bus: {}", e);
                return AudioDucker { paused_players };
            }
        };

        let players = match finder.find_all() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Audio ducking: Failed to find players: {}", e);
                return AudioDucker { paused_players };
            }
        };

        for player in players {
            let identity = player.identity().to_string();

            // Check if player is currently playing
            match player.get_playback_status() {
                Ok(PlaybackStatus::Playing) => {
                    // Try to pause this player
                    match player.pause() {
                        Ok(_) => {
                            println!("Audio ducking: Paused '{}'", identity);
                            paused_players.push(player.bus_name().to_string());
                        }
                        Err(e) => {
                            eprintln!("Audio ducking: Failed to pause '{}': {}", identity, e);
                        }
                    }
                }
                Ok(_) => {
                    // Player not playing, skip
                }
                Err(e) => {
                    eprintln!(
                        "Audio ducking: Failed to get status for '{}': {}",
                        identity, e
                    );
                }
            }
        }

        if paused_players.is_empty() {
            println!("Audio ducking: No active media players found");
        }

        AudioDucker { paused_players }
    }

    /// Resume all players that were paused
    fn resume_media_players(self) {
        if self.paused_players.is_empty() {
            return;
        }

        println!(
            "Audio ducking: Resuming {} paused player(s)...",
            self.paused_players.len()
        );

        let finder = match PlayerFinder::new() {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Audio ducking: Failed to reconnect to D-Bus: {}", e);
                return;
            }
        };

        let players = match finder.find_all() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Audio ducking: Failed to find players for resume: {}", e);
                return;
            }
        };

        for player in players {
            if self.paused_players.contains(&player.bus_name().to_string()) {
                let identity = player.identity().to_string();
                match player.play() {
                    Ok(_) => {
                        println!("Audio ducking: Resumed '{}'", identity);
                    }
                    Err(e) => {
                        eprintln!("Audio ducking: Failed to resume '{}': {}", identity, e);
                    }
                }
            }
        }
    }
}

/// Play tones and audio based on tone type
fn play_tones_and_audio(
    audio_bytes: Vec<u8>,
    volume: f32,
    tone_type: ToneType,
) -> Result<(), String> {
    // Check for headless mode (WSL, headless servers, testing)
    if is_headless_mode() {
        println!("Headless mode: Skipping audio playback (TTS generated successfully)");
        return Ok(());
    }

    let (_stream, stream_handle) = OutputStream::try_default()
        .map_err(|e| format!("Failed to create output stream: {}", e))?;
    let sink =
        Sink::try_new(&stream_handle).map_err(|e| format!("Failed to create sink: {}", e))?;

    let sample_rate = 48000;

    match tone_type {
        ToneType::Quindar => {
            println!("Playing opening Quindar tone...");

            // Opening Quindar tone (500ms)
            let opening_tone_samples = generate_quindar_tone_samples(500);
            let opening_tone_source = AudioSource {
                samples: opening_tone_samples,
                sample_rate,
                current: 0,
            };
            sink.append(opening_tone_source);
        }
        ToneType::ThreeNote => {
            println!("Playing three-note audience recall chime...");

            // Three-note chime
            let chime_samples = generate_three_note_chime();
            let chime_source = AudioSource {
                samples: chime_samples,
                sample_rate,
                current: 0,
            };
            sink.append(chime_source);
        }
        ToneType::None => {
            println!("No tone - playing voice only...");
            // No opening tone, just play the voice
        }
    }

    println!("Playing voice transmission (volume: {:.1}x)...", volume);

    // TTS audio with volume boost
    let cursor = Cursor::new(audio_bytes);
    let source = Decoder::new(cursor).map_err(|e| format!("Failed to decode audio: {}", e))?;

    // Apply volume gain
    let amplified_source = source.amplify(volume);
    sink.append(amplified_source);

    // Closing tone (only for Quindar and ThreeNote)
    match tone_type {
        ToneType::Quindar => {
            println!("Playing closing Quindar tone...");

            // Closing Quindar tone (shorter - 250ms)
            let closing_tone_samples = generate_quindar_tone_samples(250);
            let closing_tone_source = AudioSource {
                samples: closing_tone_samples,
                sample_rate,
                current: 0,
            };
            sink.append(closing_tone_source);
        }
        ToneType::ThreeNote => {
            println!("Playing closing chime...");

            // Shorter chime for closing (single note, like a bell)
            let closing_freq = 783.99; // G5 - final note of the chime
            let closing_duration_ms = 300;
            let closing_samples = sample_rate * closing_duration_ms / 1000;

            let closing_chime: Vec<f32> = (0..closing_samples)
                .map(|i| {
                    let t = i as f32 / sample_rate as f32;
                    let sine_wave = (t * closing_freq * 2.0 * PI).sin();
                    let decay_t = i as f32 / closing_samples as f32;
                    let envelope = (-decay_t * 2.5).exp();
                    sine_wave * envelope * 0.35
                })
                .collect();

            let closing_source = AudioSource {
                samples: closing_chime,
                sample_rate,
                current: 0,
            };
            sink.append(closing_source);
        }
        ToneType::None => {
            // No closing tone
        }
    }

    sink.sleep_until_end();

    Ok(())
}

/// Custom audio source for samples
struct AudioSource {
    samples: Vec<f32>,
    sample_rate: u32,
    current: usize,
}

impl Iterator for AudioSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.samples.len() {
            let sample = self.samples[self.current];
            self.current += 1;
            Some(sample)
        } else {
            None
        }
    }
}

impl Source for AudioSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

impl EdgeBackend {
    async fn synthesize(&self, req: &SynthesisRequest) -> Result<Vec<u8>, String> {
        // Map speed to edge-tts rate format (percentage)
        // speed 0.25 -> -75%, speed 1.0 -> +0%, speed 4.0 -> +300%
        let rate_percent = ((req.speed - 1.0) * 100.0).round() as i32;
        let rate_str = if rate_percent >= 0 {
            format!("+{}%", rate_percent)
        } else {
            format!("{}%", rate_percent)
        };

        // Legacy voice names from the old OpenAI backend route to the configured
        // Edge default; any other value is treated as a literal Edge voice id.
        let final_voice = if matches!(
            req.voice.as_str(),
            "alloy" | "echo" | "fable" | "onyx" | "nova" | "shimmer"
        ) {
            self.default_voice.clone()
        } else {
            req.voice.clone()
        };

        println!(
            "Calling Edge TTS API with voice: {}, rate: {}",
            final_voice, rate_str
        );

        // Retry logic for Edge TTS - first request after startup sometimes fails with WebSocket error
        let max_retries = 3;
        let mut last_error = String::new();

        for attempt in 1..=max_retries {
            let client_result = connect_async().await;
            let mut client = match client_result {
                Ok(c) => c,
                Err(e) => {
                    last_error = format!("Failed to connect to Edge TTS: {}", e);
                    if attempt < max_retries {
                        println!(
                            "Connection attempt {} failed, retrying... ({})",
                            attempt, last_error
                        );
                        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
                        continue;
                    }
                    return Err(last_error);
                }
            };

            let config = SpeechConfig {
                voice_name: final_voice.clone(),
                audio_format: "audio-24khz-48kbitrate-mono-mp3".to_string(),
                pitch: 0,
                rate: rate_percent,
                volume: 0,
            };

            match client.synthesize(&req.text, &config).await {
                Ok(audio_result) => {
                    if attempt > 1 {
                        println!("✓ Connection succeeded on attempt {}", attempt);
                    }
                    return Ok(audio_result.audio_bytes);
                }
                Err(e) => {
                    last_error = format!("Failed to synthesize speech: {}", e);
                    if attempt < max_retries {
                        println!(
                            "Synthesis attempt {} failed, retrying... ({})",
                            attempt, last_error
                        );
                        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
                        continue;
                    }
                }
            }
        }

        Err(last_error)
    }
}

/// Process a single transmission (called by queue processor)
async fn process_transmission(
    req: TransmissionRequest,
    backend: Arc<TtsBackend>,
    sink: Arc<AudioSink>,
) {
    println!(
        "\n=== Processing transmission: {} (voice: {}) ===",
        req.text, req.voice
    );

    let mut log_msg = format!(
        "Requesting TTS from {} with voice '{}' → {}",
        backend.display_name(),
        req.voice,
        sink.display_name()
    );
    if req.speed != 1.0 {
        log_msg.push_str(&format!(", speed: {}", req.speed));
    }
    if req.volume != 2.0 {
        log_msg.push_str(&format!(", volume: {:.1}x", req.volume));
    }
    println!("{}...", log_msg);

    let synth_req = SynthesisRequest {
        text: apply_pronunciation(&req.text),
        voice: req.voice.clone(),
        speed: req.speed,
    };

    let backend_for_task = Arc::clone(&backend);
    let tts_task = tokio::spawn(async move { backend_for_task.synthesize(&synth_req).await });

    // Wait for TTS to complete buffering (no pre-transmission audio)
    let audio_bytes = match tts_task.await {
        Ok(Ok(bytes)) => {
            println!("Voice buffered successfully ({} bytes)", bytes.len());
            if bytes.is_empty() {
                eprintln!("Error: TTS returned 0 bytes — the requested audio format may be unsupported by Edge TTS");
                return;
            }
            bytes
        }
        Ok(Err(e)) => {
            eprintln!("Error getting TTS: {}", e);
            return;
        }
        Err(e) => {
            eprintln!("Task error: {}", e);
            return;
        }
    };

    // Audio ducking only matters for local playback. Telegram delivery doesn't
    // touch the local audio device, so we skip the D-Bus dance entirely.
    let needs_local_playback = matches!(*sink, AudioSink::Local);
    let ducker = if needs_local_playback {
        Some(
            tokio::task::spawn_blocking(AudioDucker::pause_media_players)
                .await
                .unwrap_or_else(|e| {
                    eprintln!("Audio ducking task failed: {}", e);
                    AudioDucker {
                        paused_players: Vec::new(),
                    }
                }),
        )
    } else {
        None
    };

    if let Err(e) = sink
        .deliver(audio_bytes, &req.text, req.tone_type.clone(), req.volume)
        .await
    {
        eprintln!("Error delivering audio: {}", e);
    }

    if let Some(ducker) = ducker {
        tokio::task::spawn_blocking(move || {
            ducker.resume_media_players();
        })
        .await
        .ok();
    }

    println!("Transmission complete!\n");
}

/// Background task that processes the transmission queue
async fn transmission_queue_processor(
    mut rx: mpsc::UnboundedReceiver<TransmissionRequest>,
    backend: Arc<TtsBackend>,
    sink: Arc<AudioSink>,
) {
    println!("Transmission queue processor started");

    while let Some(req) = rx.recv().await {
        process_transmission(req, Arc::clone(&backend), Arc::clone(&sink)).await;
    }

    println!("Transmission queue processor stopped");
}

/// Load .env file from executable directory or current directory
#[allow(clippy::collapsible_if)]
fn load_env_file() {
    use std::path::PathBuf;

    let mut loaded = false;
    let mut env_path: Option<PathBuf> = None;

    // Try 1: Load from executable's directory
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let exe_env = exe_dir.join(".env");
            if exe_env.exists() {
                match dotenv::from_path(&exe_env) {
                    Ok(_) => {
                        loaded = true;
                        env_path = Some(exe_env);
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: Found .env at {:?} but failed to load: {}",
                            exe_env, e
                        );
                    }
                }
            }
        }
    }

    // Try 2: Load from current directory (if not already loaded)
    if !loaded {
        match dotenv::dotenv() {
            Ok(path) => {
                env_path = Some(path);
            }
            Err(_) => {
                // No .env file found in current directory either
            }
        }
    }

    // Log the result
    if let Some(path) = env_path {
        println!("Loaded configuration from: {}", path.display());
    } else {
        println!("No .env file found - using default configuration and environment variables");
    }
}

/// API handler to enqueue transmission requests
async fn play_tone_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PlayRequest>,
) -> impl IntoResponse {
    let mut log_msg = format!(
        "Received request, adding to queue: {} (voice: {})",
        payload.text, payload.voice
    );
    if payload.speed != 1.0 {
        log_msg.push_str(&format!(", speed: {}", payload.speed));
    }
    if payload.volume != 2.0 {
        log_msg.push_str(&format!(", volume: {:.1}x", payload.volume));
    }
    println!("{}", log_msg);

    // Determine tone type (from request or environment default)
    let tone_type = match &payload.tone {
        Some(tone_str) => ToneType::from_str(tone_str),
        None => ToneType::from_env(),
    };

    let transmission = TransmissionRequest {
        text: payload.text,
        voice: payload.voice,
        speed: payload.speed,
        volume: payload.volume,
        tone_type,
    };

    if let Err(e) = state.tx.send(transmission) {
        eprintln!("Failed to enqueue transmission: {}", e);
        return "Error: Failed to queue transmission".to_string();
    }

    "Transmission queued successfully!".to_string()
}

#[tokio::main]
async fn main() {
    // Load .env file - try executable directory first, then current directory
    load_env_file();

    // Construct the TTS backend once at startup so misconfiguration fails fast
    // instead of waiting for the first request.
    let backend = match TtsBackend::from_env() {
        Ok(b) => Arc::new(b),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    // Choose audio destination: Telegram if both bot token and chat id are
    // configured, otherwise local playback.
    let sink = Arc::new(AudioSink::from_env());

    // Create the transmission queue channel
    let (tx, rx) = mpsc::unbounded_channel::<TransmissionRequest>();

    // Spawn the queue processor task
    tokio::spawn(transmission_queue_processor(
        rx,
        Arc::clone(&backend),
        Arc::clone(&sink),
    ));

    // Create app state with the sender
    let state = Arc::new(AppState { tx });

    // Build the router with a POST endpoint and shared state
    let app = Router::new()
        .route("/play", post(play_tone_handler))
        .with_state(state);

    // Get bind address from environment or use default
    let bind_address =
        std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1:42069".to_string());

    let listener = tokio::net::TcpListener::bind(&bind_address)
        .await
        .unwrap_or_else(|e| {
            eprintln!("Failed to bind to {}: {}", bind_address, e);
            eprintln!("Make sure the address is valid and the port is not already in use.");
            std::process::exit(1);
        });

    println!("Quindar Tone API server running on http://{}", bind_address);
    println!("TTS Provider: {}", backend.display_name());
    println!("Audio Destination: {}", sink.display_name());

    if is_headless_mode() {
        println!("Audio Output: HEADLESS MODE (no audio playback, TTS generation only)");
        println!("  → Perfect for WSL, headless servers, and testing environments");
    } else {
        println!("Audio Output: ENABLED");
    }

    if is_audio_ducking_enabled() {
        println!("Audio Ducking: ENABLED (will pause media players during announcements)");
    } else {
        println!("Audio Ducking: DISABLED (set ENABLE_AUDIO_DUCKING=true to enable)");
    }

    println!("Transmission queue enabled - multiple requests will play sequentially");
    println!("Send a POST request with JSON body: {{\"text\": \"your message\"}}");

    // Show example curl command with current bind address
    let example_url = if bind_address.starts_with("0.0.0.0") {
        "http://127.0.0.1:42069".to_string()
    } else {
        format!("http://{}", bind_address)
    };
    println!(
        "Example: curl -X POST {}/play -H 'Content-Type: application/json' -d '{{\"text\": \"Test message\"}}'",
        example_url
    );

    axum::serve(listener, app).await.unwrap();
}
