pub struct Internet {}

impl Internet {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}

#[async_trait::async_trait]
impl super::Block for Internet {
    async fn run(&self) -> Result<Option<String>, anyhow::Error> {
        let tuple = super::file_as_vec_str("/proc/net/wireless")?
            .into_iter()
            .find(|s| s.starts_with('w'))
            .map(|v| {
                let a = v.split_whitespace().collect::<Vec<_>>();
                (
                    a.first().map(|v| {
                        let mut v = v.to_string();
                        v.pop();
                        v
                    }),
                    a.get(2).and_then(|v| v.parse::<f32>().ok()),
                )
            });
        let Some((Some(id), Some(val))) = tuple else {
            return Ok(None);
        };
        let state = super::file_as_vec_str(&format!("/sys/class/net/{id}/operstate"))?
            .get(0)
            .map(|o| o == "up")
            .unwrap_or(false);
        let icon = if state { "🌍" } else { "❎" };

        let val = (val * 100.0 / 70.0) as i32;
        Ok(Some(format!("{icon} {val}%")))
    }
}
