use std::path::PathBuf;

pub struct Battery {
    batteries: Vec<PathBuf>,
}

impl Battery {
    pub fn new() -> Self {
        let mut batteries = vec![];
        if let Ok(dir) = std::fs::read_dir("/sys/class/power_supply") {
            for ps in dir.flatten() {
                if ps
                    .file_name()
                    .into_string()
                    .map(|x| x.starts_with("BAT"))
                    .unwrap_or(false)
                {
                    batteries.push(ps.path());
                }
            }
        }

        Self { batteries }
    }
}

impl super::Block for Battery {
    fn run(&self) -> Result<Option<String>, anyhow::Error> {
        if self.batteries.is_empty() {
            return Ok(None);
        }

        let mut out: Vec<String> = vec![];
        for bat in &self.batteries {
            let cap = std::fs::read_to_string(bat.join("capacity"))
                .ok()
                .and_then(|v| v.trim().replace('$', "").parse::<i32>().ok());
            let Some(cap) = cap else { continue };
            let sep = if cap < 25 { "❗" } else { " " };

            let current = std::fs::read_to_string(bat.join("current_now"))
                .ok()
                .and_then(|v| v.trim().parse::<f64>().ok());
            let voltage = std::fs::read_to_string(bat.join("current_now"))
                .ok()
                .and_then(|v| v.trim().parse::<f64>().ok());
            let mut watt = if let (Some(current), Some(voltage)) = (current, voltage) {
                Some((current * voltage) / 1_000_000_000_000.0)
            } else {
                None
            };

            let status = match std::fs::read_to_string(bat.join("status"))?
                .trim()
                .replace(',', "")
                .as_str()
            {
                "Discharging" => "🔋".into(),
                "Charging" | "Not charging" => {
                    watt = None; // dont show watt if we are currently charging
                    "🔌".into()
                }
                "Unknown" => "♻️".into(),
                "Full" => "⚡".into(),
                o => o.to_string(),
            };
            if let Some(watt) = watt {
                out.push(format!("{status}{sep}{cap}% ({watt:.2}W)"));
            } else {
                out.push(format!("{status}{sep}{cap}%"));
            }
        }

        if out.is_empty() {
            Ok(None)
        } else {
            Ok(Some(out.join(" | ")))
        }
    }
}
