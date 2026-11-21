# Spec: Miniwhispr v1

**Status:** ready-for-agent

## Problem Statement

I type a lot, and speaking is faster than typing, but macOS's built-in dictation is clumsy: it has to be started per app, it leaves filler words and run-on sentences in the text, and it does not understand when I correct myself mid-sentence. Commercial tools like Wispr Flow solve this but are paid, closed, and send everything to a service I do not control. I want a small, always-on tool of my own: hold a key, speak in English or Spanish, release, and clean, correctly punctuated text appears wherever my cursor is.

## Solution

Miniwhispr is a menu bar app for macOS that launches at login and stays out of the way. Holding **Ctrl + Space** starts a Dictation in Push-to-Talk: a small, text-free Toast appears at the bottom of the screen with voice waves that follow my voice, and a soft sound confirms it is listening. Releasing the keys ends the Dictation: the Toast switches to a spinner, the audio is turned into a Transcript, the Transcript goes through Cleanup (punctuation, capitalization, filler removal, self-corrections, Voice Commands), and the final text is inserted into whichever app has focus, with my clipboard restored afterwards. If anything goes wrong, the Toast flashes red. The final text is always recoverable from the menu as the Last Dictation until the next Dictation replaces it.

The project is open source and clone-and-build: anyone with a Mac, the build prerequisites and their own Groq API key can build and run it.

## User Stories

### Starting and ending a Dictation

1. As a user, I want to hold Ctrl + Space to start a Dictation, so that I can speak without clicking anything.
2. As a user, I want the Dictation to end the moment I release the keys, so that Push-to-Talk feels direct.
3. As a user, I want a hold shorter than 250 ms to be treated as a normal Ctrl + Space tap and passed through to the focused app, so that editor autocomplete and other Ctrl + Space shortcuts keep working.
4. As a user, I want a hold of 250 ms or longer to always be a Dictation in every app, so that the behaviour is predictable.
5. As a user, I want the Ctrl + Space key presses swallowed while I hold them, so that no spaces or other characters are typed into my document.
6. As a user, I want a new Ctrl + Space hold to be ignored while the previous Dictation is still processing, so that two Dictations never race or insert out of order.
7. As a user, I want a Dictation to end and insert automatically after 10 minutes, so that a stuck key or forgotten hold cannot run forever.
8. As a user, I want releasing the keys to always proceed to Insertion, so that I never lose what I said by accident (there is no cancel in v1).

### Feedback while dictating

9. As a user, I want a small pill-shaped Toast at the bottom centre of the screen, above the Dock, so that I know a Dictation is in progress.
10. As a user, I want the Toast to show animated voice waves that follow my actual microphone level while listening, so that I can see it is hearing me.
11. As a user, I want the Toast to show a spinner between release and Insertion, so that I know the text is on its way.
12. As a user, I want the Toast to contain no text at all, so that it stays minimal and unobtrusive.
13. As a user, I want the Toast to never take focus, so that Insertion lands in the app I was working in.
14. As a user, I want the Toast to flash red and shake briefly, then disappear, when something fails, so that I know the Dictation did not produce text.
15. As a user, I want the Toast to disappear once the text is inserted, so that it does not linger.
16. As a user, I want a soft sound when listening starts and another when it stops, so that I know Push-to-Talk is working without looking at the screen.
17. As a user, I want the menu bar icon to stay static during a Dictation, so that the menu bar stays calm.

### Microphone

18. As a user, I want the app to use the current macOS default input device, so that I do not have to configure a microphone.
19. As a user, I want the app to follow the default input when it changes, for example when I connect headphones, so that the next Dictation uses the right microphone automatically.

### Transcript and Cleanup

