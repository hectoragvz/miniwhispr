//! The Voice Commands: spoken phrases that Cleanup turns into formatting
//! instead of words. This one list drives both the Cleanup instructions and
//! the "Voice Commands" menu, so the menu always shows what Cleanup does.

pub struct VoiceCommand {
    pub english: &'static str,
    pub spanish: &'static str,
    /// What saying it does to the text, in a few words.
    pub effect: &'static str,
}

pub const VOICE_COMMANDS: [VoiceCommand; 2] = [
    VoiceCommand {
        english: "new line",
        spanish: "nueva línea",
        effect: "line break",
    },
    VoiceCommand {
        english: "new paragraph",
        spanish: "nuevo párrafo",
        effect: "blank line",
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cleanup, menu_bar};

    #[test]
    fn every_voice_command_is_in_the_cleanup_instructions() {
        let instructions = cleanup::instructions();

        for command in &VOICE_COMMANDS {
            let line = format!(
                "\"{}\" or \"{}\": a {}",
                command.english, command.spanish, command.effect
            );
            assert!(instructions.contains(&line), "missing {line}");
        }
    }

    #[test]
    fn every_voice_command_has_a_menu_entry_with_both_phrases_and_its_effect() {
        let entries = menu_bar::voice_command_entries();

        assert_eq!(
            entries,
            vec![
                "“new line” / “nueva línea”: line break",
                "“new paragraph” / “nuevo párrafo”: blank line",
            ]
        );
    }
}
