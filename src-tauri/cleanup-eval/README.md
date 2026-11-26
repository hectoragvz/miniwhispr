# Cleanup evaluation

`cases.json` holds Transcripts in English and Spanish, each with the final text (or texts) Cleanup may turn it into. They cover punctuation (with ¿ and ¡), capitalization, fillers, self-corrections, Voice Commands, and questions and requests that must not be answered.

It calls Groq with your real key, so CI never runs it. Run it to choose the Cleanup model, and before any change to the Cleanup instructions in `src/cleanup.rs`:

```sh
cargo test --lib cleanup_evaluation -- --ignored --nocapture
```

It tests `MODEL` from `src/cleanup.rs`. To compare candidates, list them:

```sh
CLEANUP_MODELS="openai/gpt-oss-120b,openai/gpt-oss-20b" cargo test --lib cleanup_evaluation -- --ignored --nocapture
```

The key comes from the Keychain. After each build, macOS asks whether the test may read it. Choose **Always Allow**.

`results.md` records the last evaluation and why the model was chosen.
