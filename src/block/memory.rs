use std::path::Path;
use std::time::Duration;

use super::read_into;

const MEMINFO: &str = "/proc/meminfo";

pub struct Memory {
    buf: Vec<u8>,
}

impl Memory {
    pub const fn new() -> Self {
        Self { buf: Vec::new() }
    }
}

fn parse_meminfo(s: &str) -> Option<(u64, u64)> {
    let (mut total, mut available) = (None, None);

    for line in s.lines() {
        let slot = if line.starts_with("MemTotal:") {
            &mut total
        } else if line.starts_with("MemAvailable:") {
            &mut available
        } else {
            continue;
        };
        *slot = line.split_whitespace().nth(1)?.parse::<u64>().ok();

        if total.is_some() && available.is_some() {
            break;
        }
    }

    Some((total?, available?))
}

#[allow(clippy::cast_precision_loss)]
fn gib(kb: u64) -> f64 {
    kb as f64 / 1_048_576.0
}

fn render(used: u64, total: u64, opts: super::Options) -> String {
    let pct = used * 100 / total;
    if opts.compact {
        format!("🧠 {:.1}G", gib(used))
    } else {
        format!("🧠 {:.1}/{:.1}G ({pct}%)", gib(used), gib(total))
    }
}

impl super::Block for Memory {
    fn run(&mut self, opts: super::Options) -> Result<Option<String>, anyhow::Error> {
        let Some((total, available)) = parse_meminfo(read_into(Path::new(MEMINFO), &mut self.buf)?)
        else {
            return Ok(None);
        };
        if total == 0 {
            return Ok(None);
        }

        let used = total.saturating_sub(available);
        Ok(Some(render(used, total, opts)))
    }

    fn interval(&self) -> Duration {
        Duration::from_secs(2)
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_meminfo, render};
    use crate::block::Options;

    #[test]
    fn meminfo_finds_total_and_available() {
        let s = "MemTotal:       32724116 kB\nMemFree:         1234567 kB\nMemAvailable:   22801748 kB\n";
        assert_eq!(parse_meminfo(s), Some((32_724_116, 22_801_748)));
    }

    #[test]
    fn meminfo_finds_the_fields_in_either_order() {
        let s = "MemAvailable:   22801748 kB\nMemTotal:       32724116 kB\n";
        assert_eq!(parse_meminfo(s), Some((32_724_116, 22_801_748)));
    }

    #[test]
    fn meminfo_rejects_input_without_available() {
        let s = "MemTotal:       32724116 kB\nMemFree:         1234567 kB\n";
        assert!(parse_meminfo(s).is_none());
    }

    #[test]
    fn meminfo_rejects_a_non_numeric_value() {
        let s = "MemTotal:       kB\nMemAvailable:   22801748 kB\n";
        assert!(parse_meminfo(s).is_none());
    }

    #[test]
    fn full_shows_used_total_and_percentage() {
        let (used, total) = (9_961_472, 32_724_116);
        assert_eq!(
            render(used, total, Options::default()),
            "🧠 9.5/31.2G (30%)"
        );
    }

    #[test]
    fn compact_shows_used_alone() {
        let (used, total) = (9_961_472, 32_724_116);
        assert_eq!(render(used, total, Options { compact: true }), "🧠 9.5G");
    }
}
