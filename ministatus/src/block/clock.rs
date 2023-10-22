pub struct Clock {}

impl Clock {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}

#[async_trait::async_trait]
impl super::Block for Clock {
    async fn run(&self) -> Result<Option<String>, anyhow::Error> {
        Ok(Some(format!(
            "🕛 {}",
            chrono::offset::Local::now().format("%m/%d/%Y %I:%M %p")
        )))
    }
}
