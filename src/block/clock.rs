pub struct Clock {}

impl Clock {
    pub const fn new() -> Self {
        Self {}
    }
}

fn render<Tz: chrono::TimeZone>(now: &chrono::DateTime<Tz>, opts: super::Options) -> String
where
    Tz::Offset: std::fmt::Display,
{
    let fmt = if opts.compact {
        "%m/%d %H:%M"
    } else {
        "(KW%V) %m/%d/%Y %I:%M %p"
    };
    format!("🕛 {}", now.format(fmt))
}

impl super::Block for Clock {
    fn run(&self, opts: super::Options) -> Result<Option<String>, anyhow::Error> {
        Ok(Some(render(&chrono::offset::Local::now(), opts)))
    }
}

#[cfg(test)]
mod tests {
    use super::render;
    use crate::block::Options;

    fn at(s: &str) -> chrono::DateTime<chrono::FixedOffset> {
        chrono::DateTime::parse_from_rfc3339(s).unwrap()
    }

    #[test]
    fn full_keeps_week_number_year_and_meridiem() {
        let now = at("2026-09-05T12:32:00+02:00");
        assert_eq!(
            render(&now, Options::default()),
            "🕛 (KW36) 09/05/2026 12:32 PM"
        );
    }

    #[test]
    fn compact_drops_week_number_and_year_and_uses_24h() {
        let now = at("2026-09-05T13:32:00+02:00");
        assert_eq!(render(&now, Options { compact: true }), "🕛 09/05 13:32");
    }
}
