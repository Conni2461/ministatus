use std::process::Command;

pub struct Volume {
    regex: regex::Regex,
}

impl Volume {
    pub fn new() -> Result<Box<Self>, anyhow::Error> {
        Ok(Box::new(Self {
            regex: regex::Regex::new(r"(\d+%)")?,
        }))
    }
}

impl super::Block for Volume {
    fn run(&self) -> Result<Option<String>, anyhow::Error> {
        let mute = String::from_utf8(
            Command::new("pactl")
                .arg("get-sink-mute")
                .arg("@DEFAULT_SINK@")
                .output()?
                .stdout,
        )?
        .trim()
        .strip_prefix("Mute: ")
        .map(|v| v == "yes")
        .unwrap_or_default();

        if mute {
            return Ok(Some("🔇".into()));
        }

        let volume = String::from_utf8(
            Command::new("pactl")
                .arg("get-sink-volume")
                .arg("@DEFAULT_SINK@")
                .output()?
                .stdout,
        )?;
        let vol = self
            .regex
            .find(&volume)
            .and_then(|m| m.as_str()[..m.len() - 1].parse::<i32>().ok());
        if let Some(vol) = vol {
            let symbol = if vol > 70 {
                "🔊"
            } else if vol > 30 {
                "🔉"
            } else {
                "🔈"
            };
            Ok(Some(format!("{symbol} {vol}%")))
        } else {
            Ok(None)
        }
    }
}
