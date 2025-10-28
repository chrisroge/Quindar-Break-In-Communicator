use axum::{extract::{Json, State}, response::IntoResponse, routing::post, Router};
use rand::Rng;
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
}

fn default_voice() -> String {
    "alloy".to_string()
}

fn default_speed() -> f32 {
    1.0
}

fn default_volume() -> f32 {
    2.0  // Default to 2x volume to match Quindar tone loudness
}

#[derive(Clone)]
struct TransmissionRequest {
    text: String,
    voice: String,
    instructions: Option<String>,
    speed: f32,
    volume: f32,
}

#[derive(Clone)]
struct AppState {
    tx: mpsc::UnboundedSender<TransmissionRequest>,
}

/// Generate a mic pop sound (short impulse)
fn generate_mic_pop(sample_rate: u32) -> Vec<f32> {
    let duration_ms = 30; // Very short pop
    let total_samples = sample_rate * duration_ms / 1000;
    let mut rng = rand::thread_rng();

    (0..total_samples)
        .map(|i| {
            // Create a quick burst with exponential decay
            let t = i as f32 / total_samples as f32;
            let decay = (-t * 8.0).exp(); // Fast exponential decay
            let noise: f32 = rng.gen_range(-1.0..1.0);

            // Mix a low frequency bump with noise for that "pop" sound
            let bump = (t * 100.0 * 2.0 * PI).sin() * decay;

            (bump * 0.15 + noise * decay * 0.05).clamp(-0.2, 0.2)
        })
        .collect()
}

/// Generate radio static/crackle
fn generate_static(duration_ms: u32, sample_rate: u32) -> Vec<f32> {
    let total_samples = sample_rate * duration_ms / 1000;
    let mut rng = rand::thread_rng();

    (0..total_samples)
        .map(|i| {
            // Generate white noise
            let noise: f32 = rng.gen_range(-1.0..1.0);

            // Apply fade in/out envelope for smooth transitions
            let fade_samples = sample_rate * 20 / 1000; // 20ms fade
            let envelope = if i < fade_samples {
                i as f32 / fade_samples as f32
            } else if i > total_samples - fade_samples {
                (total_samples - i) as f32 / fade_samples as f32
            } else {
                1.0
            };

            // Low amplitude for subtle static
            noise * envelope * 0.08
        })
        .collect()
}

