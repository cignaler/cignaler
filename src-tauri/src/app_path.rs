//! Resolving an executable path that will still exist on the next launch.
//!
//! Two macOS/Linux mechanisms run an app from a location that evaporates when
//! it quits, and `std::env::current_exe()` reports that location happily:
//!
//! - **App Translocation.** macOS runs a quarantined app that the Finder has
//!   not moved from a read-only image under
//!   `/private/var/folders/.../AppTranslocation/<uuid>/d/`. This is the normal
//!   path for an app launched straight out of a downloaded DMG.
//! - **A mounted image.** Running the app from the DMG window itself, or from
//!   an AppImage, puts it on a read-only mount the user will unmount.
//!
//! Neither matters for code that only reads its own resources. It matters a
//! great deal for the Chrome native messaging manifest, which persists an
//! absolute path to the host binary and is read by Chrome long after we are
//! gone — a manifest written from a translocated path points at nothing.

use std::path::{Path, PathBuf};

/// Why a path must not be persisted.
#[derive(Debug, PartialEq, Eq)]
pub enum NotDurable {
    /// Running under App Translocation, and the original location could not
    /// be recovered.
    Translocated,
    /// The real location is a read-only mount — the app is running from a DMG
    /// or other image the user will eject.
    ReadOnlyMount(PathBuf),
}

impl std::fmt::Display for NotDurable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotDurable::Translocated => write!(
                f,
                "the app is running under macOS App Translocation and its real \
                 location could not be recovered; move it to /Applications with \
                 the Finder and reopen it"
            ),
            NotDurable::ReadOnlyMount(p) => write!(
                f,
                "the app is running from the read-only mount holding {}; drag \
                 it to /Applications and open it from there",
                p.display()
            ),
        }
    }
}

/// Map `path` to a location that outlives this process, or explain why there
/// isn't one.
///
/// Translocation is undone first: the translocated image is itself read-only,
/// so checking the mount before resolving would report every translocated
/// launch as `ReadOnlyMount` and hide the real cause.
pub fn durable_path(path: &Path) -> Result<PathBuf, NotDurable> {
    let resolved = untranslocated(path)?;

    if is_read_only_mount(&resolved) {
        return Err(NotDurable::ReadOnlyMount(resolved));
    }

    Ok(resolved)
}

/// `durable_path` applied to the running executable. `Err(None)` means the
/// executable path itself could not be determined.
pub fn durable_current_exe() -> Result<PathBuf, Option<NotDurable>> {
    let exe = std::env::current_exe().map_err(|_| None)?;
    durable_path(&exe).map_err(Some)
}

/// True when `path` sits on a mount that cannot be written to. `statvfs` is
/// the portable spelling — macOS and Linux both have it, unlike `statfs`,
/// whose struct differs between them.
#[cfg(unix)]
fn is_read_only_mount(path: &Path) -> bool {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let Ok(c_path) = CString::new(path.as_os_str().as_bytes()) else {
        return false;
    };

    // SAFETY: c_path is a valid NUL-terminated string that outlives the call,
    // and statvfs only writes into the buffer we hand it.
    let mut buf: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(c_path.as_ptr(), &mut buf) } != 0 {
        return false;
    }

    buf.f_flag & libc::ST_RDONLY != 0
}

#[cfg(not(unix))]
fn is_read_only_mount(_path: &Path) -> bool {
    false
}

/// A translocated path always contains this component. Used only as a
/// backstop: if the Security framework call fails outright we still refuse to
/// persist an obviously translocated path rather than trusting it.
#[cfg(target_os = "macos")]
const TRANSLOCATION_MARKER: &str = "/AppTranslocation/";

#[cfg(target_os = "macos")]
fn looks_translocated(path: &Path) -> bool {
    path.to_string_lossy().contains(TRANSLOCATION_MARKER)
}

