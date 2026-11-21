//! The Dictation core: the rules of one Dictation, from pressing Ctrl + Space
//! to the text being ready. It knows nothing about macOS, Groq or the UI; it
//! talks to the outside world only through the traits below, so tests can
//! drive it with fakes and a clock they control.

use std::time::{Duration, Instant};

use serde::Serialize;

/// A press shorter than this is a normal Ctrl + Space tap for the focused app.
pub const HOLD_THRESHOLD: Duration = Duration::from_millis(250);
/// A Dictation that reaches this length ends as if the keys were released.
pub const MAX_DICTATION: Duration = Duration::from_secs(10 * 60);

/// Captured speech: 16 kHz mono, 16-bit samples.
pub struct Audio {
    pub samples: Vec<i16>,
}

pub const SAMPLE_RATE: u32 = 16_000;

/// Captures audio from the microphone. Input levels reach the core through
/// [`Dictation::level`].
pub trait Recorder {
    fn start(&mut self) -> Result<(), String>;
    fn stop(&mut self) -> Audio;
}

/// Turns audio into a Transcript.
pub trait Transcriber {
    fn transcribe(&mut self, audio: &Audio) -> Result<String, String>;
}

/// Turns a Transcript into the final text.
pub trait Cleaner {
    fn clean(&mut self, transcript: &str) -> Result<String, String>;
}

/// Puts the final text into the focused app.
pub trait Inserter {
    fn insert(&mut self, text: &str);
}

/// What the user sees and hears.
pub trait Feedback {
    fn show(&mut self, toast: Toast);
    fn play(&mut self, sound: Sound);
}

/// The current time. A trait so tests can move time forward by hand.
pub trait Clock {
    fn now(&self) -> Instant;
}

/// What the Toast shows. Never any text.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Toast {
    Hidden,
    /// `level` is the microphone input level, from 0 (silence) to 1 (loud).
    Listening {
        level: f32,
    },
    Processing,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Sound {
    Start,
    Stop,
}

/// What should happen to a Ctrl + Space press once it is released.
#[must_use]
#[derive(Debug, PartialEq)]
pub enum Release {
    /// It was a quick tap: send Ctrl + Space on to the focused app.
    PassThrough,
    /// It was Miniwhispr's: the focused app never sees it.
    Swallow,
}

enum State {
    Idle,
    /// Keys are down, but not yet long enough to be a Dictation.
    Pending {
        pressed_at: Instant,
    },
    Listening {
        started_at: Instant,
    },
}

pub struct Dictation {
    state: State,
    /// When the last Dictation finished processing. Presses made before this
    /// happened while it was busy, and are ignored.
    ready_at: Instant,
    recorder: Box<dyn Recorder>,
    transcriber: Box<dyn Transcriber>,
    cleaner: Box<dyn Cleaner>,
    inserter: Box<dyn Inserter>,
    feedback: Box<dyn Feedback>,
    clock: Box<dyn Clock>,
}

impl Dictation {
    pub fn new(
        recorder: Box<dyn Recorder>,
        transcriber: Box<dyn Transcriber>,
        cleaner: Box<dyn Cleaner>,
        inserter: Box<dyn Inserter>,
        feedback: Box<dyn Feedback>,
        clock: Box<dyn Clock>,
    ) -> Self {
        Self {
            state: State::Idle,
            ready_at: clock.now(),
            recorder,
            transcriber,
            cleaner,
            inserter,
            feedback,
            clock,
        }
    }

    /// Ctrl + Space went down at `at`.
    pub fn pressed(&mut self, at: Instant) {
        self.advance(at);
        if let State::Idle = self.state
            && at >= self.ready_at
        {
            self.state = State::Pending { pressed_at: at };
        }
    }

    /// The microphone input level, from 0 (silence) to 1 (loud).
    pub fn level(&mut self, level: f32) {
        if let State::Listening { .. } = self.state {
            self.feedback.show(Toast::Listening { level });
        }
    }

    /// The next moment the core needs [`Dictation::tick`] called, if any.
    pub fn next_deadline(&self) -> Option<Instant> {
        match self.state {
            State::Pending { pressed_at } => Some(pressed_at + HOLD_THRESHOLD),
            State::Listening { started_at } => Some(started_at + MAX_DICTATION),
            State::Idle => None,
        }
    }

    /// Lets the core act on time passing.
    pub fn tick(&mut self) {
        self.advance(self.clock.now());
    }

