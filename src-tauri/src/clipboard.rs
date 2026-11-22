//! The macOS clipboard.

use objc2_app_kit::{NSPasteboard, NSPasteboardTypeString};
use objc2_foundation::NSString;

/// Replaces what is on the clipboard with `text`. Call on the main thread:
/// AppKit does not promise the clipboard works from others.
pub fn copy(text: &str) {
    let pasteboard = NSPasteboard::generalPasteboard();
    pasteboard.clearContents();
    // SAFETY: NSPasteboardTypeString is a constant AppKit defines.
    let copied =
        pasteboard.setString_forType(&NSString::from_str(text), unsafe { NSPasteboardTypeString });
    if !copied {
        log::error!("Could not copy the Last Dictation to the clipboard");
    }
}
