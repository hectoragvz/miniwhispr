# Miniwhispr

A personal, always-on desktop dictation tool: you hold or toggle a fixed shortcut, speak, and clean, punctuated text lands in whatever app you are typing in.

## Language

### Dictating

**Dictation**:
One complete cycle of speaking and receiving text, from the moment listening starts until the text is inserted or discarded.
_Avoid_: Recording, session, transcription (as a noun for the whole cycle)

**Push-to-Talk**:
The mode where a Dictation lasts exactly as long as the shortcut is held; releasing it ends the Dictation.
_Avoid_: Hold mode, PTT

**Hands-Free Mode**:
The mode where one shortcut press starts a Dictation and a later press ends it, with no key held in between.
_Avoid_: Toggle mode, continuous mode, lock mode

**Last Dictation**:
The final text of the most recent Dictation, kept only until the next one replaces it or the app quits, so it can be recovered if Insertion missed.
_Avoid_: History, recent, clipboard

### Turning speech into text

**Transcript**:
The raw text produced from the spoken audio, before any Cleanup.
_Avoid_: Raw text, ASR output

**Cleanup**:
The step that turns a Transcript into the final text by applying punctuation, removing filler words, and resolving self-corrections to match what the speaker meant.
_Avoid_: Post-processing, polish, formatting

**Voice Command**:
A spoken phrase that Cleanup turns into formatting instead of words, such as "new line" or "nuevo párrafo".
_Avoid_: Command, keyword, trigger phrase

**Insertion**:
Placing the final text into whichever app has focus at the moment the text is ready, even if focus moved after the Dictation ended.
_Avoid_: Pasting, typing, output

### Feedback

**Toast**:
The small, text-free floating indicator shown during a Dictation: animated voice waves while listening, a spinner while processing, and a red flash and shake when the Dictation fails.
_Avoid_: Popup, HUD, overlay, notification
