pub struct Mailbox {
    pattern: String,
}

impl Mailbox {
    pub fn new(home: &str) -> Result<Box<Self>, anyhow::Error> {
        if !std::path::Path::new(&format!("{}/.local/share/mail/", home)).exists() {
            return Err(anyhow::anyhow!("mailbox does not exist"));
        }

        Ok(Box::new(Self {
            pattern: format!("{}/.local/share/mail/*/INBOX/new/*", home),
        }))
    }
}

#[async_trait::async_trait]
impl super::Block for Mailbox {
    async fn run(&self) -> Result<Option<String>, anyhow::Error> {
        let mut c = 0;
        for _ in glob::glob(&self.pattern)? {
            c += 1;
        }

        if c == 0 {
            Ok(None)
        } else {
            Ok(Some(format!("📬 {c}")))
        }
    }
}
