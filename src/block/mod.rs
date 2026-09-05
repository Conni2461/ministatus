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

pub trait Block {
    fn run(&self) -> Result<Option<String>, anyhow::Error>;
}
