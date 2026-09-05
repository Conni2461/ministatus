use std::path::Path;
use std::time::Duration;

use super::read_into;

const WIRELESS: &str = "/proc/net/wireless";

pub struct Internet {
    wireless: Vec<u8>,
    operstate: Vec<u8>,
    path: String,
}

impl Internet {
    pub const fn new() -> Self {
        Self {
            wireless: Vec::new(),
            operstate: Vec::new(),
            path: String::new(),
        }
    }
}

fn parse_wireless(s: &str) -> Option<(&str, f32)> {
    let line = s
        .lines()
        .map(str::trim_start)
        .find(|l| l.starts_with('w'))?;
    let mut fields = line.split_whitespace();

    let id = fields.next()?.trim_end_matches(':');
    let quality = fields.nth(1)?.parse::<f32>().ok()?;
    Some((id, quality))
}

#[allow(clippy::cast_possible_truncation)]
fn render(up: bool, quality: f32) -> String {
    let icon = if up { "🌍" } else { "❎" };
    let val = (quality * 100.0 / 70.0) as i32;
    format!("{icon} {val}%")
}

impl super::Block for Internet {
    fn run(&mut self, _: super::Options) -> Result<Option<String>, anyhow::Error> {
        let s = read_into(Path::new(WIRELESS), &mut self.wireless)?;
        let Some((id, quality)) = parse_wireless(s) else {
            return Ok(None);
        };

        self.path.clear();
        self.path.push_str("/sys/class/net/");
        self.path.push_str(id);
        self.path.push_str("/operstate");

        let up = read_into(Path::new(&self.path), &mut self.operstate)?
            .lines()
            .next()
            == Some("up");

        Ok(Some(render(up, quality)))
    }

    fn interval(&self) -> Duration {
        Duration::from_secs(5)
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_wireless, render};

    const SAMPLE: &str = concat!(
        "Inter-| sta-|   Quality        |   Discarded packets               | Missed | WE\n",
        " face | tus | link level noise |  nwid  crypt   frag  retry   misc | beacon | 22\n",
        " wlan0: 0000   58.  -52.  -256        0      0      0      0     12        0\n",
    );

    #[test]
    fn wireless_reads_the_name_and_link_quality() {
        let (id, quality) = parse_wireless(SAMPLE).unwrap();
        assert_eq!(id, "wlan0");
        assert!((quality - 58.0).abs() < f32::EPSILON);
    }

    #[test]
    fn wireless_reads_an_unpadded_long_interface_name() {
        let s = SAMPLE.replace(" wlan0:", "wlp3s0:");
        let (id, quality) = parse_wireless(&s).unwrap();
        assert_eq!(id, "wlp3s0");
        assert!((quality - 58.0).abs() < f32::EPSILON);
    }

    #[test]
    fn wireless_rejects_a_header_only_file() {
        let header = SAMPLE.lines().take(2).collect::<Vec<_>>().join("\n");
        assert!(parse_wireless(&header).is_none());
    }

    #[test]
    fn an_interface_that_is_down_is_marked() {
        assert_eq!(render(true, 58.0), "🌍 82%");
        assert_eq!(render(false, 58.0), "❎ 82%");
    }
}
