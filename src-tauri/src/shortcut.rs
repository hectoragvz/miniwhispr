//! Watches the keyboard for Ctrl + Space with a macOS event tap, which sees
//! every key press system-wide and can stop one from reaching the focused
//! app. Needs the Accessibility permission.

use std::ffi::c_void;
use std::ptr::{self, NonNull};
use std::thread;
use std::time::{Duration, Instant};

use objc2_application_services::{AXIsProcessTrustedWithOptions, kAXTrustedCheckOptionPrompt};
use objc2_core_foundation::{
    CFBoolean, CFDictionary, CFMachPort, CFRetained, CFRunLoop, kCFRunLoopCommonModes,
};
use objc2_core_graphics::{
    CGEvent, CGEventField, CGEventFlags, CGEventTapLocation, CGEventTapOptions,
    CGEventTapPlacement, CGEventTapProxy, CGEventType, CGKeyCode,
};

const SPACE: CGKeyCode = 49;
/// Tags the key taps Miniwhispr sends (a replayed Ctrl + Space, the Cmd + V
/// of an Insertion), so its own event tap lets them by.
const REPLAY_TAG: i64 = 0x6d77;

/// Starts watching the keyboard on a thread of its own. `on_shortcut` gets
/// every press and release of Ctrl + Space, with the moment it happened.
pub fn watch(on_shortcut: impl Fn(Shortcut, Instant) + Send + 'static) {
    thread::Builder::new()
        .name("shortcut".into())
        .spawn(move || run_tap(Box::new(on_shortcut)))
        .expect("could not start the shortcut thread");
}

/// Sends a Ctrl + Space tap to the focused app, for presses too short to be a
/// Dictation.
pub fn replay_tap() {
    send_keys(SPACE, CGEventFlags::MaskControl);
}

/// Sends a tap of `key` with exactly the `modifiers` held, whatever keys the
/// user is holding, to the focused app. Tagged as Miniwhispr's own, so its
/// event tap lets it by.
pub fn send_keys(key: CGKeyCode, modifiers: CGEventFlags) {
    for key_down in [true, false] {
        let Some(event) = CGEvent::new_keyboard_event(None, key, key_down) else {
            log::error!("Could not create a key event");
            return;
        };
        CGEvent::set_flags(Some(&event), modifiers);
        CGEvent::set_integer_value_field(
            Some(&event),
            CGEventField::EventSourceUserData,
            REPLAY_TAG,
        );
        CGEvent::post(CGEventTapLocation::HIDEventTap, Some(&event));
    }
}

/// What the tap callback needs. macOS hands it back to the callback as a raw
/// pointer.
struct Tap {
    filter: ShortcutFilter,
    on_shortcut: Box<dyn Fn(Shortcut, Instant) + Send>,
    port: Option<CFRetained<CFMachPort>>,
}

fn run_tap(on_shortcut: Box<dyn Fn(Shortcut, Instant) + Send>) {
    // Leaked on purpose: the tap lives as long as the app.
    let tap = Box::into_raw(Box::new(Tap {
        filter: ShortcutFilter::default(),
        on_shortcut,
        port: None,
    }));
    let events = [
        CGEventType::KeyDown,
        CGEventType::KeyUp,
        CGEventType::FlagsChanged,
    ]
    .iter()
    .fold(0, |mask, kind| mask | 1 << kind.0);

    let mut asked = false;
    let port = loop {
        // SAFETY: `on_event` matches what macOS expects of a tap callback,
        // and `tap` stays valid forever.
        let port = unsafe {
            CGEvent::tap_create(
                CGEventTapLocation::SessionEventTap,
                CGEventTapPlacement::HeadInsertEventTap,
                CGEventTapOptions::Default,
                events,
                Some(on_event),
                tap.cast(),
            )
        };
        if let Some(port) = port {
            break port;
        }
        // macOS refuses the tap until Miniwhispr has Accessibility. Ask once,
        // then keep trying so it starts working as soon as it is granted.
        if !asked {
            log::warn!("Ctrl + Space needs the Accessibility permission");
            ask_for_accessibility();
            asked = true;
        }
        thread::sleep(Duration::from_secs(1));
    };

    let Some(source) = CFMachPort::new_run_loop_source(None, Some(&port), 0) else {
        log::error!("Could not listen to the keyboard event tap");
        return;
    };
    // SAFETY: the run loop is not running yet, so the callback cannot be
    // using `tap` at the same time.
    unsafe { (*tap).port = Some(port) };
    let run_loop = CFRunLoop::current().expect("every thread has a run loop");
    // SAFETY: reading a constant macOS defines.
    run_loop.add_source(Some(&source), unsafe { kCFRunLoopCommonModes });
    CFRunLoop::run();
}

