use std::ffi::CString;
use std::time::Duration;

use super::gib;

const DEFAULT_TARGET: &str = "/";

pub struct Disk;

impl Disk {
    pub const fn new() -> Self {
        Self
    }
}

fn statvfs(target: &str) -> Result<(u64, u64), anyhow::Error> {
    let path = CString::new(target)?;
    let mut fs: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(path.as_ptr(), &raw mut fs) } != 0 {
        return Err(std::io::Error::last_os_error().into());
    }

    let unit: u64 = fs.f_frsize;
    let total = fs.f_blocks.saturating_mul(unit);
    let used = fs.f_blocks.saturating_sub(fs.f_bfree).saturating_mul(unit);
    Ok((used, total))
}

fn render(used: u64, total: u64, opts: &super::Options) -> String {
    let (used_kib, total_kib) = (used / 1024, total / 1024);
    let pct = used * 100 / total;
    if opts.compact {
        format!("💾 {:.1}G", gib(used_kib))
    } else {
        format!("💾 {:.1}/{:.1}G ({pct}%)", gib(used_kib), gib(total_kib))
    }
}

impl super::Block for Disk {
    fn run(&mut self, opts: &super::Options) -> Result<Option<String>, anyhow::Error> {
        let target = opts.target.as_deref().unwrap_or(DEFAULT_TARGET);
        let (used, total) = statvfs(target)?;
        if total == 0 {
            return Ok(None);
        }
        Ok(Some(render(used, total, opts)))
    }

    fn interval(&self) -> Duration {
        Duration::from_secs(30)
    }
}

#[cfg(test)]
mod tests {
    use super::{DEFAULT_TARGET, render, statvfs};
    use crate::block::Options;

    const SAMPLE: (u64, u64) = (229_681_233_920, 500_684_042_240);

    #[test]
    fn full_shows_used_total_and_percentage() {
        assert_eq!(
            render(SAMPLE.0, SAMPLE.1, &Options::default()),
            "💾 213.9/466.3G (45%)"
        );
    }

    #[test]
    fn compact_shows_used_alone() {
        assert_eq!(
            render(
                SAMPLE.0,
                SAMPLE.1,
                &Options {
                    compact: true,
                    target: None,
                }
            ),
            "💾 213.9G"
        );
    }

    #[test]
    fn statvfs_reads_the_root_filesystem() {
        let (used, total) = statvfs(DEFAULT_TARGET).unwrap();
        assert!(total > used);
    }
}
