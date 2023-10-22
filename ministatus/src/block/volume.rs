use tokio::process::Command;

pub struct Volume {}

impl Volume {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}

#[async_trait::async_trait]
impl super::Block for Volume {
    async fn run(&self) -> Result<Option<String>, anyhow::Error> {
        let v = String::from_utf8(Command::new("volume").output().await?.stdout)?
            .trim()
            .to_string();
        if v.is_empty() {
            Ok(None)
        } else {
            Ok(Some(v))
        }
    }
}
