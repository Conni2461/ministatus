use std::collections::HashMap;

use tokio::signal::unix::{signal, SignalKind};
use tokio::time::{sleep, Duration};

use crate::block::Block;

mod block;
mod xorg;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), anyhow::Error> {
    let window = xorg::Window::new();
    let mut signal_recv = signal(SignalKind::user_defined1())?;
    let blocks: Vec<Box<dyn Block>> = vec![
        block::News::new(),
        block::Mailbox::new()?,
        block::Weather::new(),
        block::Internet::new(),
        block::Volume::new(),
        block::Clock::new(),
    ];
    let mut prev_state: HashMap<usize, String> = HashMap::new();

    loop {
        let mut out: Vec<String> = vec![];
        for (i, m) in blocks.iter().enumerate() {
            match m.run().await {
                Ok(Some(v)) => {
                    out.push(v.clone());
                    prev_state.insert(i, v);
                }
                Ok(None) => continue, // if we have a None Value we dont wanna show this block
                Err(_) => {
                    // If we have a Error we check the previous state for a value
                    if let Some(v) = prev_state.get(&i) {
                        out.push(v.clone());
                    }
                }
            }
        }

        let _ = window.set_title(&out.join(" | "));
        tokio::select! {
            _ = signal_recv.recv() => (),
            _ = sleep(Duration::from_secs(15)) => (),
        }
    }
}