20. As a user, I want to dictate in English or Spanish and have the language detected for each Dictation, so that I do not have to switch modes.
21. As a user, I want the Transcript to be steered toward English or Spanish, so that short Spanish Dictations are less likely to be mistaken for Portuguese or Galician.
22. As a user, I want my pauses and intonation turned into commas, periods and question marks (including ¿ and ¡ in Spanish), so that the text reads the way I spoke it.
23. As a user, I want filler words such as "um", "uh", "like", "este", "o sea" removed when they are filler, so that the text is clean.
24. As a user, I want my self-corrections applied, for example "at 3, no, at 4" becoming "at 4", so that the text matches what I meant.
25. As a user, I want correct capitalization, so that I do not have to fix it by hand.
26. As a user, I want Cleanup to keep my words and tone and never rephrase, summarize or add content, so that the text still sounds like me.
27. As a user, I want to say "new line" or "nueva línea" to insert a line break, so that I can format while speaking.
28. As a user, I want to say "new paragraph" or "nuevo párrafo" to insert a blank line, so that I can structure longer text.
29. As a user, I want accented and special characters (á, é, ñ, ü, ¿, ¡) inserted correctly in every app, so that Spanish text is never garbled.

### Insertion

30. As a user, I want the final text inserted at my cursor in one piece, so that it appears instantly regardless of length.
31. As a user, I want my clipboard restored to what it was before the Dictation, so that dictating does not destroy what I had copied.
32. As a user, I want the text inserted into whichever app has focus when the text is ready, so that the behaviour is predictable even if I switched apps while it was processing.
33. As a user, I want the final text kept as the Last Dictation, so that I can recover it if Insertion landed nowhere (for example, no text field was focused).
34. As a user, I want "Copy last dictation" in the menu to copy the Last Dictation to my clipboard, so that I can paste it where I meant to.
35. As a user, I want the Last Dictation kept only in memory and never written to disk, so that my dictated text is not stored.

### Failure handling

36. As a user, I want a failed transcription (no network, Groq error, timeout) to be retried once automatically, so that brief glitches do not cost me a Dictation.
37. As a user, I want the red Toast if the retry also fails, so that I know to dictate again.
38. As a user, I want the raw Transcript inserted if Cleanup fails, so that I still get my words rather than nothing.
39. As a user, I want the red Toast and nothing inserted when no speech was detected, so that silence or an accidental hold does not insert junk.
40. As a user, I want the red Toast when the microphone or Accessibility permission is missing at Dictation time, so that I learn something needs fixing.

### Menu bar

41. As a user, I want Miniwhispr to live only in the menu bar with no Dock icon, so that it behaves like a background utility.
42. As a user, I want the menu to show whether the Microphone and Accessibility permissions are granted, so that I can spot setup problems at a glance.
43. As a user, I want a "Voice Commands" submenu listing every Voice Command with its English and Spanish phrase and its effect, so that I can remember what I can say.
44. As a user, I want the Voice Commands submenu to come from the same list Cleanup uses, so that the reference can never drift from the real behaviour.
45. As a user, I want a "Groq API key…" menu item, so that I can change my key later.
46. As a user, I want a "Launch at login" toggle, on by default, so that Miniwhispr is always available after a restart.
47. As a user, I want a Quit item, so that I can stop the app.

### First-run setup

48. As a new user, I want a first-run window that walks me through setup in order, so that I end up with a working app without reading docs.
49. As a new user, I want to paste my Groq API key in step one and have it stored in the macOS Keychain, so that it is never in a config file or the repo.
50. As a new user, I want to grant the Microphone permission in step two, so that the app can hear me.
51. As a new user, I want to grant the Accessibility permission in step three, so that the app can catch Ctrl + Space and insert text.
52. As a new user, I want step four to check whether macOS's "Select the previous input source" shortcut is enabled on Ctrl + Space and tell me how to turn it off, so that the two do not conflict.
53. As a new user, I want each step to show a live check mark once it is done, so that I know when I can move on.
54. As a returning user, I want the first-run window not to appear again once setup is complete, so that startup is silent.

### Open source and building

55. As someone discovering the repo, I want a README listing the prerequisites (Xcode Command Line Tools, Rust, Node and pnpm, a Groq API key) and the build commands, so that I can clone and build it myself.
56. As a developer, I want a one-time script that creates a local self-signed code-signing certificate and uses it for every build, so that macOS keeps my Accessibility and Microphone permissions across rebuilds.
57. As a developer, I want the project MIT-licensed, so that others can use and modify it freely.
58. As a developer, I want CI on every push that runs Rust format, lint and tests and TypeScript typecheck, lint and tests, with no network or API key, so that main stays green.
59. As a developer, I want a manual Cleanup evaluation I can run locally with my key, so that I can choose the Groq model and catch regressions when the Cleanup instructions change.

