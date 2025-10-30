use axum::{
    Router,
    extract::{Json, State},
    response::IntoResponse,
    routing::post,
};
use msedge_tts::tts::{SpeechConfig, client::connect_async};
use notify_rust::Notification;
use rodio::{Decoder, OutputStream, Sink, Source};
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use std::io::Cursor;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

/// TTS Provider options
#[derive(Debug, Clone, PartialEq)]
enum TtsProvider {
    Edge,
    OpenAI,
}

impl TtsProvider {
    fn from_env() -> Self {
        match std::env::var("DEFAULT_TTS").as_deref() {
            Ok("OPENAI") => TtsProvider::OpenAI,
            Ok("EDGE") => TtsProvider::Edge,
            _ => TtsProvider::Edge, // Default to Edge
        }
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

/// Toast notification urgency levels
#[derive(Debug, Clone, PartialEq)]
enum ToastUrgency {
    Info,     // General information - blue icon
    Warning,  // Warning message - yellow/orange icon
    Critical, // Critical/urgent - red icon
}

impl ToastUrgency {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "warning" | "warn" => ToastUrgency::Warning,
            "critical" | "error" | "urgent" => ToastUrgency::Critical,
            _ => ToastUrgency::Info, // Default to Info
        }
    }

    fn icon(&self) -> &str {
        match self {
            ToastUrgency::Info => "dialog-information",
            ToastUrgency::Warning => "dialog-warning",
            ToastUrgency::Critical => "dialog-error",
        }
    }

    fn timeout_ms(&self) -> i32 {
        match self {
            ToastUrgency::Info => 5000,       // 5 seconds
            ToastUrgency::Warning => 8000,    // 8 seconds
            ToastUrgency::Critical => 0,      // Persistent (requires dismissal)
        }
    }
}

#[derive(Deserialize)]
struct PlayRequest {
    text: String,
    #[serde(default = "default_voice")]
    voice: String,
    #[serde(default)]
    instructions: Option<String>,
    #[serde(default = "default_speed")]
    speed: f32,
    #[serde(default = "default_volume")]
    volume: f32,
    #[serde(default)]
    tone: Option<String>,
    #[serde(default)]
    enable_toast: Option<bool>,
    #[serde(default)]
    toast_urgency: Option<String>,
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
    instructions: Option<String>,
    speed: f32,
    volume: f32,
    tone_type: ToneType,
    enable_toast: bool,
    toast_urgency: ToastUrgency,
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

/// Stream TTS from OpenAI and return audio bytes
async fn get_openai_tts(
    text: &str,
    voice: &str,
    instructions: Option<&str>,
    speed: f32,
    api_key: &str,
) -> Result<Vec<u8>, String> {
    let client = reqwest::Client::new();

    #[derive(Serialize)]
    struct TTSRequest {
        model: String,
        input: String,
        voice: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        instructions: Option<String>,
        speed: f32,
    }

    let request_body = TTSRequest {
        model: "tts-1".to_string(),
        input: text.to_string(),
        voice: voice.to_string(),
        instructions: instructions.map(|s| s.to_string()),
        speed,
    };

    let response = client
        .post("https://api.openai.com/v1/audio/speech")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Failed to call OpenAI API: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("OpenAI API error {}: {}", status, error_text));
    }

    let audio_bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read audio stream: {}", e))?;

    Ok(audio_bytes.to_vec())
}

