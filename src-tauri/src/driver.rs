//! Runs the Dictation core on a thread of its own, wired to the real
//! keyboard, microphone, Toast and sounds. The core sees one input at a
//! time, and is woken whenever it has a deadline.

use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::Instant;

use tauri::AppHandle;

use crate::dictation::{
    Audio, Cleaner, Clock, Dictation, Feedback, Inserter, Release, SAMPLE_RATE, Sound, Toast,
    Transcriber,
};
use crate::microphone::Microphone;
use crate::shortcut::{self, Shortcut};
use crate::sounds::Sounds;
use crate::toast;

enum Input {
    Shortcut(Shortcut, Instant),
    Level(f32),
}

/// Starts listening for Ctrl + Space.
pub fn start(app: &AppHandle) {
    let (inputs, received) = mpsc::channel();
    let levels = inputs.clone();
    let app = app.clone();
    thread::Builder::new()
        .name("dictation".into())
        .spawn(move || {
            // Built on this thread: the microphone stream and the sounds must
            // stay on the thread that created them.
            let mut core = Dictation::new(
                Box::new(Microphone::new(move |level| {
                    let _ = levels.send(Input::Level(level));
                })),
                Box::new(NotYet),
                Box::new(NotYet),
                Box::new(NotYet),
                Box::new(AppFeedback {
                    app,
                    sounds: Sounds::load(),
                }),
                Box::new(SystemClock),
            );
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
                    Err(RecvTimeoutError::Timeout) => core.tick(),
                    Err(RecvTimeoutError::Disconnected) => return,
                }
            }
        })
        .expect("could not start the Dictation thread");

    shortcut::watch(move |shortcut, at| {
        let _ = inputs.send(Input::Shortcut(shortcut, at));
    });
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

/// Stands in for transcription, Cleanup and Insertion until they exist:
/// nothing is transcribed or inserted.
struct NotYet;

impl Transcriber for NotYet {
    fn transcribe(&mut self, audio: &Audio) -> Result<String, String> {
        let seconds = audio.samples.len() as f32 / SAMPLE_RATE as f32;
        log::info!("Captured {seconds:.1} s of audio");
        Ok(String::new())
    }
}

impl Cleaner for NotYet {
    fn clean(&mut self, transcript: &str) -> Result<String, String> {
        Ok(transcript.to_string())
    }
}

impl Inserter for NotYet {
    fn insert(&mut self, _text: &str) {}
}