/// Play mic pop and static
fn play_mic_pop_and_static(duration_ms: u32) {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    let sample_rate = 48000;

    // Mic pop first
    let mic_pop_samples = generate_mic_pop(sample_rate);
    let mic_pop_source = AudioSource {
        samples: mic_pop_samples,
        sample_rate,
        current: 0,
    };
    sink.append(mic_pop_source);

    // Then static
    let static_samples = generate_static(duration_ms, sample_rate);
    let static_source = AudioSource {
        samples: static_samples,
        sample_rate,
        current: 0,
    };
    sink.append(static_source);
    sink.sleep_until_end();
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

/// Play Quindar tone followed by post-static, then TTS audio, then closing Quindar
fn play_quindar_and_audio(audio_bytes: Vec<u8>, volume: f32) -> Result<(), String> {
    let (_stream, stream_handle) = OutputStream::try_default()
        .map_err(|e| format!("Failed to create output stream: {}", e))?;
    let sink = Sink::try_new(&stream_handle)
        .map_err(|e| format!("Failed to create sink: {}", e))?;

    let sample_rate = 48000;

    println!("Playing opening Quindar tone...");

    // Opening Quindar tone (500ms)
    let opening_tone_samples = generate_quindar_tone_samples(500);
    let opening_tone_source = AudioSource {
        samples: opening_tone_samples,
        sample_rate,
        current: 0,
    };
    sink.append(opening_tone_source);

    // Post-transmission static (200ms)
    let post_static = generate_static(200, sample_rate);
    let post_static_source = AudioSource {
        samples: post_static,
        sample_rate,
        current: 0,
    };
    sink.append(post_static_source);

    println!("Playing voice transmission (volume: {:.1}x)...", volume);

    // TTS audio with volume boost
    let cursor = Cursor::new(audio_bytes);
    let source = Decoder::new(cursor)
        .map_err(|e| format!("Failed to decode audio: {}", e))?;

    // Apply volume gain
    let amplified_source = source.amplify(volume);
    sink.append(amplified_source);

    println!("Playing closing Quindar tone...");

    // Closing Quindar tone (shorter - 250ms)
    let closing_tone_samples = generate_quindar_tone_samples(250);
    let closing_tone_source = AudioSource {
        samples: closing_tone_samples,
        sample_rate,
        current: 0,
    };
    sink.append(closing_tone_source);

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

/// Get TTS from Edge TTS (free, local)
async fn get_edge_tts(
    text: &str,
    voice: &str,
    speed: f32,
) -> Result<Vec<u8>, String> {
    // Map speed to edge-tts rate format (percentage)
    // speed 0.25 -> -75%, speed 1.0 -> +0%, speed 4.0 -> +300%
    let rate_percent = ((speed - 1.0) * 100.0).round() as i32;
    let rate_str = if rate_percent >= 0 {
        format!("+{}%", rate_percent)
    } else {
        format!("{}%", rate_percent)
    };

    // Get edge voice (use env var or default)
    let edge_voice = std::env::var("EDGE_VOICE")
        .unwrap_or_else(|_| "en-US-AndrewNeural".to_string());

    // Use edge voice if no specific voice mapping needed
    // In the future, we could map OpenAI voice names to Edge voices
    let final_voice = if voice == "alloy" || voice == "echo" || voice == "fable"
                         || voice == "onyx" || voice == "nova" || voice == "shimmer" {
        // Use default edge voice for OpenAI voice names
        edge_voice
    } else {
        // Use provided voice name (might be an edge-tts voice)
        voice.to_string()
    };

    // Create a temporary file for output
    let temp_file = format!("/tmp/edge_tts_{}.mp3", uuid::Uuid::new_v4());

    println!("Calling edge-tts with voice: {}, rate: {}", final_voice, rate_str);

    // Call edge-tts command
    let output = tokio::process::Command::new("edge-tts")
        .arg("--text")
        .arg(text)
        .arg("--voice")
        .arg(&final_voice)
        .arg("--rate")
        .arg(&rate_str)
        .arg("--write-media")
        .arg(&temp_file)
        .output()
        .await
        .map_err(|e| format!("Failed to execute edge-tts: {}. Make sure edge-tts is installed (pip install edge-tts)", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("edge-tts failed: {}", stderr));
    }

    // Read the generated audio file
    let audio_bytes = tokio::fs::read(&temp_file)
        .await
        .map_err(|e| format!("Failed to read generated audio: {}", e))?;

    // Clean up temp file
    let _ = tokio::fs::remove_file(&temp_file).await;

    Ok(audio_bytes)
}

/// Process a single transmission (called by queue processor)
async fn process_transmission(req: TransmissionRequest) {
    println!("\n=== Processing transmission: {} (voice: {}) ===", req.text, req.voice);

    // Determine TTS provider
    let tts_provider = TtsProvider::from_env();

    // Validate API key if using OpenAI
    if tts_provider == TtsProvider::OpenAI {
        if std::env::var("OPENAI_API_KEY").is_err() {
            eprintln!("Error: OPENAI_API_KEY not set but DEFAULT_TTS=OPENAI");
            return;
        }
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
                    println!("Note: Edge TTS does not support instructions parameter (OpenAI only)");
                }
                get_edge_tts(&text, &voice, speed).await
            }
        }
    });

    // Play mic pop and pre-transmission static while TTS is being fetched/buffered
    println!("Playing mic pop and pre-transmission static while buffering voice...");
    tokio::task::spawn_blocking(|| {
        play_mic_pop_and_static(200);
    })
    .await
    .unwrap();

    // Wait for TTS to complete buffering
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

    // Now play Quindar tone followed immediately by the buffered voice
    let volume = req.volume;
    tokio::task::spawn_blocking(move || {
        if let Err(e) = play_quindar_and_audio(audio_bytes, volume) {
            eprintln!("Error playing audio: {}", e);
        }
    })
    .await
    .unwrap();

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

/// API handler to enqueue transmission requests
async fn play_tone_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PlayRequest>,
) -> impl IntoResponse {
    let mut log_msg = format!("Received request, adding to queue: {} (voice: {})", payload.text, payload.voice);
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

    let transmission = TransmissionRequest {
        text: payload.text,
        voice: payload.voice,
        instructions: payload.instructions,
        speed: payload.speed,
        volume: payload.volume,
    };

    if let Err(e) = state.tx.send(transmission) {
        eprintln!("Failed to enqueue transmission: {}", e);
        return format!("Error: Failed to queue transmission");
    }

    "Transmission queued successfully!".to_string()
}

#[tokio::main]
async fn main() {
    // Load .env file
    dotenv::dotenv().ok();

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

    let listener = tokio::net::TcpListener::bind("127.0.0.1:42069")
        .await
        .unwrap();

    // Show TTS provider information
    let tts_provider = TtsProvider::from_env();
    let tts_name = match tts_provider {
        TtsProvider::Edge => "Edge TTS (free)",
        TtsProvider::OpenAI => "OpenAI (premium)",
    };

    println!("Quindar Tone API server running on http://127.0.0.1:42069");
    println!("TTS Provider: {}", tts_name);
    println!("Transmission queue enabled - multiple requests will play sequentially");
    println!("Send a POST request with JSON body: {{\"text\": \"your message\"}}");
    println!("Example: curl -X POST http://127.0.0.1:42069/play -H 'Content-Type: application/json' -d '{{\"text\": \"Test message\"}}'");

    axum::serve(listener, app).await.unwrap();
}
