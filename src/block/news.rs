use std::path::PathBuf;
use std::time::Duration;

const STMT: &str = "SELECT Count(*) FROM rss_item WHERE unread = 1;";

pub struct News {
    update: PathBuf,
    conn: rusqlite::Connection,
}

impl News {
    pub fn new(home: &str) -> Result<Self, anyhow::Error> {
        let dbfile = format!("{home}/.local/share/newsboat/cache.db");
        if !std::path::Path::new(&dbfile).exists() {
            return Err(anyhow::anyhow!("file does not exist"));
        }

        Ok(Self {
            update: PathBuf::from(format!("{home}/.config/newsboat/.update")),
            conn: rusqlite::Connection::open_with_flags(
                dbfile,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
            )?,
        })
    }
}

impl super::Block for News {
    fn run(&mut self, _: super::Options) -> Result<Option<String>, anyhow::Error> {
        if self.update.exists() {
            return Ok(Some("📰 🔃".into()));
        }

        let news = self
            .conn
            .prepare_cached(STMT)?
            .query_row([], |row| row.get::<_, i32>(0))?;

        if news == 0 {
            Ok(None)
        } else {
            Ok(Some(format!("📰 {news}")))
        }
    }

    fn interval(&self) -> Duration {
        Duration::from_secs(15)
    }
}
