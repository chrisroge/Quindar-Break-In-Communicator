# Quindar Break-In API - Developer Guide

## Overview

The Quindar Break-In API is a **pure Rust** audio communication service designed for AI agents to get human attention. It plays text using configurable Text-to-Speech (Edge TTS or OpenAI TTS), complete with NASA Quindar tones, radio static, and microphone pops.

**Perfect for AI Agents:** This API provides a simple HTTP endpoint that any autonomous agent can call to interrupt and notify users with audible messages - no complex integrations required.

## Table of Contents

- [Quick Start](#quick-start)
- [TTS Provider Configuration](#tts-provider-configuration)
- [API Endpoint](#api-endpoint)
- [Request Format](#request-format)
- [Response Format](#response-format)
- [Voice Options](#voice-options)
- [Queue Behavior](#queue-behavior)
- [Audio Sequence](#audio-sequence)
- [Examples](#examples)
- [Error Handling](#error-handling)
- [Best Practices](#best-practices)

## Quick Start

```bash
curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{"text": "Hello, this is a test message"}'
```

## TTS Provider Configuration

The API supports two TTS providers that can be configured via environment variables:

### Edge TTS (Default - Free)

**Configuration (.env):**
```env
DEFAULT_TTS=EDGE
EDGE_VOICE=en-US-AndrewNeural
```

**Features:**
- ✅ **Free** - No API key required
- ✅ **High Quality** - Neural voices from Microsoft Edge
- ✅ **100+ Voices** - Multiple languages and accents
- ✅ **Fast** - Low latency
- ✅ **Pure Rust** - Native implementation, no Python dependencies
- ❌ **No Instructions** - Does not support voice personalization

**Available Voices:**
See [Microsoft's full voice list](https://speech.microsoft.com/portal/voicegallery). Popular options:
- `en-US-AndrewNeural` - Confident, warm male (default)
- `en-US-AriaNeural` - Professional female
- `en-US-GuyNeural` - Clear male voice
- `en-US-JennyNeural` - Friendly female
- Many more in 40+ languages

### OpenAI TTS (Premium)

**Configuration (.env):**
```env
DEFAULT_TTS=OPENAI
OPENAI_API_KEY=your_api_key_here
```

**Features:**
- 💰 **Paid** - Requires OpenAI API key
- ✅ **High Quality** - Premium neural voices
- ✅ **6 Voices** - alloy, echo, fable, onyx, nova, shimmer
- ✅ **Instructions** - Voice personalization support
- ✅ **Consistent** - Reliable for production

**Best For:**
- Production deployments
- When voice personalization (instructions) is needed
- Commercial applications with budget

### Provider Comparison

| Feature | Edge TTS | OpenAI TTS |
|---------|----------|-----------|
| Cost | Free | Paid (API key) |
| API Key Required | ❌ No | ✅ Yes |
| Voice Count | 100+ | 6 |
| Languages | 40+ | English primary |
| Instructions Support | ❌ | ✅ |
| Speed Control | ✅ | ✅ |
| Dependencies | None (built-in) | API key only |
| Best Use Case | Development, multi-language | Production, personalization |

### Switching Providers

Simply change the `DEFAULT_TTS` value in your `.env` file and restart the server:

```bash
# Use free Edge TTS
DEFAULT_TTS=EDGE

# Use premium OpenAI TTS
DEFAULT_TTS=OPENAI
```

The API endpoints and request format remain the same regardless of provider.

## API Endpoint

### POST /play

Queues a text message for transmission with Quindar tone break-in.

**Base URL:** `http://127.0.0.1:42069`

**Endpoint:** `/play`

**Method:** `POST`

**Content-Type:** `application/json`

## Request Format

### Request Body

The request body must be valid JSON with the following structure:

```json
{
  "text": "Your message here",
  "voice": "alloy",
  "instructions": "Speak with a sense of urgency",
  "speed": 1.0,
  "volume": 2.0
}
```

### Parameters

| Parameter      | Type   | Required | Default  | Description                                             |
|----------------|--------|----------|----------|---------------------------------------------------------|
| `text`         | string | Yes      | -        | The message to be spoken (max ~4096 chars)              |
| `voice`        | string | No       | `"alloy"` | The OpenAI TTS voice to use (see Voice Options)         |
| `instructions` | string | No       | -        | Instructions to personalize the voice delivery          |
| `speed`        | number | No       | `1.0`    | Playback speed (0.25 to 4.0, where 1.0 is normal speed) |
| `volume`       | number | No       | `2.0`    | Volume gain multiplier (0.1 to 5.0, where 1.0 is original volume) |

### Example Request

```json
{
  "text": "System status update: all parameters within normal range.",
  "voice": "echo",
  "instructions": "Speak calmly and professionally",
  "speed": 0.95,
  "volume": 2.5
}
```

## Response Format

### Success Response

**Status Code:** `200 OK`

**Response Body:**

```
Transmission queued successfully!
```

The API responds immediately after queuing your request. The actual audio playback happens asynchronously in the background.

### Error Response

**Status Code:** `200 OK` (with error message in body)

**Response Body:**

```
Error: Failed to queue transmission
```

## Voice Options

The API supports all OpenAI TTS voices. Choose the voice that best fits your use case:

| Voice     | Description                                    | Best For                          |
|-----------|------------------------------------------------|-----------------------------------|
| `alloy`   | Neutral, balanced voice                        | General purpose (default)         |
| `echo`    | Clear, authoritative male voice                | Commands, formal announcements    |
| `fable`   | Warm, expressive voice                         | Storytelling, narration           |
| `onyx`    | Deep, rich male voice                          | Dramatic announcements            |
| `nova`    | Friendly, approachable female voice            | Status updates, notifications     |
| `shimmer` | Bright, energetic female voice                 | Alerts, urgent communications     |

### Voice Example

```bash
# Using the "echo" voice for clear announcements
curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "Initiating process sequence",
    "voice": "echo"
  }'
```

### Default Voice Behavior

If you omit the `voice` parameter, the API defaults to `"alloy"`:

```json
{
  "text": "Message without voice parameter"
}
```

This is equivalent to:

```json
{
  "text": "Message without voice parameter",
  "voice": "alloy"
}
```

## Voice Personalization

### Instructions Parameter

The `instructions` parameter allows you to guide how the voice should deliver your text. This is an experimental feature that can add character and emotion to the transmission.

**Examples:**

```bash
# Urgent announcement
curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "Emergency alert! Critical system error detected!",
    "voice": "shimmer",
    "instructions": "Speak with urgency and concern"
  }'

# Calm, professional delivery
curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "All checkpoints completed. Proceeding to next phase.",
    "voice": "echo",
    "instructions": "Speak calmly and authoritatively"
  }'

# Excited announcement
curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "Success! All tests passed with optimal results!",
    "voice": "nova",
    "instructions": "Speak with enthusiasm and excitement"
  }'
```

**Tips for instructions:**
- Be specific about the emotion or tone (urgent, calm, excited, concerned)
- Keep instructions concise (under 100 characters works best)
- Combine with appropriate voice selection for best results

### Speed Parameter

The `speed` parameter controls playback speed, ranging from 0.25 (very slow) to 4.0 (very fast). Default is 1.0 (normal speed).

**Use cases:**

```bash
# Slow for clarity
curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "Attention: Initiating backup protocol sequence.",
    "voice": "onyx",
    "speed": 0.9
  }'

# Fast for urgent alerts
curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "Warning! Immediate action required!",
    "voice": "shimmer",
    "speed": 1.15,
    "instructions": "Speak urgently"
  }'

# Very slow for emphasis
curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "System critical failure detected.",
    "voice": "alloy",
    "speed": 0.75
  }'
```

**Speed recommendations:**
- **0.25-0.75:** Very slow, for emphasis or clarity
- **0.8-0.95:** Slightly slower, for important announcements
- **1.0:** Normal speed (default)
- **1.05-1.25:** Slightly faster, for urgent messages
- **1.3-4.0:** Very fast, for time-compressed audio

### Volume Parameter

The `volume` parameter controls the volume gain/loudness of the voice transmission. Default is 2.0 (2x amplification) to match the loudness of the Quindar tones and static.

**Use cases:**

```bash
# Quiet message
curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "Background process initiated.",
    "voice": "alloy",
    "volume": 0.8,
    "instructions": "Speak quietly"
  }'

# Normal volume (default)
curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "All systems operating normally.",
    "voice": "echo",
    "volume": 2.0
  }'

# Loud, clear message
curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "ATTENTION! EMERGENCY PROTOCOL ACTIVATED!",
    "voice": "shimmer",
    "volume": 3.5,
    "instructions": "Speak loudly and urgently"
  }'
```

**Volume recommendations:**
- **0.1-0.7:** Very quiet, distant or weak signal
- **0.8-1.5:** Quieter than default
- **2.0:** Default (matches Quindar tone loudness)
- **2.5-3.5:** Louder, clear and strong
- **4.0-5.0:** Very loud (may distort on some systems)

**Note:** Values above 3.0 may cause audio distortion/clipping depending on your system. Use with caution.

### Combining Voice, Instructions, Speed, and Volume

For maximum impact, combine all four parameters:

```bash
curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "Critical alert: System resources at fifteen percent and declining. Immediate action required.",
    "voice": "echo",
    "instructions": "Speak with controlled urgency",
    "speed": 1.05,
    "volume": 2.5
  }'
```

## Queue Behavior

### Sequential Processing

The Quindar Break-In API uses a **queue system** to ensure transmissions never overlap. Multiple requests are automatically queued and played sequentially.

**How it works:**

1. Your API request returns immediately after queuing
2. A background worker processes transmissions one at a time
3. Each transmission completes fully before the next begins
4. Queue is processed in FIFO (First In, First Out) order

### Example: Multiple Concurrent Requests

```bash
# Send three messages rapidly
curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{"text": "First message"}' &

curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{"text": "Second message"}' &

curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{"text": "Third message"}' &
```

**Result:** All three requests return immediately, but audio plays sequentially:
1. First message plays completely
2. Second message plays completely
3. Third message plays completely

## Audio Sequence

Each request follows this audio sequence:

### 1. Pre-Transmission Phase (Buffering)
- **Mic Pop** (30ms) - Subtle microphone activation sound
- **Radio Static** (200ms) - While TTS audio is buffering from OpenAI

### 2. Break-In Signal
- **Opening Quindar Tone** (500ms) - 2500 Hz beep with smooth fade in/out

### 3. Transmission Phase
- **Post-Quindar Static** (200ms) - Brief crackle after tone
- **Voice Message** - Your text spoken using OpenAI TTS (no delay)

### 4. Close-Out Signal
- **Closing Quindar Tone** (250ms) - Shorter 2500 Hz beep signaling end

**Total Overhead:** ~1 second of audio effects per message (excluding voice length)

### Timing Diagram

```
┌─────────────────────────────────────────────────────────────┐
│ [pop] [static] [BEEEEP] [static] "Your message" [BEEP]      │
│  30ms   200ms    500ms    200ms    (variable)    250ms      │
└─────────────────────────────────────────────────────────────┘
   ↑                                                    ↑
   Pre-transmission                              Close-out
   (TTS buffering happens here)
```

## Examples

### Basic Usage

```bash
curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{"text": "Test message received"}'
```

### With Custom Voice

```bash
curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "Process completed successfully.",
    "voice": "echo"
  }'
```

### Multiple Messages (Queue)

```bash
# Multiple messages in sequence
curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{"text": "Starting sequence", "voice": "nova"}'

curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{"text": "Process initiated", "voice": "echo"}'

curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{"text": "Sequence complete", "voice": "echo"}'
```

### Using cURL with JSON File

**message.json:**
```json
{
  "text": "Alert: Anomaly detected in system output. Recommend immediate review.",
  "voice": "onyx"
}
```

**Command:**
```bash
curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d @message.json
```

### JavaScript/Node.js Example

```javascript
const axios = require('axios');

async function sendMessage(text, voice = 'alloy') {
  try {
    const response = await axios.post('http://127.0.0.1:42069/play', {
      text: text,
      voice: voice
    });
    console.log(response.data);
  } catch (error) {
    console.error('Request failed:', error.message);
  }
}

// Send a single message
sendMessage('This is a test message', 'echo');

// Send multiple messages (they will queue)
sendMessage('First message', 'nova');
sendMessage('Second message', 'echo');
sendMessage('Third message', 'alloy');
```

### Python Example

```python
import requests

def send_message(text, voice='alloy'):
    url = 'http://127.0.0.1:42069/play'
    payload = {
        'text': text,
        'voice': voice
    }

    try:
        response = requests.post(url, json=payload)
        print(response.text)
    except Exception as e:
        print(f'Request failed: {e}')

# Send messages
send_message('This is a test message', 'echo')
send_message('All systems operational', 'nova')
```

### Shell Script Example

```bash
#!/bin/bash

# Function to send message
send_message() {
    local text="$1"
    local voice="${2:-alloy}"  # Default to alloy if not specified

    curl -X POST http://127.0.0.1:42069/play \
        -H 'Content-Type: application/json' \
        -d "{\"text\": \"$text\", \"voice\": \"$voice\"}"

    echo ""
}

# Usage examples
send_message "Starting process" "echo"
send_message "Step one complete" "nova"
send_message "Step two complete" "onyx"
```

## Error Handling

### Common Issues

#### 1. OPENAI_API_KEY Not Set

**Symptom:** Server logs show `Error: OPENAI_API_KEY not set`

**Solution:** Ensure `.env` file exists with valid API key:

```bash
OPENAI_API_KEY=sk-your-actual-key-here
```

#### 2. Invalid JSON

**Request:**
```bash
curl -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d 'not valid json'
```

**Response:** Request will fail with JSON parsing error

**Solution:** Ensure valid JSON format

#### 3. Missing Text Parameter

**Request:**
```json
{
  "voice": "echo"
}
```

**Response:** Request will fail - `text` is required

**Solution:** Always include `text` parameter

#### 4. Invalid Voice Name

**Request:**
```json
{
  "text": "Test",
  "voice": "invalid_voice"
}
```

**Response:** OpenAI API will return an error

**Solution:** Use one of the valid voice options: `alloy`, `echo`, `fable`, `onyx`, `nova`, `shimmer`

### Debugging Tips

1. **Check server logs** - The API prints detailed information about each request:
   ```
   === Processing transmission: Your message (voice: echo) ===
   Requesting TTS from OpenAI with voice 'echo'...
   Voice buffered successfully!
   Playing opening Quindar tone...
   Playing voice transmission...
   Playing closing Quindar tone...
   Transmission complete!
   ```

2. **Test with minimal message** - Start with simple test:
   ```bash
   curl -X POST http://127.0.0.1:42069/play \
     -H 'Content-Type: application/json' \
     -d '{"text": "test"}'
   ```

3. **Verify audio output** - Ensure your system's audio output is working and volume is up

## Best Practices

### 1. Message Length

- **Recommended:** Keep messages under 500 characters for best experience
- **Maximum:** OpenAI TTS supports up to ~4096 characters
- **Why:** Very long messages delay the queue for subsequent requests

### 2. Voice Selection

- Use consistent voices for similar types of messages
- `echo` and `onyx` work well for authoritative announcements
- `nova` and `shimmer` work well for status updates and alerts
- `alloy` is a safe default for general use

### 3. Queue Management

- The queue is unbounded - be mindful of sending too many requests at once
- Each request takes: ~1 second (effects) + TTS duration
- For time-sensitive messages, consider implementing a priority system in your app

### 4. Text Formatting

- Use proper punctuation for natural-sounding speech
- Acronyms work well: "NASA", "ISS", "EVA"
- Spell out numbers that should be read individually: "T minus 10"
- Add pauses with commas or periods

**Good:**
```json
{
  "text": "Process complete. All checks passed successfully."
}
```

**Less Natural:**
```json
{
  "text": "process complete all checks passed successfully"
}
```

### 5. Error Resilience

Always handle the response, even though it's simple:

```javascript
const response = await fetch('http://127.0.0.1:42069/play', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ text: message, voice: 'echo' })
});

const result = await response.text();
if (result.includes('Error')) {
  console.error('Request failed:', result);
}
```

## Advanced Usage

### Building a Notification System

```javascript
class AudioNotifier {
  constructor(baseUrl = 'http://127.0.0.1:42069') {
    this.baseUrl = baseUrl;
    this.voices = {
      info: 'alloy',      // Information
      warning: 'nova',    // Warnings
      error: 'shimmer',   // Errors
      success: 'echo'     // Success messages
    };
  }

  async notify(level, message) {
    const voice = this.voices[level] || this.voices.info;

    const response = await fetch(`${this.baseUrl}/play`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ text: message, voice })
    });

    return await response.text();
  }

  // Level-specific methods
  info(message) { return this.notify('info', message); }
  warning(message) { return this.notify('warning', message); }
  error(message) { return this.notify('error', message); }
  success(message) { return this.notify('success', message); }
}

// Usage
const notifier = new AudioNotifier();

await notifier.info('Process started.');
await notifier.warning('Low memory detected.');
await notifier.success('Operation completed successfully.');
```

### Countdown Timer Example

```bash
#!/bin/bash

# Countdown from 10
for i in {10..1}; do
  curl -s -X POST http://127.0.0.1:42069/play \
    -H 'Content-Type: application/json' \
    -d "{\"text\": \"$i\", \"voice\": \"echo\"}" > /dev/null
  sleep 1
done

curl -s -X POST http://127.0.0.1:42069/play \
  -H 'Content-Type: application/json' \
  -d '{"text": "Timer complete", "voice": "echo"}' > /dev/null
```

## Technical Details

### Audio Specifications

- **Quindar Tones:** 2500 Hz sine wave
- **Sample Rate:** 48000 Hz
- **Channels:** Mono (1 channel)
- **Format:** OpenAI returns MP3, decoded to PCM for playback
- **Volume:** Normalized to prevent clipping

### Performance

- **Queue Processing:** Asynchronous, non-blocking
- **TTS Buffering:** Happens during pre-transmission static (parallel)
- **Latency:** ~200ms from API call to first audio (mic pop)
- **Throughput:** Limited by TTS generation time + audio playback time

### Requirements

- Rust application must be running on `http://127.0.0.1:42069`
- Valid OpenAI API key in `.env` file
- System audio output configured and working
- Network access to OpenAI API (api.openai.com)

## Support

For issues, questions, or feature requests, please refer to the main README.md or check the server logs for detailed debugging information.

---

**Last Updated:** 2025-10-22
**API Version:** 0.1.0
**Compatibility:** OpenAI TTS API v1
