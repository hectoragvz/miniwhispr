# 05: Cleanup and Voice Commands

**Spec:** `.scratch/miniwhispr-v1/spec.md` (see "Cleanup contract")

**What to build:** Every Transcript goes through Cleanup before Insertion: punctuation (including ¿ and ¡), capitalization, filler removal and self-corrections, keeping my words and tone. Saying "new line" / "nueva línea" or "new paragraph" / "nuevo párrafo" formats the text. A "Voice Commands" submenu lists every Voice Command. If Cleanup fails, the raw Transcript is inserted instead.

**Blocked by:** 04 (Insertion and failure handling)

**Status:** ready-for-agent

- [ ] A single fixed Voice Command list (phrase per language plus effect) drives both the Cleanup instructions and the "Voice Commands" submenu
- [ ] Rust tests confirm every Voice Command appears in both the Cleanup instructions and the menu entries
- [ ] Groq Cleaner applies the conservative Cleanup contract: never rephrases, summarizes, translates, answers questions in the text, or adds content; output language matches the spoken language; output is the final text only
- [ ] The Cleanup model is a single configuration constant
- [ ] Cleanup failure inserts the raw Transcript; covered by a core test
- [ ] "Voice Commands" submenu shows each command's English and Spanish phrase and its effect, read-only
- [ ] Manual Cleanup evaluation: about 20 Transcript/expected-text pairs in English and Spanish (punctuation, fillers, self-corrections, Voice Commands, a question that must not be answered), runnable locally with the Groq key, outside CI
- [ ] The evaluation is run against candidate Groq models and the chosen model's results are recorded alongside it
