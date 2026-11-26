//! Settings for PCs with little memory (lowspec branch): below
//! [`LOW_MEMORY`] of physical memory, the switches that keep the most of
//! our own memory are turned down, unless the user set them. They change
//! how much is kept for reuse or held in flight, not what is drawn; each
//! costs some speed. `VICTORY_LOWSPEC=0` leaves them all at the fork's own
//! defaults, `VICTORY_LOWSPEC=1` turns them down on any PC.
//!
//! The fork's own measurements (docs/findings.md): 40-46 GB private in a
//! run, of it lists' allocators 9 GB, windows 7, uploads 4.

/// Below this much physical memory the settings are turned down.
pub const LOW_MEMORY: u64 = 24 << 30;

/// The switches and their lower values.
pub const LOW: &[(&str, &str)] = &[
    // Lists in flight (default 256): each holds its allocator, upload
    // memory and scratch until it has run.
    ("VICTORY_LATE_WAIT_LISTS", "64"),
    // Title memory opened in place and locked into the PC's memory
    // (default 4096 MB).
    ("VICTORY_INPLACE_MB", "1024"),
    // Snapshots kept for reuse (default 1024) and their bytes (default 256 MB).
    ("VICTORY_SNAPSHOT_POOL", "256"),
    ("VICTORY_KEEP_SNAPSHOTS_MB", "128"),
];

/// The switches to set: none when `asked` is "0", all of [`LOW`] when it
/// is "1" or the PC has less than [`LOW_MEMORY`], none otherwise; never one
/// the environment sets already (`set`).
pub fn switches(asked: Option<&str>, physical: Option<u64>, set: impl Fn(&str) -> bool) -> Vec<(&'static str, &'static str)> {
    let on = match asked {
        Some("0") => false,
        Some("1") => true,
        _ => physical.is_some_and(|bytes| bytes < LOW_MEMORY),
    };
    if !on {
        return Vec::new();
    }
    LOW.iter().copied().filter(|(name, _)| !set(name)).collect()
}

/// The PC's physical memory in bytes.
#[cfg(windows)]
pub fn physical_memory() -> Option<u64> {
    use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    let mut m = MEMORYSTATUSEX {
        dwLength: size_of::<MEMORYSTATUSEX>() as u32,
        ..Default::default()
    };
    // SAFETY: a sized MEMORYSTATUSEX for the call to fill.
    (unsafe { GlobalMemoryStatusEx(&mut m) } != 0).then_some(m.ullTotalPhys)
}

#[cfg(not(windows))]
pub fn physical_memory() -> Option<u64> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const GB: u64 = 1 << 30;

    #[test]
    fn low_below_24_gb_unless_told() {
        let none = |_: &str| false;
        assert_eq!(switches(None, Some(16 * GB), none).len(), LOW.len());
        assert!(switches(None, Some(32 * GB), none).is_empty());
        assert!(switches(None, None, none).is_empty());
        assert!(switches(Some("0"), Some(8 * GB), none).is_empty());
        assert_eq!(switches(Some("1"), Some(64 * GB), none).len(), LOW.len());
    }

    #[test]
    fn a_switch_already_set_keeps_its_value() {
        let kept = switches(Some("1"), None, |n| n == "VICTORY_INPLACE_MB");
        assert_eq!(kept.len(), LOW.len() - 1);
        assert!(kept.iter().all(|(n, _)| *n != "VICTORY_INPLACE_MB"));
    }
}
