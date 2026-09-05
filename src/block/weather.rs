use std::fmt::Display;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

use serde::{Deserialize, Deserializer};

const REFRESH: Duration = Duration::from_hours(4);
const RETRY: Duration = Duration::from_hours(1);

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

    next: Arc<Mutex<Instant>>,
    fetching: Arc<AtomicBool>,
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
                    .provider(ureq::tls::TlsProvider::Rustls)
                    .build(),
            )
            .build()
            .into();

        Self {
            agent,
            data: Arc::new(RwLock::new(None)),

            next: Arc::new(Mutex::new(Instant::now())),
            fetching: Arc::new(AtomicBool::new(false)),
        }
    }

    fn refresh_data(&self) {
        if self.fetching.load(Ordering::Acquire) {
            return;
        }
        match self.next.lock() {
            Ok(next) if Instant::now() < *next => return,
            Ok(_) => (),
            Err(_) => return,
        }
        self.fetching.store(true, Ordering::Release);

        let data = self.data.clone();
        let next = self.next.clone();
        let fetching = self.fetching.clone();
        let agent = self.agent.clone();

        std::thread::spawn(move || {
            let new = match get_weather_data(&agent) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("failed to retrieve weather data: {e}");
                    None
                }
            };

            let delay = if new.is_some() {
                if let Ok(mut w) = data.write() {
                    *w = new;
                }
                REFRESH
            } else {
                RETRY
            };
            if let Ok(mut n) = next.lock() {
                *n = Instant::now() + delay;
            }

            fetching.store(false, Ordering::Release);
        });
    }
}

fn render(d: Data, opts: super::Options) -> String {
    if opts.compact {
        format!("☂️ {}% {}/{}°", d.rain, d.min_temp, d.max_temp)
    } else {
        format!("☂️ {}% ❄ {}° ☀️ {}°", d.rain, d.min_temp, d.max_temp)
    }
}

impl super::Block for Weather {
    fn run(&mut self, opts: super::Options) -> Result<Option<String>, anyhow::Error> {
        self.refresh_data();

        let Ok(data) = self.data.read() else {
            return Ok(None);
        };
        Ok(data.map(|d| render(d, opts)))
    }

    fn interval(&self) -> Duration {
        Duration::from_mins(1)
    }
}

#[cfg(test)]
mod tests {
    use super::{Data, render};
    use crate::block::Options;

    const SAMPLE: Data = Data {
        rain: 20,
        min_temp: 12,
        max_temp: 24,
    };

    #[test]
    fn full_labels_each_metric_with_its_own_emoji() {
        assert_eq!(render(SAMPLE, Options::default()), "☂️ 20% ❄ 12° ☀️ 24°");
    }

    #[test]
    fn compact_collapses_the_temperatures_into_a_range() {
        assert_eq!(render(SAMPLE, Options { compact: true }), "☂️ 20% 12/24°");
    }
}
