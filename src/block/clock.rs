pub struct Clock {}

impl Clock {
    pub const fn new() -> Self {
        Self {}
    }
}

fn render(now: &jiff::civil::DateTime, opts: super::Options) -> String {
    let fmt = if opts.compact {
        "%m/%d %H:%M"
    } else {
        "(KW%V) %m/%d/%Y %I:%M %p"
    };
    format!("🕛 {}", now.strftime(fmt))
}

impl super::Block for Clock {
    fn run(&mut self, opts: super::Options) -> Result<Option<String>, anyhow::Error> {
        Ok(Some(render(&jiff::Zoned::now().datetime(), opts)))
    }
}

#[cfg(test)]
mod tests {
    use super::render;
    use crate::block::Options;

    fn at(s: &str) -> jiff::civil::DateTime {
        s.parse().unwrap()
    }

    #[test]
    fn full_keeps_week_number_year_and_meridiem() {
        let now = at("2026-09-05T12:32:00");
        assert_eq!(
            render(&now, Options::default()),
            "🕛 (KW36) 09/05/2026 12:32 PM"
        );
    }

    #[test]
    fn compact_drops_week_number_and_year_and_uses_24h() {
        let now = at("2026-09-05T13:32:00");
        assert_eq!(render(&now, Options { compact: true }), "🕛 09/05 13:32");
    }
}
