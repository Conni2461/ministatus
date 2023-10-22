use tokio::process::Command;

pub struct News {}

impl News {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}

#[async_trait::async_trait]
impl super::Block for News {
    async fn run(&self) -> Result<Option<String>, anyhow::Error> {
        let news = String::from_utf8(Command::new("news").output().await?.stdout)?
            .trim()
            .to_string();
        if news.is_empty() {
            Ok(None)
        } else {
            Ok(Some(format!("📰 {news}")))
        }
    }
}
