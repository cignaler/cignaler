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

/// Every translocated path contains this component.
#[cfg(target_os = "macos")]
const TRANSLOCATION_MARKER: &str = "/AppTranslocation/";

#[cfg(target_os = "macos")]
fn looks_translocated(path: &Path) -> bool {
    path.to_string_lossy().contains(TRANSLOCATION_MARKER)
}

#[cfg(target_os = "macos")]
fn untranslocated(path: &Path) -> Result<PathBuf, NotDurable> {
    // Either signal is enough to start looking for the original. The marker is
    // checked first and on its own because SecTranslocateIsTranslocatedURL has
    // been observed answering "no" for a process asking about its own
    // translocated path -- see sec_translocate_original below.
    if !looks_translocated(path) && !sec_is_translocated(path).unwrap_or(false) {
        return Ok(path.to_path_buf());
    }

    // The documented API first, the mount underneath it second. Neither is a
    // superset of the other: SecTranslocate knows about translocations this
    // process cannot see the mount for, and the mount answers when
    // SecTranslocate declines.
    if let Some(original) = sec_translocate_original(path) {
        return Ok(original);
    }
    if let Some(original) = mount_source_original(path) {
        return Ok(original);
    }

    Err(NotDurable::Translocated)
}

// SecTranslocate.h, available since 10.12. Both functions accept a NULL error
// pointer, which is what we pass -- there is nothing to do with a CFError here
// beyond what the return value already says, and not taking one means there is
// nothing to release.
#[cfg(target_os = "macos")]
#[link(name = "Security", kind = "framework")]
extern "C" {
    fn SecTranslocateIsTranslocatedURL(
        path: core_foundation::url::CFURLRef,
        is_translocated: *mut core_foundation::base::Boolean,
        error: *mut core_foundation::error::CFErrorRef,
    ) -> core_foundation::base::Boolean;

    fn SecTranslocateCreateOriginalPathForURL(
        translocated_path: core_foundation::url::CFURLRef,
        error: *mut core_foundation::error::CFErrorRef,
    ) -> core_foundation::url::CFURLRef;
}

/// `None` when the framework declined to answer at all.
#[cfg(target_os = "macos")]
fn sec_is_translocated(path: &Path) -> Option<bool> {
    use core_foundation::base::{Boolean, TCFType};
    use core_foundation::url::CFURL;

    let url = CFURL::from_path(path, false)?;
    let mut translocated: Boolean = 0;

    // SAFETY: `url` outlives the call and `translocated` is a valid out-param.
    let answered = unsafe {
        SecTranslocateIsTranslocatedURL(
            url.as_concrete_TypeRef(),
            &mut translocated,
            std::ptr::null_mut(),
        )
    };

    (answered != 0).then(|| translocated != 0)
}

/// Ask the Security framework for the original location.
///
/// This is the right call and it works when one process asks about another's
/// translocated path. It did not answer when the translocated process asked
/// about itself, which is exactly our case -- hence mount_source_original.
#[cfg(target_os = "macos")]
fn sec_translocate_original(path: &Path) -> Option<PathBuf> {
    use core_foundation::base::TCFType;
    use core_foundation::url::CFURL;

    let url = CFURL::from_path(path, false)?;

    // SAFETY: `url` outlives the call; the result is a +1 reference (Create
    // rule) or NULL, and wrap_under_create_rule takes ownership of the former.
    let original = unsafe {
        SecTranslocateCreateOriginalPathForURL(url.as_concrete_TypeRef(), std::ptr::null_mut())
    };
    if original.is_null() {
        return None;
    }

    unsafe { CFURL::wrap_under_create_rule(original) }.to_path()
}

/// Recover the original bundle from the translocation mount itself.
///
/// A translocated app is a nullfs mount whose *source* is the real bundle:
///
///   /Applications/cignaler.app on /private/var/.../AppTranslocation/<uuid>
///       (nullfs, local, nodev, nosuid, read-only, nobrowse)
///
/// statfs reports both halves -- f_mntfromname is the source, f_mntonname the
/// mount point -- and answers for a process asking about its own path, which
/// is the case SecTranslocate leaves unserved.
#[cfg(target_os = "macos")]
fn mount_source_original(path: &Path) -> Option<PathBuf> {
    let info = statfs(path)?;
    let mount_point = c_chars_to_path(&info.f_mntonname)?;
    let mount_source = c_chars_to_path(&info.f_mntfromname)?;

    let original = graft_onto_source(&mount_point, &mount_source, path)?;

    // The source is only meaningful if it is really there. A nullfs mount
    // outlives a deleted source, and persisting a path to a bundle someone has
    // already thrown away helps nobody.
    original.exists().then_some(original)
}

/// Map a path inside the mount back onto the mount's source.
///
/// The layout is `<mount_point>/d/<Bundle>.app/<rest>`, and the source already
/// names the bundle, so the `d` wrapper and the bundle component are dropped
/// and only `<rest>` is grafted on. Split out from the statfs call so the
/// reshaping can be tested without a live mount.
#[cfg(target_os = "macos")]
fn graft_onto_source(mount_point: &Path, mount_source: &Path, path: &Path) -> Option<PathBuf> {
    let relative = path.strip_prefix(mount_point).ok()?;
    let mut components = relative.components();
    components.next()?; // the "d" wrapper directory
    components.next()?; // <Bundle>.app, which mount_source already names
    Some(mount_source.join(components.as_path()))
}

#[cfg(target_os = "macos")]
fn statfs(path: &Path) -> Option<libc::statfs> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let c_path = CString::new(path.as_os_str().as_bytes()).ok()?;

    // SAFETY: c_path is a valid NUL-terminated string that outlives the call,
    // and statfs only writes into the buffer we hand it.
    let mut buf: libc::statfs = unsafe { std::mem::zeroed() };
    (unsafe { libc::statfs(c_path.as_ptr(), &mut buf) } == 0).then_some(buf)
}

/// Read one of statfs's fixed-size NUL-terminated name fields.
#[cfg(target_os = "macos")]
fn c_chars_to_path(field: &[libc::c_char]) -> Option<PathBuf> {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let bytes: Vec<u8> = field
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8)
        .collect();

    (!bytes.is_empty()).then(|| PathBuf::from(OsString::from_vec(bytes)))
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
    fn mount_source_is_grafted_back_onto() {
        // Exactly the shape `mount` reports for a translocated app:
        //   /Applications/cignaler.app on /private/var/.../AppTranslocation/<uuid>
        let mount_point =
            Path::new("/private/var/folders/s7/x/T/AppTranslocation/D83C19CB-F1EE-450F");
        let mount_source = Path::new("/Applications/cignaler.app");
        let inside = mount_point.join("d/cignaler.app/Contents/MacOS/cignaler-native-host");

        assert_eq!(
            graft_onto_source(mount_point, mount_source, &inside),
            Some(PathBuf::from(
                "/Applications/cignaler.app/Contents/MacOS/cignaler-native-host"
            ))
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn grafting_declines_paths_that_are_not_inside_the_mount() {
        let mount_point = Path::new("/private/var/folders/s7/x/T/AppTranslocation/ABC");
        let mount_source = Path::new("/Applications/cignaler.app");

        // Not under the mount point at all.
        assert_eq!(
            graft_onto_source(mount_point, mount_source, Path::new("/Applications/other.app")),
            None
        );
        // Under it, but too shallow to carry a bundle component.
        assert_eq!(
            graft_onto_source(mount_point, mount_source, &mount_point.join("d")),
            None
        );
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