/// Called by macOS for every key event, before any app sees it. Returning
/// null swallows the event.
unsafe extern "C-unwind" fn on_event(
    _proxy: CGEventTapProxy,
    kind: CGEventType,
    event: NonNull<CGEvent>,
    tap: *mut c_void,
) -> *mut CGEvent {
    // SAFETY: `tap` is the pointer given to `tap_create`, only used on this
    // thread; `event` is valid for the duration of the call.
    let (tap, cg_event) = unsafe { (&mut *tap.cast::<Tap>(), event.as_ref()) };
    let pass = event.as_ptr();

    if kind == CGEventType::TapDisabledByTimeout || kind == CGEventType::TapDisabledByUserInput {
        // macOS turns a slow tap off; turn it straight back on.
        if let Some(port) = &tap.port {
            CGEvent::tap_enable(port, true);
        }
        return pass;
    }
    let field = |f| CGEvent::integer_value_field(Some(cg_event), f);
    if field(CGEventField::EventSourceUserData) == REPLAY_TAG {
        return pass;
    }
    let flags = CGEvent::flags(Some(cg_event));
    let is_space = field(CGEventField::KeyboardEventKeycode) == i64::from(SPACE);
    let key = match kind {
        CGEventType::KeyDown if is_space => {
            let modifiers = CGEventFlags::MaskControl
                | CGEventFlags::MaskCommand
                | CGEventFlags::MaskAlternate
                | CGEventFlags::MaskShift;
            Key::SpaceDown {
                ctrl_only: flags & modifiers == CGEventFlags::MaskControl,
            }
        }
        CGEventType::KeyUp if is_space => Key::SpaceUp,
        CGEventType::FlagsChanged => Key::ModifiersChanged {
            ctrl: flags.contains(CGEventFlags::MaskControl),
        },
        _ => return pass,
    };

    let decision = tap.filter.decide(key);
    if let Some(shortcut) = decision.report {
        (tap.on_shortcut)(shortcut, Instant::now());
    }
    if decision.swallow {
        ptr::null_mut()
    } else {
        pass
    }
}

/// Shows macOS's prompt to grant Miniwhispr Accessibility.
fn ask_for_accessibility() {
    // SAFETY: reading a constant macOS defines.
    let prompt = unsafe { kAXTrustedCheckOptionPrompt };
    let options = CFDictionary::from_slices(&[prompt], &[CFBoolean::new(true)]);
    // SAFETY: the options dictionary has the key and value types macOS expects.
    unsafe { AXIsProcessTrustedWithOptions(Some(options.as_opaque())) };
}

