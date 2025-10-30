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

Download the binary for your platform from the [Releases page](../../releases):

- **Windows x64**: `quindar-tone-api-windows-x64.exe`
- **macOS Intel**: `quindar-tone-api-macos-x64`
- **macOS Apple Silicon**: `quindar-tone-api-macos-arm64`
- **Linux x64**: `quindar-tone-api-linux-x64`

```bash
# Windows: Double-click or run from terminal
quindar-tone-api-windows-x64.exe

# macOS/Linux: Make executable and run
chmod +x quindar-tone-api-*
./quindar-tone-api-*
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

1. **TTS generation** - Voice requested from your configured TTS provider
2. **Opening Quindar tone** (2500 Hz beep, 500ms) - Once TTS is ready
3. **Your voice message** - Plays immediately
4. **Closing Quindar tone** (2500 Hz beep, 250ms - shorter)
5. **Transmission complete**

#### THREE-NOTE-CHIME Mode

1. **TTS generation** - Voice requested from your configured TTS provider
2. **Three-note chime** (C-E-G ascending with echo and depth) - Once TTS is ready
3. **Your voice message** - Plays immediately
4. **Closing chime** - Same three-note pattern
5. **Transmission complete**

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

## Documentation

- **[Quindar-Break-In-Developer_guide.md](Quindar-Break-In-Developer_guide.md)** - Complete API reference with:
  - Detailed endpoint documentation
  - Voice option reference
  - Queue behavior explanation
  - Code examples in multiple languages
  - Advanced usage patterns
  - Error handling guide

## Using with WSL (Windows Subsystem for Linux)

### RECOMMENDED: Access Windows Binary from WSL

**The Problem:** WSL lacks direct ALSA audio device access, causing audio playback errors even though TTS generation works.

**The Solution:** Run the Windows binary and access it from WSL over the network. This provides full audio support through Windows' audio system.

#### Setup Steps

1. **Run the Windows binary with network access:**

On Windows PowerShell:
```powershell
# Set bind address to accept WSL connections
$env:BIND_ADDRESS="0.0.0.0:42069"
.\quindar-tone-api-windows-x64.exe
```

Or add to your `.env` file (in same directory as the binary):
```env
BIND_ADDRESS=0.0.0.0:42069
```

Then run normally:
```powershell
.\quindar-tone-api-windows-x64.exe
```

2. **Call from WSL using Windows host IP:**

From your WSL terminal:
```bash
# Get Windows host IP
WINDOWS_HOST=$(cat /etc/resolv.conf | grep nameserver | awk '{print $2}')

# Test the connection
curl -X POST http://$WINDOWS_HOST:42069/play \
  -H 'Content-Type: application/json' \
  -d '{"text": "Hello from WSL"}'
```

3. **Make it permanent (optional):**

Add to your WSL `~/.bashrc` or `~/.zshrc`:
```bash
# Quindar API Windows host
export QUINDAR_HOST=$(cat /etc/resolv.conf | grep nameserver | awk '{print $2}')
alias quindar-play='curl -X POST http://$QUINDAR_HOST:42069/play -H "Content-Type: application/json" -d'
```

Then use it like:
```bash
quindar-play '{"text": "Build complete!"}'
```

#### Why This Works

- Windows binary runs natively with full Windows audio support
- WSL can reach Windows services via the Windows host IP address
- Audio plays through your Windows audio system (speakers/headphones)
- No ALSA configuration needed in WSL
- Single binary serves both Windows and WSL applications

#### Security Note

Binding to `0.0.0.0` makes the API accessible to your local network. For localhost-only access with WSL support:
- Keep default `127.0.0.1:42069` binding
- Set up Windows Firewall to allow WSL subnet access to port 42069
- Or use `BIND_ADDRESS=172.16.0.0:42069` to limit to typical WSL subnet

## Troubleshooting

### ALSA Errors on Linux/WSL

If you see ALSA error messages like:
```
ALSA lib confmisc.c:855:(parse_card) cannot find card '0'
ALSA lib conf.c:5204:(_snd_config_evaluate) function snd_func_card_inum returned error
```

These are expected when running on WSL (Windows Subsystem for Linux) or headless Linux systems without audio hardware.

**BEST SOLUTION for WSL: See [Using with WSL](#using-with-wsl-windows-subsystem-for-linux) section above** - Run the Windows binary and access it from WSL.

**Alternative - Headless Mode (no audio):**

Run with `HEADLESS_MODE=true` to completely eliminate ALSA errors:

```bash
# One-time run
HEADLESS_MODE=true ./quindar-tone-api-linux-x64

# Or add to .env file
echo "HEADLESS_MODE=true" >> .env
./quindar-tone-api-linux-x64
```

**What headless mode does:**
- ✅ **Eliminates ALL ALSA errors** - No audio device creation attempted
- ✅ **TTS still generated** - Voice synthesis works normally
- ✅ **API returns success** - Perfect for headless servers, Docker, CI/CD
- ⚠️ **No audio playback** - Audio is generated but not played

**Other Alternatives:**

1. **Suppress ALSA errors** (if you prefer logging):
```bash
cat > ~/.asoundrc << 'EOF'
pcm.!default {
    type plug
    slave.pcm "null"
}
EOF
```

2. **Redirect stderr**:
```bash
./quindar-tone-api-linux-x64 2>/dev/null
```

3. **Enable audio in WSL2** (complex, not recommended): Follow [Microsoft's WSL audio guide](https://learn.microsoft.com/en-us/windows/wsl/tutorials/gui-apps)

### No Audio Output

- Ensure audio is not muted and volume is turned up
- On Linux, verify ALSA/PulseAudio is configured correctly
- **On WSL, use the Windows binary** - See [Using with WSL](#using-with-wsl-windows-subsystem-for-linux) section
- Try the test request in the Quick Start section to verify functionality

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
