mod battery;
mod clock;
mod internet;
mod mailbox;
mod news;
mod pulse;
mod weather;

pub use battery::Battery;
pub use clock::Clock;
pub use internet::Internet;
pub use mailbox::Mailbox;
pub use news::News;
pub use pulse::Pulse;
pub use weather::Weather;

pub trait Block {
    fn run(&self) -> Result<Option<String>, anyhow::Error>;
}
