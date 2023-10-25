mod battery;
mod clock;
mod internet;
mod mailbox;
mod news;
mod pulse;
mod volume;
mod weather;

pub use battery::Battery;
pub use clock::Clock;
pub use internet::Internet;
pub use mailbox::Mailbox;
pub use news::News;
pub use pulse::Pulse;
pub use volume::Volume;
pub use weather::Weather;

pub trait Block {
    fn run(&self) -> Result<Option<String>, anyhow::Error>;
}

fn file_as_vec_str(p: &str) -> Result<Vec<String>, anyhow::Error> {
    let contents = std::fs::read_to_string(p)?;
    Ok(contents.split('\n').map(ToOwned::to_owned).collect())
}
