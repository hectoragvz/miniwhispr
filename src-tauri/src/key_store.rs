//! The Groq API key, kept in the macOS Keychain and nowhere else: never in a
//! config file, the log or the repo.

use security_framework::passwords::{get_generic_password, set_generic_password};

const SERVICE: &str = "com.hectoragvz.miniwhispr";
const ACCOUNT: &str = "groq-api-key";
/// Keychain's `errSecItemNotFound`: no key saved yet.
const NOT_FOUND: i32 = -25300;

/// The saved key, or `None` if there is none yet.
pub fn read() -> Result<Option<String>, String> {
    match get_generic_password(SERVICE, ACCOUNT) {
        Ok(bytes) => String::from_utf8(bytes)
            .map(Some)
            .map_err(|_| "The saved Groq API key is not valid text".to_string()),
        Err(e) if e.code() == NOT_FOUND => Ok(None),
        Err(e) => Err(format!(
            "Could not read the Groq API key from the Keychain: {e}"
        )),
    }
}

/// Saves `key`, replacing any saved one.
pub fn save(key: &str) -> Result<(), String> {
    set_generic_password(SERVICE, ACCOUNT, key.as_bytes())
        .map_err(|e| format!("Could not save the Groq API key to the Keychain: {e}"))
}
