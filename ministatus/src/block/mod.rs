mod clock;
mod internet;
mod mailbox;
mod news;
mod volume;
mod weather;

pub use clock::Clock;
pub use internet::Internet;
pub use mailbox::Mailbox;
pub use news::News;
pub use volume::Volume;
pub use weather::Weather;

#[async_trait::async_trait]
pub trait Block {
    async fn run(&self) -> Result<Option<String>, anyhow::Error>;
}
