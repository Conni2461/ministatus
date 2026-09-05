use std::sync::RwLock;

#[derive(Debug, Clone, Copy)]
struct Sample {
    total: u64,
    idle: u64,
}

pub struct Cpu {
    prev: RwLock<Option<Sample>>,
}

fn parse_stat(s: &str) -> Option<Sample> {
    let fields = s
        .lines()
        .find(|l| l.starts_with("cpu "))?
        .split_whitespace()
        .skip(1)
        .map(|v| v.parse::<u64>().ok())
        .collect::<Option<Vec<_>>>()?;

    let idle = fields.get(3)? + fields.get(4)?;
    Some(Sample {
        total: fields.iter().sum(),
        idle,
    })
}

fn parse_loadavg(s: &str) -> Option<String> {
    let load = s.split_whitespace().take(3).collect::<Vec<_>>();
    if load.len() == 3 {
        Some(load.join(" "))
    } else {
        None
    }
}

fn read_stat() -> Option<Sample> {
    parse_stat(&std::fs::read_to_string("/proc/stat").ok()?)
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            prev: RwLock::new(read_stat()),
        }
    }
}

fn render(pct: u64, load: Option<&str>) -> String {
    match load {
        Some(load) => format!("⚙ {pct}% ({load})"),
        None => format!("⚙ {pct}%"),
    }
}

impl super::Block for Cpu {
    fn run(&self, opts: super::Options) -> Result<Option<String>, anyhow::Error> {
        let load = if opts.compact {
            None
        } else {
            let Some(load) = parse_loadavg(&std::fs::read_to_string("/proc/loadavg")?) else {
                return Ok(None);
            };
            Some(load)
        };

        let Some(cur) = read_stat() else {
            return Ok(None);
        };

        let prev = self.prev.write().unwrap().replace(cur);
        let Some(prev) = prev else {
            return Ok(None);
        };

        let total = cur.total.saturating_sub(prev.total);
        let idle = cur.idle.saturating_sub(prev.idle);
        if total == 0 {
            return Ok(None);
        }
        let pct = total.saturating_sub(idle) * 100 / total;

        Ok(Some(render(pct, load.as_deref())))
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_loadavg, parse_stat, render};

    #[test]
    fn stat_sums_the_aggregate_line() {
        let s = "cpu  100 20 30 800 40 0 10 0 0 0\ncpu0 50 10 15 400 20 0 5 0 0 0\n";
        let sample = parse_stat(s).unwrap();
        assert_eq!(sample.total, 1000);
        assert_eq!(sample.idle, 840);
    }

    #[test]
    fn stat_rejects_input_without_an_aggregate_line() {
        assert!(parse_stat("cpu0 50 10 15 400 20 0 5 0 0 0\n").is_none());
    }

    #[test]
    fn loadavg_keeps_the_first_three_fields() {
        assert_eq!(
            parse_loadavg("1.24 0.88 0.61 1/1234 5678\n").as_deref(),
            Some("1.24 0.88 0.61")
        );
    }

    #[test]
    fn loadavg_rejects_short_input() {
        assert!(parse_loadavg("1.24 0.88\n").is_none());
    }

    #[test]
    fn full_appends_the_load_average() {
        assert_eq!(render(12, Some("1.24 0.88 0.61")), "⚙ 12% (1.24 0.88 0.61)");
    }

    #[test]
    fn compact_shows_the_percentage_alone() {
        assert_eq!(render(12, None), "⚙ 12%");
    }
}