## Implementation Decisions

### Platform and stack

- macOS only in v1; Windows is a planned later target (ADR 0001).
- Tauri application: a Rust core owns all behaviour; a React + TypeScript UI renders the Toast, the first-run setup window, and nothing else. The menu bar menu is native (Tauri tray menu), not React.
- The app runs as a menu-bar-only agent app (no Dock icon) and registers itself as a login item, toggleable from the menu.
- App name "Miniwhispr" everywhere; bundle identifier `com.hectoragvz.miniwhispr`.

### Modules

- **Dictation core** (Rust, deep module, the main test seam): owns the Dictation lifecycle as a state machine. It receives inputs (shortcut pressed, shortcut released, input level samples, clock ticks) and drives outputs through narrow interfaces. It holds the rules: the 250 ms hold threshold, tap pass-through, ignore-while-processing, the 10-minute limit, retry-once, Cleanup fallback, no-speech handling, Last Dictation. It has no knowledge of macOS, Groq, or the UI.
- **Interfaces the core depends on** (each a Rust trait with one real implementation and one test fake):
  - **Recorder**: start, stop and return the captured audio, publish input levels.
  - **Transcriber**: audio in, Transcript out (or a failure).
  - **Cleaner**: Transcript in, final text out (or a failure).
  - **Inserter**: final text in, inserts it into the focused app.
  - **Feedback**: receives Toast state changes (hidden, listening with level, processing, error) and start/stop sound cues.
  - **Clock**: current time and timers, so tests control time.
- **Shortcut adapter** (macOS): a keyboard event tap that watches Ctrl + Space, swallows it, reports press/release to the core, and re-emits a Ctrl + Space tap when the core decides a hold was under 250 ms. Requires Accessibility.
- **Recorder adapter** (macOS): captures from the current default input device, follows default-device changes between Dictations, encodes 16 kHz mono audio compact enough to stay within Groq's upload limit for 10 minutes.
- **Groq Transcriber**: calls Groq's speech-to-text endpoint with Whisper large-v3, with a prompt hint that the speaker uses English or Spanish; no explicit language parameter so detection stays automatic.
- **Groq Cleaner**: calls a Groq-hosted chat model with the Cleanup instructions. The exact model is chosen with the Cleanup evaluation and is a single configuration constant.
- **Voice Command list**: one shared, fixed list of Voice Commands (phrase per language, effect). Cleanup builds its instructions from it, and the menu builds the Voice Commands submenu from it. v1 contains: "new line" / "nueva línea" (line break) and "new paragraph" / "nuevo párrafo" (blank line).
- **Paste Inserter** (macOS): saves the clipboard, writes the final text, simulates Cmd+V into the focused app, then restores the previous clipboard contents.
- **Key store** (macOS Keychain): stores and reads the Groq API key.
- **Permissions probe** (macOS): reports Microphone and Accessibility status and whether the "Select the previous input source" shortcut is enabled, for the menu and first-run setup.
- **Toast window**: a borderless, transparent, always-on-top, non-activating window at bottom centre above the Dock, rendering the React Toast from state events emitted by the core.
- **First-run setup window**: React, four ordered steps (Groq key, Microphone, Accessibility, input-source shortcut), each with a live check mark driven by the Permissions probe and Key store. Shown only until all steps are complete.

### Dictation lifecycle (behaviour of the core)

- Idle → shortcut pressed → pending. If released before 250 ms: request a pass-through tap, back to Idle, no Toast, no sound.
- Pending reaches 250 ms → Listening: start recorder, play start sound, Toast listening with live levels.
- Listening → shortcut released, or 10 minutes elapsed → play stop sound, Toast processing, stop recorder.
- Processing: transcribe; on failure retry once; on second failure → error. Empty or no-speech Transcript → error. Otherwise Cleanup; on Cleanup failure use the raw Transcript. Insert, store as Last Dictation, Toast hidden, back to Idle.
- Error: Toast error briefly, then hidden, back to Idle. Nothing inserted; Last Dictation unchanged.
- Shortcut presses during Processing or Error are ignored (and not passed through).

### Cleanup contract

