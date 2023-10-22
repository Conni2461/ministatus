mod clock;
mod volume;
mod internet;
mod weather;
mod mailbox;
mod news;

pub use clock::Clock;
pub use volume::Volume;
pub use internet::Internet;
pub use weather::Weather;
pub use mailbox::Mailbox;
pub use news::News;

#[async_trait::async_trait]
pub trait Block {
    async fn run(&self) -> Result<Option<String>, anyhow::Error>;
}
