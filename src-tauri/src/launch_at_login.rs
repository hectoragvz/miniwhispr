use std::fs;
use std::path::Path;

// SAFETY (all `unsafe` below): objc2 marks every generated Apple binding as
// unsafe; these SMAppService calls take no pointers and have no preconditions.
use objc2_service_management::{SMAppService, SMAppServiceStatus};

/// Whether macOS will open Miniwhispr at login. Reads the system state, so it
/// reflects changes made in System Settings too.
pub fn is_enabled() -> bool {
    unsafe { SMAppService::mainAppService().status() == SMAppServiceStatus::Enabled }
}

/// Adds or removes Miniwhispr as a login item. Only works for the bundled app.
pub fn set_enabled(on: bool) -> Result<(), String> {
    let service = unsafe { SMAppService::mainAppService() };
    let result = unsafe {
        if on {
            service.registerAndReturnError()
        } else {
            service.unregisterAndReturnError()
        }
    };
    result.map_err(|e| e.to_string())?;
    if on && unsafe { service.status() } == SMAppServiceStatus::RequiresApproval {
        unsafe { SMAppService::openSystemSettingsLoginItems() };
    }
    Ok(())
}

/// Turns Launch at login on the first time Miniwhispr runs, so it is on by
/// default but a later choice to turn it off is respected. `marker` records
/// that the default was applied.
pub fn apply_default(
    marker: &Path,
    enable: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    if marker.exists() {
        return Ok(());
    }
    enable()?;
    if let Some(dir) = marker.parent() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    fs::write(marker, "").map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn first_run_enables_and_later_runs_do_not() {
        let dir = tempfile::tempdir().unwrap();
        let marker = dir.path().join("nested/launch-at-login-defaulted");
        let calls = Cell::new(0);
        let enable = || {
            calls.set(calls.get() + 1);
            Ok(())
        };

        apply_default(&marker, enable).unwrap();
        apply_default(&marker, enable).unwrap();

        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn a_failed_enable_is_retried_on_the_next_run() {
        let dir = tempfile::tempdir().unwrap();
        let marker = dir.path().join("launch-at-login-defaulted");
        let calls = Cell::new(0);

        let failed = apply_default(&marker, || Err("not a bundled app".to_string()));
        apply_default(&marker, || {
            calls.set(calls.get() + 1);
            Ok(())
        })
        .unwrap();

        assert_eq!(failed, Err("not a bundled app".to_string()));
        assert_eq!(calls.get(), 1);
    }
}
