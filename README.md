# Quindar Tone API

[![MIT License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.87%2B-orange.svg)](https://www.rust-lang.org/)

A Rust application that plays a Quindar tone followed by text-to-speech audio when called via API request.

**For complete API documentation, see [Quindar-Break-In-Developer_guide.md](Quindar-Break-In-Developer_guide.md)**

> 💡 **Free for commercial use** - Licensed under MIT. If this project saves you time or adds value to your work, consider [sponsoring development](#-support-this-project)!

## What is a Quindar Tone?

Quindar tones are the iconic "beeps" heard during NASA's Apollo missions. Named after Quindar Electronics Inc., these in-band signaling tones (2525 Hz opening, 2475 Hz closing) were used to remotely key transmitters at ground stations when CapCom (capsule communicator) needed to speak with astronauts in space. Rather than building expensive dedicated control lines between NASA's mission control and remote tracking stations worldwide, engineers cleverly embedded these 250ms audio tones to turn transmitters on and off—a cost-effective solution that became one of the most recognizable sounds of the space age.

## Why This Matters for AI Agents

In the era of autonomous AI agents, there's a critical need for agents to get human attention when necessary—whether for approvals, urgent notifications, or time-sensitive decisions. Just as NASA's CapCom used Quindar tones to break into radio communications with astronauts, AI agents need a reliable "break-in" mechanism to interrupt and notify people.

This API provides a lightweight, native service that any AI agent can call via simple HTTP requests. Running locally on your machine, it gives agents an immediate, audible way to capture your attention with familiar NASA-style tones followed by spoken messages. No complex notification systems, no dependencies on cloud services—just a straightforward API that any agent can integrate with a single curl command.

## Quick Start

### Option 1: Download Pre-compiled Binary (Recommended)

Download the latest binary for your platform from the [Releases page](../../releases):

- **Linux**: `quindar-tone-api-linux-x86_64`
- **macOS**: `quindar-tone-api-macos-x86_64` or `quindar-tone-api-macos-aarch64` (Apple Silicon)
- **Windows**: `quindar-tone-api-windows-x86_64.exe`

```bash
# Linux/macOS: Make executable and run
chmod +x quindar-tone-api-*
./quindar-tone-api-*

# Windows: Double-click or run from terminal
quindar-tone-api-windows-x86_64.exe
```

**No configuration needed!** The binary runs immediately with sensible defaults:
- **TTS Provider**: Edge TTS (free, no API key required)
- **Voice**: en-US-AndrewNeural (professional male voice)
- **Tone Type**: Quindar (classic NASA tones)

Optional: Create a `.env` file in the same directory as the binary to customize settings (see [Setup](#setup) section below).

### Option 2: Build from Source

**Requirements**:
- Rust 1.87+ ([install from rustup.rs](https://rustup.rs/))
- **Linux only**: System libraries for compilation
  - **pkg-config**: `sudo apt install pkg-config` (Debian/Ubuntu) or `sudo yum install pkg-config` (Fedora/RHEL)
  - **OpenSSL development libraries**: `sudo apt install libssl-dev` (Debian/Ubuntu) or `sudo yum install openssl-devel` (Fedora/RHEL)
  - **ALSA development libraries**: `sudo apt install libasound2-dev` (Debian/Ubuntu) or `sudo yum install alsa-lib-devel` (Fedora/RHEL)

```bash
git clone https://github.com/chrisroge/Quindar-Break-In-Communicator.git
cd Quindar-Break-In-Communicator
./install.sh  # Interactive setup (checks and guides you through dependencies)
# Or manually:
cp .env.example .env
cargo run --release
```

**Note**: macOS and Windows users typically don't need additional system libraries.

## Setup

1. Copy `.env.example` to `.env`:
```bash
cp .env.example .env
```

2. **Choose your TTS provider** in `.env`:

### Edge TTS (Default - Free)
```env
DEFAULT_TTS=EDGE
EDGE_VOICE=en-US-AndrewNeural
```
- **No API key required** - Pure Rust implementation
- Uses Microsoft Edge TTS (free service)
- Excellent quality neural voices
- 100+ voices in 40+ languages
- **No Python dependencies required**

### OpenAI TTS (Premium)
```env
DEFAULT_TTS=OPENAI
OPENAI_API_KEY=your_actual_api_key_here
```
- Requires OpenAI API key
- Supports `instructions` parameter for voice personalization
- Uses OpenAI's premium TTS voices

3. **Choose your tone type** in `.env`:

```env
DEFAULT_TONE=QUINDAR
```

**Available tone types:**
- **QUINDAR** (default) - Classic NASA Quindar tones (2500 Hz beep before/after voice)
- **THREE-NOTE-CHIME** - Audience recall chime (C-E-G, like theater/concert hall)
- **NO-TONE** - No tones, just voice

You can also override the tone per-request (see examples below).

## Running the Application

```bash
cargo run
```

The server will start on `http://127.0.0.1:42069`

## Usage

### Basic Request

Send a POST request to the `/play` endpoint with JSON containing your text:

```bash
curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{"text": "Hello, this is a test message"}'
```

This uses the default TTS provider configured in your `.env` file.

### Voice Options

**Voice options depend on your TTS provider:**

#### Using Edge TTS (Default - Free)

Specify any Edge TTS voice name from Microsoft's 100+ available voices.

**Popular English voices:**
```bash
# Male voices
curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "Testing Edge TTS male voice",
    "voice": "en-US-AndrewNeural"
  }'

# Female voices
curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "Testing Edge TTS female voice",
    "voice": "en-US-AriaNeural"
  }'
```

**Common Edge TTS voices:**
- `en-US-AndrewNeural` - Confident, warm male (default)
- `en-US-AriaNeural` - Professional female
- `en-US-GuyNeural` - Clear male
- `en-US-JennyNeural` - Friendly female
- `en-GB-RyanNeural` - British male
- `en-GB-SoniaNeural` - British female

**Or omit the `voice` parameter** to use your `EDGE_VOICE` from `.env`

#### Using OpenAI TTS (Premium)

If you set `DEFAULT_TTS=OPENAI`, use one of these 6 voices:

```bash
curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "Testing OpenAI TTS",
    "voice": "echo"
  }'
```

**OpenAI voices:** `alloy`, `echo`, `fable`, `onyx`, `nova`, `shimmer`

| Voice | Description |
|-------|-------------|
| `alloy` | Neutral, balanced |
| `echo` | Clear, authoritative male |
| `fable` | Warm, expressive |
| `onyx` | Deep, rich male |
| `nova` | Friendly female |
| `shimmer` | Bright, energetic female |

### Voice Personalization

Control playback with optional parameters:

#### Available for Both Providers (Edge TTS + OpenAI)

- **`speed`**: Playback speed from 0.25 (very slow) to 4.0 (very fast), default 1.0
- **`volume`**: Volume gain from 0.1 (very quiet) to 5.0 (very loud), default 2.0

```bash
# Works with both Edge TTS and OpenAI
curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "This is an urgent alert!",
    "voice": "en-US-AndrewNeural",
    "speed": 1.15,
    "volume": 2.5
  }'
```

#### OpenAI Only

- **`instructions`**: Guide how the voice should deliver the text (e.g., "Speak with urgency")

```bash
# Only works with DEFAULT_TTS=OPENAI
curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "Critical system alert",
    "voice": "shimmer",
    "instructions": "Speak with urgency and concern",
    "speed": 1.15,
    "volume": 2.5
  }'
```

**Note:** If you use `instructions` with Edge TTS, it will be ignored (with a log message).

See [Quindar-Break-In-Developer_guide.md](Quindar-Break-In-Developer_guide.md) for detailed examples and recommendations.

### Tone Options

Override the default tone type for individual requests using the `tone` parameter:

```bash
# Use the three-note audience recall chime (theater/concert hall style)
curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "Please return to your seats, the presentation will begin shortly",
    "voice": "en-US-AndrewNeural",
    "tone": "THREE-NOTE-CHIME"
  }'

# Play voice only, no tones
curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "Simple announcement without any tones",
    "voice": "en-US-JennyNeural",
    "tone": "NO-TONE"
  }'

# Use classic Quindar tone (override if DEFAULT_TONE is different)
curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "Mission control style announcement",
    "voice": "en-US-GuyNeural",
    "tone": "QUINDAR"
  }'
```

**Available tone values:**
- `QUINDAR` - Classic NASA Quindar tones (2500 Hz beep before/after)
- `THREE-NOTE-CHIME` - Audience recall chime (C-E-G ascending, xylophone-like with echo)
- `NO-TONE` - Voice only, no tones

If omitted, uses the `DEFAULT_TONE` from your `.env` file.

### Audio Sequence

The audio sequence varies based on your tone type selection:

#### QUINDAR Mode (Default)

1. **TTS generation** - Voice requested from your configured TTS provider (starts buffering immediately)
2. **Mic pop** - Subtle microphone activation sound
3. **Radio static** (200ms) - Plays while TTS is buffering in background
4. **Opening Quindar tone** (2500 Hz beep, 500ms) - Once TTS is fully buffered
5. **Post-transmission static** (200ms)
6. **Your voice message** - Plays with no delay!
7. **Closing Quindar tone** (2500 Hz beep, 250ms - shorter)
8. **Transmission complete**

#### THREE-NOTE-CHIME Mode

1. **TTS generation** - Voice buffering begins
2. **Mic pop** - Subtle microphone activation sound
3. **Radio static** (200ms) - Plays while buffering
4. **Three-note chime** (C-E-G ascending with echo and depth)
5. **Your voice message** - Plays immediately after chime
6. **Closing chime** - Same three-note pattern
7. **Transmission complete**

#### NO-TONE Mode

1. **TTS generation** - Voice buffering
2. **Your voice message** - Plays directly, no tones or static
3. **Transmission complete**

This approach eliminates any awkward pause between tones and voice by buffering the TTS during the pre-transmission audio.

### Request Queuing

Multiple requests are automatically queued and played sequentially - no overlapping audio! You can send multiple requests rapidly and they will play one after another in order:

```bash
# Send multiple messages - they'll queue up automatically
curl -X POST http://127.0.0.1:42069/play -H 'Content-Type: application/json' -d '{"text": "First message"}' &
curl -X POST http://127.0.0.1:42069/play -H 'Content-Type: application/json' -d '{"text": "Second message"}' &
curl -X POST http://127.0.0.1:42069/play -H 'Content-Type: application/json' -d '{"text": "Third message"}' &
```

Each request returns immediately while a background worker processes them one at a time.

## Example Messages

### Using Edge TTS (Default)

```bash
# Professional male voice
curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "System check complete. All parameters are within normal range.",
    "voice": "en-US-GuyNeural"
  }'

# Friendly female voice
curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "Process completed successfully.",
    "voice": "en-US-JennyNeural"
  }'

# British voice with speed adjustment
curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "Warning! Anomaly detected.",
    "voice": "en-GB-RyanNeural",
    "speed": 1.1,
    "volume": 2.5
  }'
```

### Using OpenAI TTS (Premium)

```bash
# Set DEFAULT_TTS=OPENAI in .env first

# Clear authoritative voice
curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "System check complete.",
    "voice": "echo"
  }'

# With instructions for personalization
curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "Critical alert! Immediate action required!",
    "voice": "shimmer",
    "instructions": "Speak with urgency and concern",
    "speed": 1.15,
    "volume": 3.0
  }'
```

For more examples and integration patterns, see [Quindar-Break-In-Developer_guide.md](Quindar-Break-In-Developer_guide.md).

## TTS Provider Comparison

| Feature | Edge TTS (Default) | OpenAI TTS (Premium) |
|---------|-------------------|----------------------|
| Cost | Free | Requires API key |
| Quality | Excellent (neural) | Excellent (neural) |
| Setup | No dependencies - Pure Rust | API key required |
| Voice Options | 100+ voices, 40+ languages | 6 voices (alloy, echo, fable, onyx, nova, shimmer) |
| Instructions | ❌ Not supported | ✅ Supported |
| Speed Control | ✅ Supported | ✅ Supported |
| Best For | Development, free tier, multi-language | Production with personalization needs |

## Dependencies

All dependencies are pure Rust - no Python or external tools required!

- **axum**: Web framework for the HTTP API
- **tokio**: Async runtime
- **rodio**: Audio playback library
- **msedge-tts**: Native Rust client for Microsoft Edge TTS API
- **reqwest**: HTTP client for OpenAI API
- **serde**: JSON serialization
- **dotenv**: Environment variable management
- **rand**: Random number generation for radio static

## Documentation

- **[Quindar-Break-In-Developer_guide.md](Quindar-Break-In-Developer_guide.md)** - Complete API reference with:
  - Detailed endpoint documentation
  - Voice option reference
  - Queue behavior explanation
  - Code examples in multiple languages
  - Advanced usage patterns
  - Error handling guide

## 💖 Support This Project

This project is **free and open source** (MIT License) - use it however you'd like, including commercial projects!

If Quindar Tone API saves you time or adds value to your work, consider supporting its development:

- ⭐ **Star this repo** - Helps others discover it
- 🐛 **Report bugs** - Submit issues on GitHub
- 💡 **Suggest features** - Open a discussion
- 💰 **Sponsor development** - Click the "Sponsor" button above (coming soon!)
  - GitHub Sponsors (0% fees)
  - Ko-fi or Buy Me a Coffee options

Your support helps maintain this project and add new features like:
- Additional TTS providers (Piper, Coqui)
- Voice cloning support
- Cloud-hosted API option
- Advanced audio effects

## License

This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.

**What this means:**
- ✅ Commercial use allowed
- ✅ Modification allowed
- ✅ Distribution allowed
- ✅ Private use allowed
- ❌ No liability or warranty

## Note

Make sure your audio output is enabled and volume is turned up to hear the tone and voice.
