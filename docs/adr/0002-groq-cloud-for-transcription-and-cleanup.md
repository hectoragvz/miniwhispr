# Groq cloud for both transcription and Cleanup

Each Dictation's audio is sent to Groq for transcription (Whisper large-v3), and the Transcript is sent to a Groq-hosted language model for Cleanup. We accepted that audio leaves the machine in exchange for a single provider and API key, sub-second latency (close to Wispr Flow's under-700 ms target, which is also cloud-only), and negligible cost for personal use (about $0.11 per audio hour). Transcription sits behind an interface so an on-device engine can replace it later.

## Considered Options

- **On-device Parakeet v3 (via Core ML):** free, private, offline, and fast on Apple Silicon, with strong English and Spanish; deferred because it adds a ~600 MB model download and native glue code. It is the planned privacy/offline upgrade.
- **OpenAI or Deepgram:** two to four times the cost with no latency advantage for upload-after-release audio.
- **Claude Haiku for Cleanup:** strong bilingual instruction following, but a second account and key; Groq keeps everything on one provider.
