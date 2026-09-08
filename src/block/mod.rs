use std::io::Read;
use std::path::Path;
use std::time::Duration;

mod battery;
mod clock;
mod cpu;
mod disk;
mod internet;
mod memory;
mod news;
mod pulse;
mod weather;

pub use battery::Battery;
pub use clock::Clock;
pub use cpu::Cpu;
pub use disk::Disk;
pub use internet::Internet;
pub use memory::Memory;
pub use news::News;
pub use pulse::Pulse;
pub use weather::Weather;

pub const TICK: Duration = Duration::from_secs(1);

/// GiB from kibibytes; meminfo counts kB, statvfs counts bytes (divide by 1024).
#[allow(clippy::cast_precision_loss)]
fn gib(kib: u64) -> f64 {
    kib as f64 / 1_048_576.0
}

#[derive(Debug, Default)]
pub struct Options {
    pub compact: bool,
    /// Filesystem to report on (disk block only); falls back to its default.
    pub target: Option<String>,
}

pub trait Block {
    fn run(&mut self, opts: &Options) -> Result<Option<String>, anyhow::Error>;

    fn interval(&self) -> Duration {
        TICK
    }
}

fn read_into<'a>(path: &Path, buf: &'a mut Vec<u8>) -> Result<&'a str, anyhow::Error> {
    buf.clear();
    std::fs::File::open(path)?.read_to_end(buf)?;
    Ok(std::str::from_utf8(buf)?)
}

pub const ALL: [&str; 9] = [
    "news", "weather", "internet", "cpu", "memory", "disk", "battery", "pulse", "clock",
];

pub fn canonical(name: &str) -> Option<&'static str> {
    ALL.into_iter().find(|v| *v == name)
}

pub fn build(name: &str, home: &str) -> Option<Result<Box<dyn Block>, anyhow::Error>> {
    let block: Result<Box<dyn Block>, anyhow::Error> = match canonical(name)? {
        "news" => News::new(home).map(|v| Box::new(v) as Box<dyn Block>),
        "weather" => Ok(Box::new(Weather::new())),
        "internet" => Ok(Box::new(Internet::new())),
        "cpu" => Ok(Box::new(Cpu::new())),
        "memory" => Ok(Box::new(Memory::new())),
        "disk" => Ok(Box::new(Disk::new())),
        "battery" => Ok(Box::new(Battery::new())),
        "pulse" => Pulse::new().map(|v| Box::new(v) as Box<dyn Block>),
        "clock" => Ok(Box::new(Clock::new())),
        _ => unreachable!("canonical only returns names listed in ALL"),
    };
    Some(block)
}
