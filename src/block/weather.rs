use std::fmt::Display;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

use serde::{Deserialize, Deserializer};

const REFRESH: Duration = Duration::from_hours(4);
const FIRST_RETRY: Duration = Duration::from_mins(1);
const MAX_RETRY: Duration = Duration::from_hours(1);

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
    backoff: Arc<Mutex<Duration>>,
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

fn next_delay(backoff: &mut Duration, ok: bool) -> Duration {
    if ok {
        *backoff = FIRST_RETRY;
        return REFRESH;
    }
    let delay = *backoff;
    *backoff = (delay * 2).min(MAX_RETRY);
    delay
}

impl Weather {
    pub fn new() -> Self {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(10)))
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
            backoff: Arc::new(Mutex::new(FIRST_RETRY)),
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
        let backoff = self.backoff.clone();
        let fetching = self.fetching.clone();
        let agent = self.agent.clone();

        std::thread::spawn(move || {
            let new = match get_weather_data(&agent) {
                Ok(Some(v)) => Some(v),
                Ok(None) => {
                    eprintln!("weather response held no usable hourly readings");
                    None
                }
                Err(e) => {
                    eprintln!("failed to retrieve weather data: {e}");
                    None
                }
            };

            let ok = new.is_some();
            if let Some(new) = new
                && let Ok(mut w) = data.write()
            {
                *w = Some(new);
            }

            let delay = match backoff.lock() {
                Ok(mut b) => next_delay(&mut b, ok),
                Err(_) => MAX_RETRY,
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
        match self.data.read() {
            Ok(d) if d.is_some() => Duration::from_mins(1),
            _ => super::TICK,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Data, Duration, FIRST_RETRY, MAX_RETRY, REFRESH, next_delay, render};
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

    #[test]
    fn a_cold_start_failure_retries_within_the_minute() {
        let mut backoff = FIRST_RETRY;
        assert_eq!(next_delay(&mut backoff, false), FIRST_RETRY);
    }

    #[test]
    fn repeated_failures_back_off_and_settle_at_the_cap() {
        let mut backoff = FIRST_RETRY;
        let mut seen = Vec::new();
        for _ in 0..10 {
            seen.push(next_delay(&mut backoff, false));
        }
        assert_eq!(seen[0], Duration::from_mins(1));
        assert_eq!(seen[1], Duration::from_mins(2));
        assert_eq!(seen[2], Duration::from_mins(4));
        assert_eq!(*seen.last().unwrap(), MAX_RETRY);
        assert!(seen.windows(2).all(|w| w[0] <= w[1]));
    }

    #[test]
    fn a_success_returns_the_long_refresh_and_clears_the_backoff() {
        let mut backoff = MAX_RETRY;
        assert_eq!(next_delay(&mut backoff, true), REFRESH);
        assert_eq!(backoff, FIRST_RETRY);
        assert_eq!(next_delay(&mut backoff, false), FIRST_RETRY);
    }
}
