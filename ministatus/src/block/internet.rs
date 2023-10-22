use tokio::process::Command;

pub struct Internet {}

impl Internet {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}

#[async_trait::async_trait]
impl super::Block for Internet {
    async fn run(&self) -> Result<Option<String>, anyhow::Error> {
        let i = String::from_utf8(Command::new("internet").output().await?.stdout)?
            .trim()
            .to_string();
        if i.is_empty() {
            Ok(None)
        } else {
            Ok(Some(i))
        }
    }
}
