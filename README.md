# Quindar Tone API

[![MIT License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.87%2B-orange.svg)](https://www.rust-lang.org/)

A Rust application that plays a Quindar tone followed by text-to-speech audio when called via API request.

**For complete API documentation, see [Quindar-Break-In-Developer_guide.md](Quindar-Break-In-Developer_guide.md)**

> 💡 **Free for commercial use** - Licensed under MIT. If this project saves you time or adds value to your work, consider [sponsoring development](#-support-this-project)!

## What is a Quindar Tone?

Quindar tones are beep tones that were used in NASA communications to signal when the ground-to-air radio link was activated. This implementation uses a 2500 Hz tone for 500ms with radio static before and after the beep.

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
- **No API key required**
- Uses Microsoft Edge TTS (free service)
- Excellent quality neural voices
- See available voices: `edge-tts --list-voices`
- Requires: `pip install edge-tts`

### OpenAI TTS (Premium)
```env
DEFAULT_TTS=OPENAI
OPENAI_API_KEY=your_actual_api_key_here
```
- Requires OpenAI API key
- Supports `instructions` parameter for voice personalization
- Uses OpenAI's premium TTS voices

## Running the Application

```bash
cargo run
```

The server will start on `http://127.0.0.1:42069`

## Usage

Send a POST request to the `/play` endpoint with JSON containing your text:

```bash
curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{"text": "Hello, this is a test message"}'
```

### Voice Options

You can optionally specify a voice (defaults to `alloy`):

```bash
curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "This is a test with a different voice.",
    "voice": "echo"
  }'
```

**Available voices:** `alloy` (default), `echo`, `fable`, `onyx`, `nova`, `shimmer`

### Voice Personalization

Add character and emotion with optional `instructions`, `speed`, and `volume` parameters:

```bash
curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "This is an urgent alert!",
    "voice": "shimmer",
    "instructions": "Speak with urgency",
    "speed": 1.15,
    "volume": 2.5
  }'
```

- **`instructions`**: Guide how the voice should deliver the text (e.g., "Speak calmly and clearly")
- **`speed`**: Playback speed from 0.25 (very slow) to 4.0 (very fast), default 1.0
- **`volume`**: Volume gain from 0.1 (very quiet) to 5.0 (very loud), default 2.0

See [Quindar-Break-In-Developer_guide.md](Quindar-Break-In-Developer_guide.md) for detailed examples and recommendations.

### What happens:
1. TTS voice is requested from OpenAI (starts buffering immediately)
2. **Mic pop** - subtle microphone activation sound
3. Radio static plays (200ms) - while TTS is buffering in background
4. Once TTS is fully buffered, **opening Quindar tone** plays (2500 Hz beep, 500ms)
5. Post-transmission static (200ms)
6. OpenAI TTS voice speaks your text (no delay!)
7. **Closing Quindar tone** (2500 Hz beep, 250ms - shorter)
8. Transmission complete!

The approach eliminates any awkward pause between the tone and voice by buffering the TTS while playing the pre-transmission static.

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

```bash
# Example with echo voice (clear and authoritative)
curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "System check complete. All parameters are within normal range.",
    "voice": "echo"
  }'

# Example with nova voice (friendly)
curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "Process completed successfully.",
    "voice": "nova"
  }'

# Alert with shimmer voice (energetic)
curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "Warning! Anomaly detected.",
    "voice": "shimmer"
  }'
```

For more examples and integration patterns, see [Quindar-Break-In-Developer_guide.md](Quindar-Break-In-Developer_guide.md).

## TTS Provider Comparison

| Feature | Edge TTS (Default) | OpenAI TTS (Premium) |
|---------|-------------------|----------------------|
| Cost | Free | Requires API key |
| Quality | Excellent (neural) | Excellent (neural) |
| Setup | `pip install edge-tts` | API key required |
| Voice Options | 100+ voices, 40+ languages | 6 voices (alloy, echo, fable, onyx, nova, shimmer) |
| Instructions | ❌ Not supported | ✅ Supported |
| Speed Control | ✅ Supported | ✅ Supported |
| Best For | Development, free tier, multi-language | Production with personalization needs |

## Dependencies

- **axum**: Web framework for the HTTP API
- **tokio**: Async runtime
- **rodio**: Audio playback library
- **reqwest**: HTTP client for OpenAI API
- **serde**: JSON serialization
- **dotenv**: Environment variable management
- **rand**: Random number generation for radio static
- **uuid**: Unique identifiers for temp files
- **edge-tts**: Free, high-quality TTS (Python package)

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
