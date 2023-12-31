pub struct Clock {}

impl Clock {
    pub const fn new() -> Self {
        Self {}
    }
}

impl super::Block for Clock {
    fn run(&self) -> Result<Option<String>, anyhow::Error> {
        Ok(Some(format!(
            "🕛 {}",
            chrono::offset::Local::now().format("(KW%V) %m/%d/%Y %I:%M %p")
        )))
    }
}
