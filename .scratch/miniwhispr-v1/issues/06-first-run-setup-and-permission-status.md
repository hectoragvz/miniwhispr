# 06: First-run setup and permission status

**Spec:** `.scratch/miniwhispr-v1/spec.md`

**What to build:** On first launch a setup window walks me through four steps in order: Groq API key, Microphone permission, Accessibility permission, and a check that macOS's "Select the previous input source" shortcut is not on Ctrl + Space. Each step shows a live check mark when done, and the window does not appear again once setup is complete. The menu shows whether each permission is granted, and a Dictation attempted with a missing permission gives the red Toast.

**Blocked by:** 04 (Insertion and failure handling)

**Status:** ready-for-agent

- [ ] First-run window shows the four steps in order, each with a live check mark driven by the real Keychain and permission status
- [ ] Step one reuses the Groq API key entry from ticket 03
- [ ] Microphone and Accessibility steps open the relevant System Settings pane and tick once granted, without restarting the app
- [ ] Input-source step detects whether "Select the previous input source" is enabled on Ctrl + Space and explains how to turn it off
- [ ] The window does not appear on later launches once all steps are complete
- [ ] Menu shows Microphone and Accessibility status
- [ ] A Dictation started with a missing permission shows the error Toast and inserts nothing; covered by a core test
- [ ] Manual check: reset permissions with `tccutil`, relaunch, complete setup, and dictate successfully
