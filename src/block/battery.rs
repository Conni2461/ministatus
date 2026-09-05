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
                    .is_ok_and(|x| x.starts_with("BAT"))
                {
                    batteries.push(ps.path());
                }
            }
        }

        Self { batteries }
    }
}

fn render(status: &str, cap: i32, watt: Option<f64>) -> String {
    let sep = if cap < 25 { "❗" } else { " " };
    match watt {
        Some(watt) => format!("{status}{sep}{cap}% ({watt:.2}W)"),
        None => format!("{status}{sep}{cap}%"),
    }
}

impl super::Block for Battery {
    fn run(&self, opts: super::Options) -> Result<Option<String>, anyhow::Error> {
        if self.batteries.is_empty() {
            return Ok(None);
        }

        let mut out: Vec<String> = vec![];
        for bat in &self.batteries {
            let cap = std::fs::read_to_string(bat.join("capacity"))
                .ok()
                .and_then(|v| v.trim().replace('$', "").parse::<i32>().ok());
            let Some(cap) = cap else { continue };

            let current = std::fs::read_to_string(bat.join("current_now"))
                .ok()
                .and_then(|v| v.trim().parse::<f64>().ok());
            let voltage = std::fs::read_to_string(bat.join("voltage_now"))
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
            if opts.compact {
                watt = None;
            }
            out.push(render(&status, cap, watt));
        }

        if out.is_empty() {
            Ok(None)
        } else {
            Ok(Some(out.join(" | ")))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::render;

    #[test]
    fn full_appends_the_wattage() {
        assert_eq!(render("🔋", 87, Some(12.34)), "🔋 87% (12.34W)");
    }

    #[test]
    fn compact_drops_the_wattage() {
        assert_eq!(render("🔋", 87, None), "🔋 87%");
    }

    #[test]
    fn a_low_charge_is_marked_in_both_modes() {
        assert_eq!(render("🔋", 12, Some(12.34)), "🔋❗12% (12.34W)");
        assert_eq!(render("🔋", 12, None), "🔋❗12%");
    }
}
