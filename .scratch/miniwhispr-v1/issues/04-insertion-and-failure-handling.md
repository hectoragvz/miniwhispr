# 04: Insertion and failure handling

**Spec:** `.scratch/miniwhispr-v1/spec.md` (see "Dictation lifecycle" and "Error surface")

**What to build:** When a Dictation's text is ready it is inserted at the cursor of whichever app has focus, in one piece, and my clipboard is restored afterwards. Failures are handled the agreed way: a failed transcription is retried once, then the Toast flashes red and shakes; a Dictation with no speech gives the red Toast and inserts nothing.

**Blocked by:** 03 (Groq API key and transcription)

**Status:** ready-for-agent

- [ ] Paste Inserter saves the clipboard, writes the text, simulates Cmd+V into the focused app, then restores the previous clipboard contents
- [ ] Text goes to whichever app has focus when the text is ready
- [ ] Accented and special characters (á, é, ñ, ü, ¿, ¡) insert correctly
- [ ] Transcription failure is retried once automatically; a second failure shows the error Toast and inserts nothing
- [ ] An empty or no-speech Transcript shows the error Toast and inserts nothing
- [ ] On any error the Last Dictation is unchanged
- [ ] Toast error state (red flash and shake, then hidden, no text) covered by a Vitest test
- [ ] Core tests cover: success inserts the text; fail-then-succeed inserts once; two failures give error and no Insertion; no-speech gives error and no Insertion; Last Dictation untouched on error
- [ ] Manual check: dictation works in Notes, Slack, a browser text field and a terminal; clipboard contents survive; Wi-Fi off gives the red Toast
