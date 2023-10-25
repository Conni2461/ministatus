pub struct Weather {
    data: Vec<String>,
    rain_regex: regex::Regex,
    temp_regex: regex::Regex,
}

impl Weather {
    pub async fn new() -> Result<Box<Self>, anyhow::Error> {
        Ok(Box::new(Self {
            data: Self::get_weather_data().await.unwrap_or_default(),
            rain_regex: regex::Regex::new(r"(\d+%)")?,
            temp_regex: regex::Regex::new(r"(\+\d+)")?,
        }))
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
}

#[async_trait::async_trait]
impl super::Block for Weather {
    async fn run(&self) -> Result<Option<String>, anyhow::Error> {
        // TODO: refresh data every hour or so
        let Some(r) = self.data.get(15) else {
            return Ok(None);
        };
        let Some(h) = self.data.get(12) else {
            return Ok(None);
        };

        let rain = self
            .rain_regex
            .find_iter(r)
            .filter_map(|m| m.as_str()[..m.len() - 1].parse::<i32>().ok())
            .max();
        let Some(rain) = rain else {
            return Ok(None);
        };

        let temp = self
            .temp_regex
            .find_iter(h)
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
