use std::path::PathBuf;
use std::time::Duration;

pub struct Mailbox {
    root: PathBuf,
}

impl Mailbox {
    pub fn new(home: &str) -> Result<Self, anyhow::Error> {
        let root = PathBuf::from(format!("{home}/.local/share/mail"));
        if !root.exists() {
            return Err(anyhow::anyhow!("mailbox does not exist"));
        }

        Ok(Self { root })
    }
}

impl super::Block for Mailbox {
    fn run(&mut self, _: super::Options) -> Result<Option<String>, anyhow::Error> {
        let mut c = 0usize;
        for account in std::fs::read_dir(&self.root)?.flatten() {
            if let Ok(dir) = std::fs::read_dir(account.path().join("INBOX/new")) {
                c += dir.count();
            }
        }

        if c == 0 {
            Ok(None)
        } else {
            Ok(Some(format!("📬 {c}")))
        }
    }

    fn interval(&self) -> Duration {
        Duration::from_secs(15)
    }
}
