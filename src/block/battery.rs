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

            let status = match std::fs::read_to_string(bat.join("status"))?
                .trim()
                .replace(',', "")
                .as_str()
            {
                "Discharging" => "🔋".into(),
                "Not Charging" => "🛑".into(),
                "Charging" => "🔌".into(),
                "Unknown" => "♻️".into(),
                "Full" => "⚡".into(),
                o => o.to_string(),
            };
            out.push(format!("{status}{sep}{cap}%"));
        }

        if out.is_empty() {
            Ok(None)
        } else {
            Ok(Some(out.join(" | ")))
        }
    }
}
