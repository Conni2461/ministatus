use tokio::process::Command;

pub struct Weather {}

impl Weather {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}

#[async_trait::async_trait]
impl super::Block for Weather {
    async fn run(&self) -> Result<Option<String>, anyhow::Error> {
        let w = String::from_utf8(Command::new("weather").output().await?.stdout)?
            .trim()
            .to_string();
        if w.is_empty() {
            Ok(None)
        } else {
            Ok(Some(w))
        }
    }
}
