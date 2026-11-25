//! Runs the Dictation core on a thread of its own, wired to the real
//! keyboard, microphone, Groq, Toast, sounds and menu. The core sees one
//! input at a time, and is woken whenever it has a deadline.

use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::thread;
use std::time::Instant;

use tauri::menu::MenuItem;
use tauri::{AppHandle, Manager, Wry};

use crate::clipboard;
use crate::dictation::{Cleaner, Clock, Dictation, Feedback, Release, Sound, Toast};
use crate::groq::GroqTranscriber;
use crate::microphone::Microphone;
use crate::paste::PasteInserter;
use crate::shortcut::{self, Shortcut};
use crate::sounds::Sounds;
use crate::toast;

enum Input {
    Shortcut(Shortcut, Instant),
    Level(f32),
    CopyLastDictation,
}

/// Lets the menu reach the Dictation thread.
struct Inputs(Sender<Input>);

/// Starts listening for Ctrl + Space. `copy_item` is the "Copy last
/// dictation" menu item, enabled once there is a Last Dictation.
pub fn start(app: &AppHandle, copy_item: MenuItem<Wry>) {
    let (inputs, received) = mpsc::channel();
    app.manage(Inputs(inputs.clone()));
    let levels = inputs.clone();
    let app = app.clone();
    let main_thread = app.clone();
    thread::Builder::new()
        .name("dictation".into())
        .spawn(move || {
            // Built on this thread: the microphone stream and the sounds must
            // stay on the thread that created them.
            let mut core = Dictation::new(
                Box::new(Microphone::new(move |level| {
                    let _ = levels.send(Input::Level(level));
                })),
                Box::new(GroqTranscriber::new()),
                Box::new(NotYet),
                Box::new(PasteInserter::new(main_thread.clone())),
                Box::new(AppFeedback {
                    app,
                    sounds: Sounds::load(),
                }),
                Box::new(SystemClock),
            );
            let mut copy_enabled = false;
            loop {
                let input = match core.next_deadline() {
                    Some(deadline) => {
                        received.recv_timeout(deadline.saturating_duration_since(Instant::now()))
                    }
                    None => received.recv().map_err(|_| RecvTimeoutError::Disconnected),
                };
                match input {
                    Ok(Input::Shortcut(Shortcut::Pressed, at)) => core.pressed(at),
                    Ok(Input::Shortcut(Shortcut::Released, at)) => {
                        if core.released(at) == Release::PassThrough {
                            shortcut::replay_tap();
                        }
                    }
                    Ok(Input::Level(level)) => core.level(level),
                    Ok(Input::CopyLastDictation) => {
                        if let Some(text) = core.last_dictation() {
                            let text = text.to_string();
                            if let Err(e) =
                                main_thread.run_on_main_thread(move || clipboard::copy(&text))
                            {
                                log::error!("Could not copy the Last Dictation: {e}");
                            }
                        }
                    }
                    Err(RecvTimeoutError::Timeout) => core.tick(),
                    Err(RecvTimeoutError::Disconnected) => return,
                }
                // A Last Dictation, once there, is only ever replaced.
                if !copy_enabled && core.last_dictation().is_some() {
                    copy_enabled = true;
                    if let Err(e) = copy_item.set_enabled(true) {
                        log::error!("Could not enable Copy last dictation: {e}");
                    }
                }
            }
        })
        .expect("could not start the Dictation thread");

    shortcut::watch(move |shortcut, at| {
        let _ = inputs.send(Input::Shortcut(shortcut, at));
    });
}

/// Copies the Last Dictation to the clipboard. Waits for a Dictation that
/// is still processing, so it copies that one.
pub fn copy_last_dictation(app: &AppHandle) {
    let _ = app.state::<Inputs>().0.send(Input::CopyLastDictation);
}

struct AppFeedback {
    app: AppHandle,
    sounds: Sounds,
}

impl Feedback for AppFeedback {
    fn show(&mut self, toast: Toast) {
        toast::show(&self.app, toast);
    }

    fn play(&mut self, sound: Sound) {
        self.sounds.play(sound);
    }
}

struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
}

/// Stands in for Cleanup until it exists: the Transcript is the final text.
struct NotYet;

impl Cleaner for NotYet {
    fn clean(&mut self, transcript: &str) -> Result<String, String> {
        Ok(transcript.to_string())
    }
}
