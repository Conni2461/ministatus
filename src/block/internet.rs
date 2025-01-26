pub struct Internet {}

impl Internet {
    pub const fn new() -> Self {
        Self {}
    }
}

impl super::Block for Internet {
    fn run(&self) -> Result<Option<String>, anyhow::Error> {
        let tuple = std::fs::read_to_string("/proc/net/wireless")?
            .lines()
            .find(|s| s.starts_with('w'))
            .map(|v| {
                let a = v.split_whitespace().collect::<Vec<_>>();
                (
                    a.first().map(|v| {
                        let mut v = (*v).to_string();
                        v.pop();
                        v
                    }),
                    a.get(2).and_then(|v| v.parse::<f32>().ok()),
                )
            });
        let Some((Some(id), Some(val))) = tuple else {
            return Ok(None);
        };
        let icon = if std::fs::read_to_string(format!("/sys/class/net/{id}/operstate"))?
            .lines()
            .next()
            == Some("up")
        {
            "🌍"
        } else {
            "❎"
        };

        #[allow(clippy::cast_possible_truncation)]
        let val = (val * 100.0 / 70.0) as i32;
        Ok(Some(format!("{icon} {val}%")))
    }
}