    /// Ctrl + Space came up at `at`.
    pub fn released(&mut self, at: Instant) -> Release {
        self.advance(at);
        match self.state {
            State::Pending { .. } => {
                self.state = State::Idle;
                Release::PassThrough
            }
            State::Listening { .. } => {
                self.finish();
                Release::Swallow
            }
            State::Idle => Release::Swallow,
        }
    }

    /// Catches up with everything due by `now`. Key events carry the time
    /// they happened, so a press is judged by when it was made, not by when
    /// the core got to it.
    fn advance(&mut self, now: Instant) {
        while let Some(deadline) = self.next_deadline()
            && now >= deadline
        {
            match self.state {
                State::Pending { .. } => self.start_listening(deadline),
                State::Listening { .. } => self.finish(),
                State::Idle => unreachable!("Idle has no deadline"),
            }
        }
    }

    fn start_listening(&mut self, started_at: Instant) {
        if let Err(e) = self.recorder.start() {
            log::error!("Could not start recording: {e}");
            self.state = State::Idle;
            return;
        }
        self.state = State::Listening { started_at };
        self.feedback.play(Sound::Start);
        self.feedback.show(Toast::Listening { level: 0.0 });
    }

    /// Ends listening and turns the audio into inserted text.
    fn finish(&mut self) {
        let audio = self.recorder.stop();
        self.feedback.play(Sound::Stop);
        self.feedback.show(Toast::Processing);
        let text = self
            .transcriber
            .transcribe(&audio)
            .and_then(|transcript| self.cleaner.clean(&transcript));
        match text {
            Ok(text) => self.inserter.insert(&text),
            Err(e) => log::error!("Dictation failed: {e}"),
        }
        self.feedback.show(Toast::Hidden);
        self.state = State::Idle;
        self.ready_at = self.clock.now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    /// Everything the user can notice, in the order it happened.
    #[derive(Debug, PartialEq)]
    enum Seen {
        Recording,
        StoppedRecording,
        Toast(Toast),
        Sound(Sound),
    }

    type Log = Rc<RefCell<Vec<Seen>>>;

    struct FakeRecorder(Log);
    impl Recorder for FakeRecorder {
        fn start(&mut self) -> Result<(), String> {
            self.0.borrow_mut().push(Seen::Recording);
            Ok(())
        }
        fn stop(&mut self) -> Audio {
            self.0.borrow_mut().push(Seen::StoppedRecording);
            Audio { samples: vec![] }
        }
    }

    struct FakeFeedback(Log);
    impl Feedback for FakeFeedback {
        fn show(&mut self, toast: Toast) {
            self.0.borrow_mut().push(Seen::Toast(toast));
        }
        fn play(&mut self, sound: Sound) {
            self.0.borrow_mut().push(Seen::Sound(sound));
        }
    }

    #[derive(Clone)]
    struct FakeClock(Rc<Cell<Instant>>);
    impl Clock for FakeClock {
        fn now(&self) -> Instant {
            self.0.get()
        }
    }
    impl FakeClock {
        fn advance(&self, by: Duration) {
            self.0.set(self.0.get() + by);
        }
    }

    /// Transcription takes `delay` of (fake) time, like a network call.
    struct FakeTranscriber {
        clock: FakeClock,
        delay: Duration,
    }
    impl Transcriber for FakeTranscriber {
        fn transcribe(&mut self, _audio: &Audio) -> Result<String, String> {
            self.clock.advance(self.delay);
            Ok("hello".to_string())
        }
    }

    struct EchoCleaner;
    impl Cleaner for EchoCleaner {
        fn clean(&mut self, transcript: &str) -> Result<String, String> {
            Ok(transcript.to_string())
        }
    }

    struct NoInserter;
    impl Inserter for NoInserter {
        fn insert(&mut self, _text: &str) {}
    }

    /// A Dictation core wired to fakes, plus the user's view of it.
    struct Harness {
        core: Dictation,
        clock: FakeClock,
        log: Log,
    }

    const TRANSCRIPTION_TIME: Duration = Duration::from_secs(1);

    impl Harness {
        fn new() -> Self {
            let log = Log::default();
            let clock = FakeClock(Rc::new(Cell::new(Instant::now())));
            let core = Dictation::new(
                Box::new(FakeRecorder(log.clone())),
                Box::new(FakeTranscriber {
                    clock: clock.clone(),
                    delay: TRANSCRIPTION_TIME,
                }),
                Box::new(EchoCleaner),
                Box::new(NoInserter),
                Box::new(FakeFeedback(log.clone())),
                Box::new(clock.clone()),
            );
            Self { core, clock, log }
        }

        fn press(&mut self) {
            self.core.pressed(self.clock.now());
        }

        fn release(&mut self) -> Release {
            self.core.released(self.clock.now())
        }

        /// Lets `ms` pass, waking the core at each deadline it asks for, as
        /// the real driver does.
        fn wait(&mut self, ms: u64) {
            let until = self.clock.now() + Duration::from_millis(ms);
            while let Some(deadline) = self.core.next_deadline().filter(|d| *d <= until) {
                self.clock.0.set(deadline.max(self.clock.now()));
                self.core.tick();
            }
            self.clock.0.set(until);
        }

        fn seen(&self) -> Vec<Seen> {
            self.log.take()
        }
    }

    #[test]
    fn a_tap_under_250_ms_passes_through_with_no_toast_or_sound() {
        let mut h = Harness::new();

        h.press();
        h.wait(249);
        let release = h.release();

        assert_eq!(release, Release::PassThrough);
        assert_eq!(h.seen(), vec![]);
    }

    #[test]
    fn a_hold_reaching_250_ms_starts_listening_with_the_start_sound() {
        let mut h = Harness::new();

        h.press();
        h.wait(250);

        assert_eq!(
            h.seen(),
            vec![
                Seen::Recording,
                Seen::Sound(Sound::Start),
                Seen::Toast(Toast::Listening { level: 0.0 }),
            ]
        );
    }

    #[test]
    fn releasing_plays_the_stop_sound_shows_processing_then_hides() {
        let mut h = Harness::new();
        h.press();
        h.wait(1_000);
        h.seen();

        let release = h.release();

        assert_eq!(release, Release::Swallow);
        assert_eq!(
            h.seen(),
            vec![
                Seen::StoppedRecording,
                Seen::Sound(Sound::Stop),
                Seen::Toast(Toast::Processing),
                Seen::Toast(Toast::Hidden),
            ]
        );
    }

    #[test]
    fn a_press_while_processing_is_ignored_and_not_passed_through() {
        let mut h = Harness::new();
        h.press();
        h.wait(1_000);
        let released_at = h.clock.now();
        let _ = h.release();
        h.seen();

        // The core was busy transcribing for a second, so these key events,
        // made during that second, only reach it now.
        h.core.pressed(released_at + Duration::from_millis(400));
        let release = h.core.released(released_at + Duration::from_millis(900));
        h.wait(1_000);

        assert_eq!(release, Release::Swallow);
        assert_eq!(h.seen(), vec![]);
    }

    #[test]
    fn a_hold_after_processing_starts_a_new_dictation() {
        let mut h = Harness::new();
        h.press();
        h.wait(1_000);
        let _ = h.release();
        h.seen();

        h.press();
        h.wait(250);

        assert_eq!(
            h.seen(),
            vec![
                Seen::Recording,
                Seen::Sound(Sound::Start),
                Seen::Toast(Toast::Listening { level: 0.0 }),
            ]
        );
    }

    #[test]
    fn ten_minutes_of_listening_ends_the_dictation_as_if_released() {
        let mut h = Harness::new();
        h.press();
        h.wait(250);
        h.seen();

        h.wait(10 * 60 * 1_000);

        assert_eq!(
            h.seen(),
            vec![
                Seen::StoppedRecording,
                Seen::Sound(Sound::Stop),
                Seen::Toast(Toast::Processing),
                Seen::Toast(Toast::Hidden),
            ]
        );
        assert_eq!(h.release(), Release::Swallow);
        assert_eq!(h.seen(), vec![]);
    }

    #[test]
    fn the_listening_toast_follows_the_microphone_level() {
        let mut h = Harness::new();
        h.core.level(0.5);
        h.press();
        h.wait(250);
        h.seen();

        h.core.level(0.3);
        h.core.level(0.8);
        let _ = h.release();
        h.core.level(0.6);

        assert_eq!(
            h.seen(),
            vec![
                Seen::Toast(Toast::Listening { level: 0.3 }),
                Seen::Toast(Toast::Listening { level: 0.8 }),
                Seen::StoppedRecording,
                Seen::Sound(Sound::Stop),
                Seen::Toast(Toast::Processing),
                Seen::Toast(Toast::Hidden),
            ]
        );
    }
}
