pub struct Memory {}

impl Memory {
    pub const fn new() -> Self {
        Self {}
    }
}

fn parse_meminfo(s: &str) -> Option<(u64, u64)> {
    let field = |name: &str| {
        s.lines()
            .find(|l| l.starts_with(name))?
            .split_whitespace()
            .nth(1)?
            .parse::<u64>()
            .ok()
    };

    Some((field("MemTotal:")?, field("MemAvailable:")?))
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
    fn run(&self, opts: super::Options) -> Result<Option<String>, anyhow::Error> {
        let Some((total, available)) = parse_meminfo(&std::fs::read_to_string("/proc/meminfo")?)
        else {
            return Ok(None);
        };
        if total == 0 {
            return Ok(None);
        }

        let used = total.saturating_sub(available);
        Ok(Some(render(used, total, opts)))
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
    fn meminfo_rejects_input_without_available() {
        let s = "MemTotal:       32724116 kB\nMemFree:         1234567 kB\n";
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