#[cfg(target_os = "macos")]
fn untranslocated(path: &Path) -> Result<PathBuf, NotDurable> {
    use core_foundation::base::{Boolean, TCFType};
    use core_foundation::error::CFErrorRef;
    use core_foundation::url::{CFURL, CFURLRef};

    // SecTranslocate.h, available since 10.12. Both functions accept a NULL
    // error pointer, which is what we pass — there is nothing to do with a
    // CFError here beyond what the return value already tells us, and not
    // taking one means there is nothing to release.
    #[link(name = "Security", kind = "framework")]
    extern "C" {
        fn SecTranslocateIsTranslocatedURL(
            path: CFURLRef,
            is_translocated: *mut Boolean,
            error: *mut CFErrorRef,
        ) -> Boolean;

        fn SecTranslocateCreateOriginalPathForURL(
            translocated_path: CFURLRef,
            error: *mut CFErrorRef,
        ) -> CFURLRef;
    }

    let Some(url) = CFURL::from_path(path, false) else {
        return fallback(path);
    };

    let mut translocated: Boolean = 0;
    // SAFETY: `url` outlives the call and `translocated` is a valid out-param.
    let queried = unsafe {
        SecTranslocateIsTranslocatedURL(
            url.as_concrete_TypeRef(),
            &mut translocated,
            std::ptr::null_mut(),
        )
    };

    if queried == 0 {
        return fallback(path);
    }
    if translocated == 0 {
        return Ok(path.to_path_buf());
    }

    // SAFETY: same invariants; the result is a +1 reference (Create rule) or
    // NULL, and wrap_under_create_rule takes ownership of the former.
    let original =
        unsafe { SecTranslocateCreateOriginalPathForURL(url.as_concrete_TypeRef(), std::ptr::null_mut()) };
    if original.is_null() {
        return Err(NotDurable::Translocated);
    }

    unsafe { CFURL::wrap_under_create_rule(original) }
        .to_path()
        .ok_or(NotDurable::Translocated)
}

/// Used when the Security framework cannot answer: trust the path shape
/// instead, and refuse rather than guess.
#[cfg(target_os = "macos")]
fn fallback(path: &Path) -> Result<PathBuf, NotDurable> {
    if looks_translocated(path) {
        Err(NotDurable::Translocated)
    } else {
        Ok(path.to_path_buf())
    }
}

/// Translocation is a macOS invention; everywhere else the path is the path.
#[cfg(not(target_os = "macos"))]
fn untranslocated(path: &Path) -> Result<PathBuf, NotDurable> {
    Ok(path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinary_path_is_durable() {
        let dir = std::env::temp_dir();
        assert_eq!(durable_path(&dir).unwrap(), dir);
    }

    #[test]
    fn temp_dir_is_not_a_read_only_mount() {
        assert!(!is_read_only_mount(&std::env::temp_dir()));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn translocated_paths_are_recognised() {
        let translocated = Path::new(
            "/private/var/folders/s7/x/T/AppTranslocation/0FB7803B/d/cignaler.app/Contents/MacOS/cignaler",
        );
        assert!(looks_translocated(translocated));
        assert!(!looks_translocated(Path::new(
            "/Applications/cignaler.app/Contents/MacOS/cignaler"
        )));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn a_translocated_path_that_no_longer_exists_is_refused() {
        // The image is long gone, so the Security framework cannot map it
        // back. Refusing is the whole point: the alternative is persisting a
        // path that resolves to nothing.
        let stale = Path::new(
            "/private/var/folders/s7/x/T/AppTranslocation/DEADBEEF/d/cignaler.app/Contents/MacOS/cignaler",
        );
        assert_eq!(durable_path(stale), Err(NotDurable::Translocated));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn the_read_only_system_volume_is_refused() {
        // /usr/bin lives on the Signed System Volume, which is read-only on
        // every macOS since Catalina.
        let path = Path::new("/usr/bin/true");
        match durable_path(path) {
            Err(NotDurable::ReadOnlyMount(_)) => {}
            other => panic!("expected ReadOnlyMount, got {:?}", other),
        }
    }
}
