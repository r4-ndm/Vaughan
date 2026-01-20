# Custom Audio Alert Sounds

Vaughan wallet supports custom audio alert sounds for incoming coin notifications.

## How to Use Custom Audio Alerts

1. **Supported Audio Formats:**
   - WAV (.wav)
   - MP3 (.mp3)
   - OGG (.ogg)
   - FLAC (.flac)

2. **Setup Custom Sound:**
   Place one of the following audio files in your `config/` directory:
   - `alert.wav` or `alert.mp3`
   - `notification.wav` or `notification.mp3`
   - `coin_sound.wav` or `coin_sound.mp3`

3. **Automatic Detection:**
   The wallet will automatically detect and use your custom audio file when:
   - Balance increases (indicating incoming coins)
   - Real-time transaction monitoring detects incoming transfers

4. **File Requirements:**
   - Maximum file size: 10MB
   - Must have valid audio format extension
   - File should be relatively short (1-5 seconds recommended)

## Example Setup

```bash
# Create config directory if it doesn't exist
mkdir -p config

# Copy your custom sound (replace with your actual audio file)
cp ~/Downloads/my_coin_sound.wav config/alert.wav

# Test the custom audio
./target/release/test_audio config/alert.wav
```

## Testing Custom Audio

You can test your custom audio files using the built-in test utility:

```bash
# Test default beep sound
./target/release/test_audio

# Test custom audio file
./target/release/test_audio path/to/your/audio.wav

# Test with config directory file
./target/release/test_audio config/alert.wav
```

## Fallback Behavior

- If no custom audio file is found, the wallet uses a default 800Hz beep sound
- If a custom audio file fails to load, it falls back to the default beep
- Error messages are logged for debugging if audio loading fails

## Audio Triggers

Custom audio alerts are triggered when:
- **Balance increases** detected through the 3-second auto-refresh system
- **Incoming transactions** are detected in real-time
- **Token transfers** to your wallet addresses

## Tips

- Use short, pleasant sounds to avoid interrupting your workflow
- Test your audio files with the test utility before using them
- Keep file sizes small for quick loading
- Consider the volume level of your audio files

Enjoy your personalized wallet notifications! 🎵💰