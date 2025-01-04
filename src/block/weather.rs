use std::fmt::Display;
use std::str::FromStr;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Deserializer};

// every 4h (timeout 1s)
const TIMEOUT_TIME: i32 = 60 * 60 * 4;

pub fn deserialize_number_from_string<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + Deserialize<'de>,
    <T as FromStr>::Err: Display,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt<T> {
        String(String),
        Number(T),
    }

    match StringOrInt::<T>::deserialize(deserializer)? {
        StringOrInt::String(s) => s.parse::<T>().map_err(serde::de::Error::custom),
        StringOrInt::Number(i) => Ok(i),
    }
}

#[derive(Debug, Clone, Copy)]
struct Data {
    rain: i32,
    min_temp: i32,
    max_temp: i32,
}

#[derive(Deserialize, Debug)]
struct Response {
    weather: Vec<WeatherResponse>,
}

#[derive(Deserialize, Debug)]
struct WeatherResponse {
    hourly: Vec<WeatherHourly>,
}

#[derive(Deserialize, Debug)]
struct WeatherHourly {
    #[serde(rename = "tempC", deserialize_with = "deserialize_number_from_string")]
    temp_c: i32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    time: i32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    chanceofrain: i32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    chanceofsnow: i32,
}

pub struct Weather {
    agent: ureq::Agent,
    data: Arc<RwLock<Option<Data>>>,

    timeout: Arc<AtomicI32>,
}

fn get_weather_data(agent: &ureq::Agent) -> Result<Option<Data>, anyhow::Error> {
    let output: Response = agent
        .get("https://wttr.in")
        .query("format", "j1")
        .header("accept", "application/json")
        .call()?
        .body_mut()
        .read_json()?;
    let Some(data) = output.weather.first() else {
        return Ok(None);
    };

    let filtered_data = data
        .hourly
        .iter()
        .filter(|v| v.time >= 900 && v.time <= 2100);
    let Some(min_temp) = filtered_data.clone().map(|v| v.temp_c).min() else {
        return Ok(None);
    };
    let Some(max_temp) = filtered_data.clone().map(|v| v.temp_c).max() else {
        return Ok(None);
    };
    let Some(rain) = filtered_data
        .clone()
        .flat_map(|v| std::iter::once(v.chanceofrain).chain(std::iter::once(v.chanceofsnow)))
        .max()
    else {
        return Ok(None);
    };

    Ok(Some(Data {
        rain,
        min_temp,
        max_temp,
    }))
}

impl Weather {
    pub fn new() -> Self {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(std::time::Duration::from_secs(2)))
            .tls_config(
                ureq::tls::TlsConfig::builder()
                    .provider(ureq::tls::TlsProvider::NativeTls)
                    .build(),
            )
            .build()
            .into();

        let data = match get_weather_data(&agent) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("failed to retrieve weather data: {e}");
                None
            }
        };
        let timeout = if data.is_some() {
            AtomicI32::new(TIMEOUT_TIME)
        } else {
            // if first fetch is None try again 60 ticks later
            AtomicI32::new(60)
        };

        Self {
            agent,
            data: Arc::new(RwLock::new(data)),

            timeout: Arc::new(timeout),
        }
    }

    fn refresh_data(&self) {
        self.timeout.fetch_sub(1, Ordering::SeqCst);
        if self.timeout.load(Ordering::Relaxed) == 0 {
            let timeout = self.timeout.clone();

            let d = self.data.clone();
            let agent = self.agent.clone();
            std::thread::spawn(move || {
                let new = get_weather_data(&agent).unwrap_or_default();
                if new.is_none() {
                    // if refresh data is still None, move refresh time back to 3600 ticks aka 1h
                    timeout.store(60 * 60, Ordering::Relaxed);
                    return;
                }

                timeout.store(TIMEOUT_TIME, Ordering::Relaxed);
                let mut w = d.write().unwrap();
                *w = new;
            });
        }
    }
}

impl super::Block for Weather {
    fn run(&self) -> Result<Option<String>, anyhow::Error> {
        self.refresh_data();
        self.data.read().unwrap().map_or_else(
            || Ok(None),
            |d| {
                Ok(Some(format!(
                    "☂️ {}% ❄ {}° ☀️ {}°",
                    d.rain, d.min_temp, d.max_temp
                )))
            },
        )
    }
}
