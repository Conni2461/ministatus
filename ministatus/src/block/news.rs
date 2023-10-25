const STMT: &str = "SELECT Count(*) FROM rss_item WHERE unread = 1;";

pub struct News {
    home: String,
    conn: sqlite::ConnectionWithFullMutex,
}

impl News {
    pub fn new(home: &str) -> Result<Box<Self>, anyhow::Error> {
        Ok(Box::new(Self {
            home: home.to_owned(),
            conn: sqlite::Connection::open_with_full_mutex(format!(
                "{}/.local/share/newsboat/cache.db",
                home
            ))?,
        }))
    }
}

#[async_trait::async_trait]
impl super::Block for News {
    async fn run(&self) -> Result<Option<String>, anyhow::Error> {
        if std::path::Path::new(&format!("{}/.config/newsboat/.update", self.home)).exists() {
            return Ok(Some("📰 🔃".into()));
        }

        let mut news = 0;
        self.conn.iterate(STMT, |pairs| {
            news = pairs
                .first()
                .and_then(|&(_, val)| val.and_then(|v| v.parse::<i32>().ok()))
                .unwrap_or(0);
            true
        })?;
        if news == 0 {
            Ok(None)
        } else {
            Ok(Some(format!("📰 {news}")))
        }
    }
}
