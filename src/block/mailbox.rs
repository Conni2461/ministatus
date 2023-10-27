pub struct Mailbox {
    pattern: String,
}

impl Mailbox {
    pub fn new(home: &str) -> Result<Self, anyhow::Error> {
        if !std::path::Path::new(&format!("{home}/.local/share/mail/")).exists() {
            return Err(anyhow::anyhow!("mailbox does not exist"));
        }

        Ok(Self {
            pattern: format!("{home}/.local/share/mail/*/INBOX/new/*"),
        })
    }
}

impl super::Block for Mailbox {
    fn run(&self) -> Result<Option<String>, anyhow::Error> {
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