/// Get TTS from Edge TTS using native Rust client
async fn get_edge_tts(text: &str, voice: &str, speed: f32) -> Result<Vec<u8>, String> {
    // Map speed to edge-tts rate format (percentage)
    // speed 0.25 -> -75%, speed 1.0 -> +0%, speed 4.0 -> +300%
    let rate_percent = ((speed - 1.0) * 100.0).round() as i32;
    let rate_str = if rate_percent >= 0 {
        format!("+{}%", rate_percent)
    } else {
        format!("{}%", rate_percent)
    };

    // Get edge voice (use env var or default)
    let edge_voice =
        std::env::var("EDGE_VOICE").unwrap_or_else(|_| "en-US-AndrewNeural".to_string());

    // Use edge voice if no specific voice mapping needed
    // In the future, we could map OpenAI voice names to Edge voices
    let final_voice = if voice == "alloy"
        || voice == "echo"
        || voice == "fable"
        || voice == "onyx"
        || voice == "nova"
        || voice == "shimmer"
    {
        // Use default edge voice for OpenAI voice names
        edge_voice
    } else {
        // Use provided voice name (might be an edge-tts voice)
        voice.to_string()
    };

    println!(
        "Calling Edge TTS API with voice: {}, rate: {}",
        final_voice, rate_str
    );

    // Retry logic for Edge TTS - first request after startup sometimes fails with WebSocket error
    let max_retries = 3;
    let mut last_error = String::new();

    for attempt in 1..=max_retries {
        // Create TTS client
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

        // Create speech config with voice and rate
        // SpeechConfig fields: voice_name, audio_format, pitch, rate, volume (all i32 except strings)
        let config = SpeechConfig {
            voice_name: final_voice.clone(),
            audio_format: "audio-24khz-48kbitrate-mono-mp3".to_string(),
            pitch: 0,           // Normal pitch
            rate: rate_percent, // Already calculated as i32 percentage
            volume: 0,          // Normal volume
        };

        // Synthesize speech
        match client.synthesize(text, &config).await {
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

/// Show a toast notification with the given text and urgency level
fn show_toast_notification(text: &str, urgency: &ToastUrgency) {
    let result = Notification::new()
        .appname("Quindar Service")
        .summary("Quindar Notification")
        .body(text)
        .icon(urgency.icon())
        .timeout(urgency.timeout_ms())
        .show();

    match result {
        Ok(_) => println!("Toast notification shown: {}", text),
        Err(e) => eprintln!("Failed to show toast notification: {}", e),
    }
}

/// Process a single transmission (called by queue processor)
async fn process_transmission(req: TransmissionRequest) {
    println!(
        "\n=== Processing transmission: {} (voice: {}) ===",
        req.text, req.voice
    );

    // Show toast notification if enabled
    if req.enable_toast {
        show_toast_notification(&req.text, &req.toast_urgency);
    }

    // Determine TTS provider
    let tts_provider = TtsProvider::from_env();

    // Validate API key if using OpenAI
    if tts_provider == TtsProvider::OpenAI && std::env::var("OPENAI_API_KEY").is_err() {
        eprintln!("Error: OPENAI_API_KEY not set but DEFAULT_TTS=OPENAI");
        return;
    }

    // Start requesting TTS immediately (async)
    let mut log_msg = match tts_provider {
        TtsProvider::OpenAI => format!("Requesting TTS from OpenAI with voice '{}'", req.voice),
        TtsProvider::Edge => format!("Requesting TTS from Edge TTS with voice '{}'", req.voice),
    };

    if req.speed != 1.0 {
        log_msg.push_str(&format!(", speed: {}", req.speed));
    }
    if req.volume != 2.0 {
        log_msg.push_str(&format!(", volume: {:.1}x", req.volume));
    }
    if let Some(ref instr) = req.instructions {
        log_msg.push_str(&format!(", instructions: '{}'", instr));
    }
    println!("{}...", log_msg);

    let text = req.text.clone();
    let voice = req.voice.clone();
    let instructions = req.instructions.clone();
    let speed = req.speed;

    let tts_task = tokio::spawn(async move {
        match tts_provider {
            TtsProvider::OpenAI => {
                let api_key = std::env::var("OPENAI_API_KEY").unwrap();
                get_openai_tts(&text, &voice, instructions.as_deref(), speed, &api_key).await
            }
            TtsProvider::Edge => {
                // Edge TTS doesn't support instructions parameter
                if instructions.is_some() {
                    println!(
                        "Note: Edge TTS does not support instructions parameter (OpenAI only)"
                    );
                }
                get_edge_tts(&text, &voice, speed).await
            }
        }
    });

    // Wait for TTS to complete buffering (no pre-transmission audio)
    let audio_bytes = match tts_task.await {
        Ok(Ok(bytes)) => {
            println!("Voice buffered successfully!");
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

    // Now play tones and audio based on tone type
    let volume = req.volume;
    let tone_type = req.tone_type.clone();
    if let Err(e) = tokio::task::spawn_blocking(move || {
        if let Err(e) = play_tones_and_audio(audio_bytes, volume, tone_type) {
            eprintln!("Error playing audio: {}", e);
        }
    })
    .await
    {
        eprintln!("Audio playback task failed: {}", e);
    }

    println!("Transmission complete!\n");
}

/// Background task that processes the transmission queue
async fn transmission_queue_processor(mut rx: mpsc::UnboundedReceiver<TransmissionRequest>) {
    println!("Transmission queue processor started");

    while let Some(req) = rx.recv().await {
        process_transmission(req).await;
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
    if let Some(ref instr) = payload.instructions {
        log_msg.push_str(&format!(", instructions: '{}'", instr));
    }
    println!("{}", log_msg);

    // Determine tone type (from request or environment default)
    let tone_type = match &payload.tone {
        Some(tone_str) => ToneType::from_str(tone_str),
        None => ToneType::from_env(),
    };

    // Determine if toast notifications should be enabled
    // Priority: per-request > environment variable > false (default)
    let enable_toast = match payload.enable_toast {
        Some(val) => val,
        None => std::env::var("ENABLE_TOAST_NOTIFICATIONS")
            .unwrap_or_else(|_| "false".to_string())
            .to_lowercase() == "true",
    };

    // Determine toast urgency level
    let toast_urgency = match &payload.toast_urgency {
        Some(urgency_str) => ToastUrgency::from_str(urgency_str),
        None => ToastUrgency::Info, // Default to Info
    };

    let transmission = TransmissionRequest {
        text: payload.text,
        voice: payload.voice,
        instructions: payload.instructions,
        speed: payload.speed,
        volume: payload.volume,
        tone_type,
        enable_toast,
        toast_urgency,
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

    // Create the transmission queue channel
    let (tx, rx) = mpsc::unbounded_channel::<TransmissionRequest>();

    // Spawn the queue processor task
    tokio::spawn(transmission_queue_processor(rx));

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

    // Show TTS provider information
    let tts_provider = TtsProvider::from_env();
    let tts_name = match tts_provider {
        TtsProvider::Edge => "Edge TTS (free)",
        TtsProvider::OpenAI => "OpenAI (premium)",
    };

    println!("Quindar Tone API server running on http://{}", bind_address);
    println!("TTS Provider: {}", tts_name);

    if is_headless_mode() {
        println!("Audio Output: HEADLESS MODE (no audio playback, TTS generation only)");
        println!("  → Perfect for WSL, headless servers, and testing environments");
    } else {
        println!("Audio Output: ENABLED");
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
