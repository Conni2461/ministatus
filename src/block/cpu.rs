use std::path::Path;

use super::read_into;

const STAT: &str = "/proc/stat";
const LOADAVG: &str = "/proc/loadavg";

#[derive(Debug, Clone, Copy)]
struct Sample {
    total: u64,
    idle: u64,
}

pub struct Cpu {
    prev: Option<Sample>,
    stat: Vec<u8>,
    loadavg: Vec<u8>,
}

fn parse_stat(s: &str) -> Option<Sample> {
    let line = s.lines().find(|l| l.starts_with("cpu "))?;

    let (mut total, mut idle, mut seen) = (0u64, 0u64, 0usize);
    for (i, field) in line.split_whitespace().skip(1).enumerate() {
        let v = field.parse::<u64>().ok()?;
        total += v;
        if i == 3 || i == 4 {
            idle += v;
        }
        seen = i + 1;
    }
    if seen < 5 {
        return None;
    }

    Some(Sample { total, idle })
}

fn parse_loadavg(s: &str) -> Option<String> {
    let mut fields = s.split_whitespace();
    let (one, five, fifteen) = (fields.next()?, fields.next()?, fields.next()?);

    let mut out = String::with_capacity(one.len() + five.len() + fifteen.len() + 2);
    out.push_str(one);
    out.push(' ');
    out.push_str(five);
    out.push(' ');
    out.push_str(fifteen);
    Some(out)
}

impl Cpu {
    pub fn new() -> Self {
        let mut stat = Vec::new();
        let prev = read_into(Path::new(STAT), &mut stat)
            .ok()
            .and_then(parse_stat);

        Self {
            prev,
            stat,
            loadavg: Vec::new(),
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
    fn run(&mut self, opts: &super::Options) -> Result<Option<String>, anyhow::Error> {
        // owned, so the borrow of `self.loadavg` ends before `self.prev` is touched
        let load = if opts.compact {
            None
        } else {
            let s = read_into(Path::new(LOADAVG), &mut self.loadavg)?;
            let Some(load) = parse_loadavg(s) else {
                return Ok(None);
            };
            Some(load)
        };

        let Some(cur) = parse_stat(read_into(Path::new(STAT), &mut self.stat)?) else {
            return Ok(None);
        };

        let Some(prev) = self.prev.replace(cur) else {
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
    fn stat_rejects_a_line_too_short_to_hold_idle_and_iowait() {
        assert!(parse_stat("cpu  100 20 30 800\n").is_none());
    }

    #[test]
    fn stat_rejects_a_non_numeric_field() {
        assert!(parse_stat("cpu  100 20 x 800 40\n").is_none());
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
