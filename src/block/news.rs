const STMT: &str = "SELECT Count(*) FROM rss_item WHERE unread = 1;";

pub struct News {
    home: String,
    conn: rusqlite::Connection,
}

impl News {
    pub fn new(home: &str) -> Result<Self, anyhow::Error> {
        let dbfile = format!("{home}/.local/share/newsboat/cache.db");
        if !std::path::Path::new(&dbfile).exists() {
            return Err(anyhow::anyhow!("file does not exist"));
        }

        Ok(Self {
            home: home.to_owned(),
            conn: rusqlite::Connection::open_with_flags(
                dbfile,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
            )?,
        })
    }
}

impl super::Block for News {
    fn run(&self) -> Result<Option<String>, anyhow::Error> {
        if std::path::Path::new(&format!("{}/.config/newsboat/.update", self.home)).exists() {
            return Ok(Some("📰 🔃".into()));
        }

        let news = self.conn.query_row(STMT, [], |row| row.get::<_, i32>(0))?;
        if news == 0 {
            Ok(None)
        } else {
            Ok(Some(format!("📰 {news}")))
        }
    }
}
