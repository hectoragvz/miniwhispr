# 03: Groq API key and transcription

**Spec:** `.scratch/miniwhispr-v1/spec.md`

**What to build:** I can enter my Groq API key from a "Groq API key…" menu item; it is stored in the macOS Keychain. When a Dictation ends, its audio is turned into a Transcript by Groq Whisper large-v3, steered toward English or Spanish, and kept as the Last Dictation. "Copy last dictation" in the menu copies it to the clipboard, so the full speech-to-text path is usable end to end before automatic Insertion exists.

**Blocked by:** 02 (Push-to-Talk with the Toast)

**Status:** ready-for-agent

- [ ] "Groq API key…" menu item opens a small React window to enter or replace the key
- [ ] The key is stored in and read from the macOS Keychain only; never in a config file, log or the repo
- [ ] Groq Transcriber sends the Dictation audio to Groq Whisper large-v3 with a prompt hint for English or Spanish and no fixed language parameter
- [ ] Audio encoding keeps a 10-minute Dictation within Groq's upload limit
- [ ] A successful Dictation stores the Transcript as the Last Dictation, in memory only
- [ ] "Copy last dictation" menu item copies the Last Dictation to the clipboard (disabled when there is none)
- [ ] Core tests cover: successful transcription updates the Last Dictation
- [ ] Manual check: an English and a Spanish Dictation (with á, ñ, ¿) both produce correct Transcripts via "Copy last dictation"
