//! The soft start and stop sounds, played with macOS's NSSound.

use objc2::AnyThread;
use objc2::rc::Retained;
use objc2_app_kit::NSSound;
use objc2_foundation::NSData;

use crate::dictation::Sound;

pub struct Sounds {
    start: Option<Retained<NSSound>>,
    stop: Option<Retained<NSSound>>,
}

impl Sounds {
    pub fn load() -> Self {
        Self {
            start: load(include_bytes!("../sounds/start.wav")),
            stop: load(include_bytes!("../sounds/stop.wav")),
        }
    }

    pub fn play(&self, sound: Sound) {
        let sound = match sound {
            Sound::Start => &self.start,
            Sound::Stop => &self.stop,
        };
        if let Some(sound) = sound {
            // Restart it if the previous play has not finished.
            sound.stop();
            sound.play();
        }
    }
}

fn load(wav: &[u8]) -> Option<Retained<NSSound>> {
    let sound = NSSound::initWithData(NSSound::alloc(), &NSData::with_bytes(wav));
    if sound.is_none() {
        log::error!("Could not load a Dictation sound");
    }
    sound
}
