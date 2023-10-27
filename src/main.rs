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
        Ok(v) => blocks.push(v),
        Err(e) => eprintln!("news disabled because of {e}"),
    }
    match block::Mailbox::new(&home) {
        Ok(v) => blocks.push(v),
        Err(e) => eprintln!("mailbox disabled because of {e}"),
    }
    blocks.push(block::Weather::new()?);
    blocks.push(block::Internet::new());
    blocks.push(block::Battery::new());
    blocks.push(block::Pulse::new()?);
    blocks.push(block::Clock::new());

    let mut prev_state: HashMap<usize, String> = HashMap::new();
    let debug = std::env::var("DEBUG").map(|v| v == "1").unwrap_or_default();

    loop {
        let now = std::time::Instant::now();
        let mut out: Vec<String> = vec![];
        for (i, m) in blocks.iter().enumerate() {
            match m.run() {
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
        eprintln!("Elapsed: {:.2?}", now.elapsed());

        if !debug {
            let _ = window.set_title(&out.join(" | "));
        } else {
            println!("{}", &out.join(" | "));
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
