//! The Paste Inserter: inserts the final text into whichever app has focus
//! by putting it on the clipboard and pressing Cmd + V, then gives the
//! clipboard back what it had. Pasting inserts any text in one piece, with
//! every character (á, ñ, ¿, ¡) intact. Needs the Accessibility permission.

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use objc2_core_graphics::{CGEventFlags, CGKeyCode};
use tauri::AppHandle;

use crate::clipboard::{self, Change, Contents};
use crate::dictation::Inserter;
use crate::shortcut;

/// The V key, on the US layout and the layouts based on it.
const V: CGKeyCode = 9;
/// How long the focused app gets to read the text off the clipboard before
/// it is restored. Apps read it as they handle Cmd + V, which is quick, but a
/// busy app may take a moment.
const RESTORE_DELAY: Duration = Duration::from_secs(1);

pub struct PasteInserter {
    app: AppHandle,
    /// The paste whose clipboard restore is still waiting, if any. Only used
    /// on the main thread, like the clipboard.
    waiting: Arc<Mutex<Option<Waiting>>>,
}

struct Waiting {
    /// What the clipboard had before Miniwhispr wrote to it.
    saved: Contents,
    /// Miniwhispr's write.
    ours: Change,
}

impl PasteInserter {
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            waiting: Arc::default(),
        }
    }
}

impl Inserter for PasteInserter {
    fn insert(&mut self, text: &str) {
        let text = text.to_string();
        let app = self.app.clone();
        let waiting = self.waiting.clone();
        let pasted = self.app.run_on_main_thread(move || {
            // While an earlier paste waits to restore, the clipboard holds
            // that paste's text; what the user had is in `waiting`, unless
            // they have copied something since.
            let saved = match waiting.lock().unwrap().take() {
                Some(earlier) if !clipboard::changed_since(earlier.ours) => earlier.saved,
                _ => clipboard::save(),
            };
            let Some(ours) = clipboard::write_for_paste(&text) else {
                log::error!("Could not put the text on the clipboard to paste it");
                clipboard::restore(saved);
                return;
            };
            shortcut::send_keys(V, CGEventFlags::MaskCommand);
            *waiting.lock().unwrap() = Some(Waiting { saved, ours });
            thread::spawn(move || {
                thread::sleep(RESTORE_DELAY);
                if let Err(e) = app.run_on_main_thread(move || restore(&waiting, ours)) {
                    log::error!("Could not restore the clipboard: {e}");
                }
            });
        });
        if let Err(e) = pasted {
            log::error!("Could not insert the text: {e}");
        }
    }
}

/// Gives the clipboard back what it had before the paste `ours`, unless a
/// later paste has taken over the restore, or the user copied something
/// since, which must not be thrown away.
fn restore(waiting: &Mutex<Option<Waiting>>, ours: Change) {
    let Some(Waiting { saved, .. }) = waiting.lock().unwrap().take_if(|w| w.ours == ours) else {
        return;
    };
    if clipboard::changed_since(ours) {
        log::info!("The clipboard changed since the paste, so it is left as it is");
    } else {
        clipboard::restore(saved);
    }
}