- Conservative: punctuation (including Spanish ¿ ¡), capitalization, filler removal, self-correction resolution, Voice Commands. Never rephrase, summarize, translate, answer questions in the text, or add content.
- Output language matches the spoken language.
- Output is the final text only, with no preamble or quotes.

### Error surface

- All failures surface only as the red Toast in v1; no text, no notifications. Details go to the app log for debugging.

## Testing Decisions

- **What makes a good test:** tests exercise external behaviour through a public interface and assert on observable outcomes (what got inserted, which Toast states were emitted, what the Last Dictation is), never on internal state or call order beyond what the user would notice.
- **Dictation core (primary seam, Rust unit tests in CI):** drive the core with fake Recorder, Transcriber, Cleaner, Inserter, Feedback and a controllable Clock. Cover: tap under 250 ms passes through with no Toast; hold at 250 ms starts listening with start sound; release inserts the cleaned text and restores Idle; presses during processing are ignored; transcription failure retries once then succeeds; two failures produce the error Toast and no Insertion; Cleanup failure inserts the raw Transcript; no-speech produces the error Toast; the 10-minute limit ends and inserts; Last Dictation updates on success and is untouched on error.
- **Voice Command list (Rust unit tests in CI):** the Cleanup instructions and the menu entries are both derived from the same list and include every command.
- **Toast UI (Vitest in CI):** each state renders correctly (waves respond to level, spinner, error shake, hidden) and never renders text.
- **Cleanup evaluation (manual, real Groq, not in CI):** about 20 Transcript/expected-text pairs in English and Spanish covering punctuation, fillers, self-corrections, Voice Commands and "do not answer the question in the text". Used to choose the Groq model and before any change to the Cleanup instructions.
- **Not automatically tested:** macOS adapters (event tap, paste, recorder, Keychain, login item, permissions) and the Groq HTTP clients. They stay thin and are covered by the manual QA checklist below.
- **CI:** GitHub Actions on every push and PR: `cargo fmt --check`, `cargo clippy` with warnings as errors, `cargo test`, `tsc`, ESLint, Vitest. No network or secrets.
- **Prior art:** none yet; this is a greenfield repo. The Dictation core tests become the pattern for future tests.

### Manual QA checklist

- Ctrl + Space tap in VS Code still opens autocomplete.
- Holding Ctrl + Space types no spaces into a text field.
- Dictation works in Notes, Slack, a browser text field, and a terminal.
- Spanish Dictation with á, ñ, ¿, ¡ inserts correctly.
- Clipboard contents before a Dictation are present after it.
- Plugging in headphones switches the next Dictation to their microphone.
- Turning off Wi-Fi gives the red Toast; "Copy last dictation" still has the previous text.
- Toast never steals focus and sits above the Dock.
- Quit, reboot, and confirm Miniwhispr is back in the menu bar.
- Rebuild and confirm permissions survive (self-signed certificate).

## Out of Scope

- Hands-Free Mode (toggle dictation); planned later with Fn + Space.
- Windows support (planned, see ADR 0001).
- On-device transcription (Parakeet), offline mode (planned privacy upgrade, see ADR 0002).
- Customisable shortcuts; the shortcut is fixed to Ctrl + Space.
- Cancelling a Dictation (Esc).
- Dictation history beyond the in-memory Last Dictation.
- Personal dictionary, per-app styles, context awareness (active app or screen text).
- Spoken lists to bullet points and Voice Commands beyond new line and new paragraph; user-defined Voice Commands.
- Language switching mid-sentence; a manual language selector.
- Microphone picker.
- Streaming text while speaking.
- Prebuilt .dmg, Developer ID signing, notarization, auto-updates, GitHub Releases.
- Accounts, billing, telemetry.

## Further Notes

- Glossary: see `GLOSSARY.md` for Dictation, Push-to-Talk, Hands-Free Mode, Transcript, Cleanup, Voice Command, Insertion, Last Dictation, Toast.
- ADRs: 0001 (Tauri + React for a Mac-only v1), 0002 (Groq cloud for transcription and Cleanup).
- Groq bills each transcription request for at least 10 seconds of audio; at personal usage this is negligible (about $0.11 per audio hour for Whisper large-v3).
- Target latency from release to Insertion: about one second.
- Ctrl + Space collides with "Select the previous input source" in macOS when multiple input sources are enabled; first-run setup handles this.
