use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

use tokio::sync::RwLock;

// every 4h (timeout 10s)
const TIMEOUT_TIME: i32 = 1440;

pub struct Weather {
    data: Arc<RwLock<Vec<String>>>,
    rain_regex: regex::Regex,
    temp_regex: regex::Regex,

    timeout: AtomicI32,
}

async fn get_weather_data() -> Result<Vec<String>, anyhow::Error> {
    let o = reqwest::Client::new()
        .get("https://wttr.in")
        .header("Accept", "text/plain")
        .header("User-Agent", "curl/8.1.1")
        .send()
        .await?
        .text()
        .await?;
    Ok(o.lines().map(ToOwned::to_owned).collect::<Vec<String>>())
}

impl Weather {
    pub async fn new() -> Result<Box<Self>, anyhow::Error> {
        Ok(Box::new(Self {
            data: Arc::new(RwLock::new(get_weather_data().await.unwrap_or_default())),
            rain_regex: regex::Regex::new(r"(\d+%)")?,
            temp_regex: regex::Regex::new(r"(\+\d+)")?,

            timeout: AtomicI32::new(TIMEOUT_TIME),
        }))
    }

    fn refresh_data(&self) {
        self.timeout.fetch_sub(1, Ordering::SeqCst);
        if self.timeout.load(Ordering::Relaxed) == 0 {
            self.timeout.store(TIMEOUT_TIME, Ordering::Relaxed);

            let d = self.data.clone();
            tokio::spawn(async move {
                let new = get_weather_data().await.unwrap_or_default();
                if new.is_empty() {
                    return;
                }

                let mut w = d.write().await;
                *w = new;
            });
        }
    }
}

#[async_trait::async_trait]
impl super::Block for Weather {
    async fn run(&self) -> Result<Option<String>, anyhow::Error> {
        let (r, h) = if let (Some(r), Some(h)) = {
            let d = self.data.read().await;
            (
                d.get(15).map(ToOwned::to_owned),
                d.get(12).map(ToOwned::to_owned),
            )
        } {
            (r, h)
        } else {
            return Ok(None);
        };
        self.refresh_data();

        let rain = self
            .rain_regex
            .find_iter(&r)
            .filter_map(|m| m.as_str()[..m.len() - 1].parse::<i32>().ok())
            .max();
        let Some(rain) = rain else {
            return Ok(None);
        };

        let temp = self
            .temp_regex
            .find_iter(&h)
            .filter_map(|m| m.as_str()[1..].parse::<i32>().ok())
            .collect::<Vec<_>>();
        let Some(min) = temp.iter().min() else {
            return Ok(None);
        };
        let Some(max) = temp.iter().max() else {
            return Ok(None);
        };

        Ok(Some(format!("☔ {} ❄ {}° ☀ {}°", rain, min, max)))
    }
}
