#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use std::collections::HashMap;

use crate::block::Block;
use crate::config::Config;

mod block;
mod config;
mod shared;
mod xorg;

struct Slot {
    name: &'static str,
    block: Box<dyn Block>,
    prev: Option<String>,
}

fn rebuild(cfg: &Config, live: Vec<Slot>, home: &str) -> Vec<Slot> {
    let mut old: HashMap<&'static str, Slot> = live.into_iter().map(|s| (s.name, s)).collect();

    cfg.blocks
        .iter()
        .filter_map(|name| {
            let Some(name) = block::canonical(name) else {
                eprintln!("unknown block {name:?} in config, skipping");
                return None;
            };
            if let Some(slot) = old.remove(name) {
                return Some(slot);
            }
            match block::build(name, home)? {
                Ok(block) => Some(Slot {
                    name,
                    block,
                    prev: None,
                }),
                Err(e) => {
                    eprintln!("{name} disabled because of {e}");
                    None
                }
            }
        })
        .collect()
}

fn main() -> Result<(), anyhow::Error> {
    let home = std::env::var("HOME")?;
    let debug = std::env::var("DEBUG").is_ok_and(|v| v == "1");
    let window = (!debug).then(xorg::Window::new);

    let path = Config::path(&home);
    let mut seen = config::mtime(&path);
    let mut cfg = Config::load(&path).unwrap_or_else(|e| {
        eprintln!("failed to read {}: {e}", path.display());
        Config::default()
    });
    let mut blocks = rebuild(&cfg, Vec::new(), &home);

    loop {
        let now = std::time::Instant::now();

        let mtime = config::mtime(&path);
        if mtime != seen {
            seen = mtime;
            match Config::load(&path) {
                Ok(v) => {
                    cfg = v;
                    blocks = rebuild(&cfg, blocks, &home);
                }
                Err(e) => eprintln!("failed to read {}: {e}", path.display()),
            }
        }

        let mut out: Vec<String> = vec![];
        for slot in &mut blocks {
            match slot.block.run() {
                Ok(Some(v)) => {
                    out.push(v.clone());
                    slot.prev = Some(v);
                }
                Ok(None) => (),
                Err(_) => {
                    if let Some(v) = &slot.prev {
                        out.push(v.clone());
                    }
                }
            }
        }
        let text = out.join(&cfg.separator);
        eprintln!("Elapsed: {:.2?}", now.elapsed());
        match &window {
            Some(w) => {
                if let Err(e) = w.set_title(&text) {
                    eprintln!("failed to write to window: {e}");
                }
            }
            None => println!("{}", &text),
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
