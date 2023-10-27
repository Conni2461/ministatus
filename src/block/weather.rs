use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, RwLock};

// every 4h (timeout 10s)
const TIMEOUT_TIME: i32 = 1440;

#[derive(Debug, Clone, Copy)]
struct Data {
    rain: i32,
    min_temp: i32,
    max_temp: i32,
}

pub struct Weather {
    agent: ureq::Agent,
    data: Arc<RwLock<Option<Data>>>,
    rain_regex: regex::Regex,
    temp_regex: regex::Regex,

    timeout: AtomicI32,
}

fn get_weather_data(
    agent: &ureq::Agent,
    rain_regex: &regex::Regex,
    temp_regex: &regex::Regex,
) -> Result<Option<Data>, anyhow::Error> {
    let output = agent
        .get("https://wttr.in")
        .set("Accept", "text/plain")
        .set("User-Agent", "curl/8.1.1")
        .call()?
        .into_string()?;
    let data = output
        .lines()
        .map(ToOwned::to_owned)
        .collect::<Vec<String>>();
    if data.is_empty() {
        return Ok(None);
    }

    let (r, h) = if let (Some(r), Some(h)) = {
        (
            data.get(15).map(ToOwned::to_owned),
            data.get(12).map(ToOwned::to_owned),
        )
    } {
        (r, h)
    } else {
        return Ok(None);
    };

    let rain = rain_regex
        .find_iter(&r)
        .filter_map(|m| m.as_str()[..m.len() - 1].parse::<i32>().ok())
        .max();
    let Some(rain) = rain else {
            return Ok(None);
        };

    let temp = temp_regex
        .find_iter(&h)
        .filter_map(|m| m.as_str()[1..].parse::<i32>().ok())
        .collect::<Vec<_>>();
    let Some(min) = temp.iter().min() else {
        return Ok(None);
    };
    let Some(max) = temp.iter().max() else {
        return Ok(None);
    };

    Ok(Some(Data {
        rain,
        min_temp: *min,
        max_temp: *max,
    }))
}

impl Weather {
    pub fn new() -> Result<Box<Self>, anyhow::Error> {
        let tls_connector = Arc::new(native_tls::TlsConnector::new()?);
        let agent = ureq::builder().tls_connector(tls_connector).build();

        let rain_regex = regex::Regex::new(r"(\d+%)")?;
        let temp_regex = regex::Regex::new(r"(\+\d+)")?;
        let data = Arc::new(RwLock::new(
            get_weather_data(&agent, &rain_regex, &temp_regex).unwrap_or_default(),
        ));

        Ok(Box::new(Self {
            agent,
            data,
            rain_regex,
            temp_regex,

            timeout: AtomicI32::new(TIMEOUT_TIME),
        }))
    }

    fn refresh_data(&self) {
        self.timeout.fetch_sub(1, Ordering::SeqCst);
        if self.timeout.load(Ordering::Relaxed) == 0 {
            self.timeout.store(TIMEOUT_TIME, Ordering::Relaxed);

            let d = self.data.clone();
            let agent = self.agent.clone();
            let rain_regex = self.rain_regex.clone();
            let temp_regex = self.temp_regex.clone();
            std::thread::spawn(move || {
                let new = get_weather_data(&agent, &rain_regex, &temp_regex).unwrap_or_default();
                if new.is_none() {
                    return;
                }

                let mut w = d.write().unwrap();
                *w = new;
            });
        }
    }
}

impl super::Block for Weather {
    fn run(&self) -> Result<Option<String>, anyhow::Error> {
        self.refresh_data();
        if let Some(d) = *self.data.read().unwrap() {
            Ok(Some(format!(
                "☔ {}% ❄  {}° ☀  {}°",
                d.rain, d.min_temp, d.max_temp
            )))
        } else {
            Ok(None)
        }
    }
}
