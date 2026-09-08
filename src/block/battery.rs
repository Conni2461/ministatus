use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::time::Duration;

struct Paths {
    capacity: PathBuf,
    current: PathBuf,
    voltage: PathBuf,
    status: PathBuf,
}

impl Paths {
    fn new(base: &Path) -> Self {
        Self {
            capacity: base.join("capacity"),
            current: base.join("current_now"),
            voltage: base.join("voltage_now"),
            status: base.join("status"),
        }
    }
}

pub struct Battery {
    batteries: Vec<Paths>,
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
                    batteries.push(Paths::new(&ps.path()));
                }
            }
        }

        Self { batteries }
    }
}

fn read_number<T: std::str::FromStr>(path: &Path) -> Option<T> {
    std::fs::read_to_string(path).ok()?.trim().parse().ok()
}

fn icon(status: &str) -> Cow<'static, str> {
    match status {
        "Discharging" => "🔋".into(),
        "Charging" | "Not charging" => "🔌".into(),
        "Unknown" => "♻️".into(),
        "Full" => "⚡".into(),
        o => Cow::Owned(o.to_owned()),
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
    fn run(&mut self, opts: &super::Options) -> Result<Option<String>, anyhow::Error> {
        if self.batteries.is_empty() {
            return Ok(None);
        }

        let mut out: Vec<String> = vec![];
        for bat in &self.batteries {
            let Some(cap) = read_number::<i32>(&bat.capacity) else {
                continue;
            };

            let status = std::fs::read_to_string(&bat.status)?;
            let status = status.trim();

            let watt = if opts.compact || matches!(status, "Charging" | "Not charging") {
                None
            } else {
                match (
                    read_number::<f64>(&bat.current),
                    read_number::<f64>(&bat.voltage),
                ) {
                    (Some(current), Some(voltage)) => Some((current * voltage) / 1e12),
                    _ => None,
                }
            };

            out.push(render(&icon(status), cap, watt));
        }

        if out.is_empty() {
            Ok(None)
        } else {
            Ok(Some(out.join(" | ")))
        }
    }

    fn interval(&self) -> Duration {
        Duration::from_secs(5)
    }
}

#[cfg(test)]
mod tests {
    use super::{icon, render};

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

    #[test]
    fn each_status_maps_to_its_own_icon() {
        assert_eq!(icon("Discharging"), "🔋");
        assert_eq!(icon("Charging"), "🔌");
        assert_eq!(icon("Not charging"), "🔌");
        assert_eq!(icon("Unknown"), "♻️");
        assert_eq!(icon("Full"), "⚡");
    }

    #[test]
    fn an_unrecognised_status_is_passed_through() {
        assert_eq!(icon("Weird"), "Weird");
    }
}
