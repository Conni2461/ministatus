use std::collections::HashMap;

use tokio::signal::unix::{signal, SignalKind};
use tokio::time::{sleep, Duration};

use crate::block::Block;

mod block;
mod shared;
mod xorg;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), anyhow::Error> {
    let window = xorg::Window::new();
    let mut signal_recv = signal(SignalKind::user_defined1())?;
    let home = std::env::var("HOME")?;

    let mut blocks: Vec<Box<dyn Block>> = Vec::new();
    match block::News::new(&home) {
        Ok(v) => blocks.push(v),
        Err(e) => eprintln!("news disabled because of {e}"),
    }
    match block::Mailbox::new(&home) {
        Ok(v) => blocks.push(v),
        Err(e) => eprintln!("mailbox disabled because of {e}"),
    }
    blocks.push(block::Weather::new().await?);
    blocks.push(block::Internet::new());
    blocks.push(block::Battery::new());
    blocks.push(block::Volume::new()?);
    blocks.push(block::Clock::new());

    let mut prev_state: HashMap<usize, String> = HashMap::new();
    let debug = std::env::var("DEBUG").map(|v| v == "1").unwrap_or_default();

    let pulse = block::Pulse::new()?;

    loop {
        let now = std::time::Instant::now();
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
        println!("{:?}", pulse.run());
        eprintln!("Elapsed: {:.2?}", now.elapsed());

        if !debug {
            let _ = window.set_title(&out.join(" | "));
        } else {
            println!("{}", &out.join(" | "));
        }
        tokio::select! {
            _ = signal_recv.recv() => (),
            _ = sleep(Duration::from_secs(10)) => (),
        }
    }
}