/// A keyboard event, reduced to what matters for Ctrl + Space.
#[derive(Clone, Copy, Debug)]
enum Key {
    /// Space went down (or auto-repeated). `ctrl_only` is true when Control
    /// is held and Command, Option and Shift are not.
    SpaceDown {
        ctrl_only: bool,
    },
    SpaceUp,
    /// A modifier key went down or up; `ctrl` is whether Control is now held.
    ModifiersChanged {
        ctrl: bool,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shortcut {
    Pressed,
    Released,
}

/// What to do with one keyboard event.
#[derive(Debug, PartialEq)]
struct Decision {
    /// Hide the event from the focused app.
    swallow: bool,
    /// Tell the Dictation core.
    report: Option<Shortcut>,
}

/// Decides, event by event, which keys belong to Ctrl + Space. Miniwhispr
/// swallows every Space event of the shortcut, so holding it never types
/// spaces, and passes everything else through untouched.
#[derive(Default)]
struct ShortcutFilter {
    /// Ctrl + Space is down and the core has been told.
    held: bool,
    /// The shortcut's Space is still physically down, even if Control was
    /// already let go, so its repeats and key up must not reach the app.
    space_down: bool,
}

impl ShortcutFilter {
    fn decide(&mut self, key: Key) -> Decision {
        match key {
            Key::SpaceDown { ctrl_only } if ctrl_only && !self.space_down => {
                self.held = true;
                self.space_down = true;
                Decision {
                    swallow: true,
                    report: Some(Shortcut::Pressed),
                }
            }
            Key::SpaceDown { .. } => Decision {
                swallow: self.space_down,
                report: None,
            },
            Key::SpaceUp => Decision {
                swallow: std::mem::take(&mut self.space_down),
                report: self.release(),
            },
            Key::ModifiersChanged { ctrl } => Decision {
                swallow: false,
                report: if ctrl { None } else { self.release() },
            },
        }
    }

    fn release(&mut self) -> Option<Shortcut> {
        std::mem::take(&mut self.held).then_some(Shortcut::Released)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PASS: Decision = Decision {
        swallow: false,
        report: None,
    };
    const SWALLOW: Decision = Decision {
        swallow: true,
        report: None,
    };

    fn report(shortcut: Shortcut) -> Decision {
        Decision {
            swallow: true,
            report: Some(shortcut),
        }
    }

    #[test]
    fn holding_ctrl_space_reports_one_press_and_swallows_the_repeats() {
        let mut filter = ShortcutFilter::default();

        let decisions: Vec<_> = [
            Key::ModifiersChanged { ctrl: true },
            Key::SpaceDown { ctrl_only: true },
            Key::SpaceDown { ctrl_only: true },
            Key::SpaceDown { ctrl_only: true },
        ]
        .into_iter()
        .map(|key| filter.decide(key))
        .collect();

        assert_eq!(
            decisions,
            vec![PASS, report(Shortcut::Pressed), SWALLOW, SWALLOW]
        );
    }

    fn decide_all(keys: &[Key]) -> Vec<Decision> {
        let mut filter = ShortcutFilter::default();
        keys.iter().map(|key| filter.decide(*key)).collect()
    }

    #[test]
    fn releasing_space_reports_the_release_and_swallows_the_key_up() {
        let decisions = decide_all(&[
            Key::SpaceDown { ctrl_only: true },
            Key::SpaceUp,
            Key::ModifiersChanged { ctrl: false },
        ]);

        assert_eq!(
            decisions,
            vec![report(Shortcut::Pressed), report(Shortcut::Released), PASS]
        );
    }

    #[test]
    fn releasing_ctrl_first_reports_the_release_and_still_swallows_space() {
        let decisions = decide_all(&[
            Key::SpaceDown { ctrl_only: true },
            Key::ModifiersChanged { ctrl: false },
            Key::SpaceDown { ctrl_only: false },
            Key::SpaceUp,
            Key::SpaceDown { ctrl_only: false },
        ]);

        assert_eq!(
            decisions,
            vec![
                report(Shortcut::Pressed),
                Decision {
                    swallow: false,
                    report: Some(Shortcut::Released)
                },
                SWALLOW,
                SWALLOW,
                PASS,
            ]
        );
    }

    #[test]
    fn other_spaces_reach_the_app_untouched() {
        let decisions = decide_all(&[
            Key::SpaceDown { ctrl_only: false },
            Key::SpaceUp,
            Key::ModifiersChanged { ctrl: true },
            Key::ModifiersChanged { ctrl: false },
        ]);

        assert_eq!(decisions, vec![PASS, PASS, PASS, PASS]);
    }
}
