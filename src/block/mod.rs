use std::io::Read;
use std::path::Path;
use std::time::Duration;

mod battery;
mod clock;
mod cpu;
mod internet;
mod mailbox;
mod memory;
mod news;
mod pulse;
mod weather;

pub use battery::Battery;
pub use clock::Clock;
pub use cpu::Cpu;
pub use internet::Internet;
pub use mailbox::Mailbox;
pub use memory::Memory;
pub use news::News;
pub use pulse::Pulse;
pub use weather::Weather;

pub const TICK: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, Copy, Default)]
pub struct Options {
    pub compact: bool,
}

pub trait Block {
    fn run(&mut self, opts: Options) -> Result<Option<String>, anyhow::Error>;

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
    "news", "mailbox", "weather", "internet", "cpu", "memory", "battery", "pulse", "clock",
];

pub fn canonical(name: &str) -> Option<&'static str> {
    ALL.into_iter().find(|v| *v == name)
}

pub fn build(name: &str, home: &str) -> Option<Result<Box<dyn Block>, anyhow::Error>> {
    let block: Result<Box<dyn Block>, anyhow::Error> = match canonical(name)? {
        "news" => News::new(home).map(|v| Box::new(v) as Box<dyn Block>),
        "mailbox" => Mailbox::new(home).map(|v| Box::new(v) as Box<dyn Block>),
        "weather" => Ok(Box::new(Weather::new())),
        "internet" => Ok(Box::new(Internet::new())),
        "cpu" => Ok(Box::new(Cpu::new())),
        "memory" => Ok(Box::new(Memory::new())),
        "battery" => Ok(Box::new(Battery::new())),
        "pulse" => Pulse::new().map(|v| Box::new(v) as Box<dyn Block>),
        "clock" => Ok(Box::new(Clock::new())),
        _ => unreachable!("canonical only returns names listed in ALL"),
    };
    Some(block)
}
