# 02: Push-to-Talk with the Toast

**Spec:** `.scratch/miniwhispr-v1/spec.md` (see "Dictation lifecycle" and "Testing Decisions")

**What to build:** Holding Ctrl + Space starts a Dictation in Push-to-Talk: a soft start sound plays and the Toast appears at the bottom centre above the Dock, showing voice waves that follow the real microphone level. Releasing plays the stop sound, shows the spinner, and then the Toast hides (nothing is transcribed or inserted yet). A quick Ctrl + Space tap still reaches the focused app. This ticket introduces the Dictation core and its test seam, which every later ticket builds on.

**Blocked by:** 01 (Menu bar app skeleton, CI and repo basics)

**Status:** resolved

- [x] Dictation core exists as a module with no macOS, network or UI knowledge, depending only on Recorder, Transcriber, Cleaner, Inserter, Feedback and Clock interfaces (Transcriber, Cleaner and Inserter may be no-op real implementations for now)
- [x] Core tests with fakes and a controllable clock cover: tap under 250 ms requests pass-through with no Toast and no sound; hold reaching 250 ms starts listening with start sound and listening Toast; release produces stop sound, processing Toast, then hidden; presses while processing are ignored and not passed through; the 10-minute limit ends the Dictation as if released
- [x] Ctrl + Space is captured globally via an event tap and swallowed: holding it types no spaces into any text field
- [x] A Ctrl + Space tap under 250 ms is re-emitted to the focused app (VS Code autocomplete still opens)
- [x] Recorder captures from the current default input device, follows default-device changes between Dictations, and records 16 kHz mono
- [x] Toast is a borderless, transparent, always-on-top, non-activating window that never takes focus
- [x] Toast renders listening (waves responding to level), processing (spinner) and hidden, never shows text; covered by Vitest tests per state
- [x] Soft start and stop sounds play
