#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use std::collections::HashMap;

use crate::block::Block;

mod block;
mod shared;
mod xorg;

fn main() -> Result<(), anyhow::Error> {
    let window = xorg::Window::new();
    let home = std::env::var("HOME")?;

    let mut blocks: Vec<Box<dyn Block>> = Vec::new();
    match block::News::new(&home) {
        Ok(v) => blocks.push(Box::new(v)),
        Err(e) => eprintln!("news disabled because of {e}"),
    }
    match block::Mailbox::new(&home) {
        Ok(v) => blocks.push(Box::new(v)),
        Err(e) => eprintln!("mailbox disabled because of {e}"),
    }
    blocks.push(Box::new(block::Weather::new()));
    blocks.push(Box::new(block::Internet::new()));
    blocks.push(Box::new(block::Battery::new()));
    blocks.push(Box::new(block::Pulse::new()?));
    blocks.push(Box::new(block::Clock::new()));

    let mut prev_state: HashMap<usize, String> = HashMap::new();
    let debug = std::env::var("DEBUG").is_ok_and(|v| v == "1");

    loop {
        let now = std::time::Instant::now();
        let mut out: Vec<String> = vec![];
        for (i, m) in blocks.iter().enumerate() {
            match m.run() {
                Ok(Some(v)) => {
                    out.push(v.clone());
                    prev_state.insert(i, v);
                }
                Ok(None) => (), // if we have a None Value we dont wanna show this block
                Err(_) => {
                    // If we have a Error we check the previous state for a value
                    if let Some(v) = prev_state.get(&i) {
                        out.push(v.clone());
                    }
                }
            }
        }
        let text = out.join(" | ");
        eprintln!("Elapsed: {:.2?}", now.elapsed());
        if debug {
            println!("{}", &text);
        } else if let Err(e) = window.set_title(&text) {
            eprintln!("failed to write to window: {e}");
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
