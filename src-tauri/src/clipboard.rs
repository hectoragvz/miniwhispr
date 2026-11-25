//! The macOS clipboard. AppKit does not promise the clipboard works from any
//! thread but the main one, so call everything here on the main thread.

use objc2::runtime::ProtocolObject;
use objc2_app_kit::{NSPasteboard, NSPasteboardItem, NSPasteboardTypeString};
use objc2_foundation::{NSArray, NSData, NSString};

/// Tells clipboard managers the text is only passing through, so they do not
/// keep a copy of it (nspasteboard.org).
const TRANSIENT_TYPE: &str = "org.nspasteboard.TransientType";

/// Everything that was on the clipboard: each item with its data in every
/// type it offered. Plain bytes, so it can wait on any thread.
pub struct Contents(Vec<Vec<(String, Vec<u8>)>>);

/// Identifies one write to the clipboard; any later write changes it.
#[derive(Clone, Copy, PartialEq)]
pub struct Change(isize);

/// Replaces what is on the clipboard with `text`.
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

/// What is on the clipboard now.
pub fn save() -> Contents {
    let pasteboard = NSPasteboard::generalPasteboard();
    let items = pasteboard.pasteboardItems().unwrap_or_default();
    Contents(
        items
            .iter()
            .map(|item| {
                item.types()
                    .iter()
                    .filter_map(|kind| {
                        let data = item.dataForType(&kind)?;
                        Some((kind.to_string(), data.to_vec()))
                    })
                    .collect()
            })
            .collect(),
    )
}

/// Puts `text` on the clipboard to be pasted straight away. It goes in one
/// item together with the transient mark, so a clipboard manager never sees
/// the text without it.
pub fn write_for_paste(text: &str) -> Option<Change> {
    let item = NSPasteboardItem::new();
    // SAFETY: NSPasteboardTypeString is a constant AppKit defines.
    let written = item
        .setString_forType(&NSString::from_str(text), unsafe { NSPasteboardTypeString })
        && item.setData_forType(&NSData::new(), &NSString::from_str(TRANSIENT_TYPE));
    let pasteboard = NSPasteboard::generalPasteboard();
    pasteboard.clearContents();
    let items = [ProtocolObject::from_retained(item)];
    (written && pasteboard.writeObjects(&NSArray::from_retained_slice(&items)))
        .then(|| Change(pasteboard.changeCount()))
}

/// Whether anything was written to the clipboard after `ours`.
pub fn changed_since(ours: Change) -> bool {
    Change(NSPasteboard::generalPasteboard().changeCount()) != ours
}

/// Puts `contents` back on the clipboard.
pub fn restore(contents: Contents) {
    let pasteboard = NSPasteboard::generalPasteboard();
    pasteboard.clearContents();
    let items: Vec<_> = contents
        .0
        .iter()
        .map(|types| {
            let item = NSPasteboardItem::new();
            for (kind, data) in types {
                item.setData_forType(&NSData::with_bytes(data), &NSString::from_str(kind));
            }
            ProtocolObject::from_retained(item)
        })
        .collect();
    if !items.is_empty() && !pasteboard.writeObjects(&NSArray::from_retained_slice(&items)) {
        log::error!("Could not restore the clipboard");
    }
}
