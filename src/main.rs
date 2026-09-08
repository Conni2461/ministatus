#![cfg_attr(
    test,
    allow(
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic,
        clippy::unwrap_used
    )
)]

use std::collections::HashMap;
use std::time::Instant;

use crate::block::{Block, Options, TICK};
use crate::config::Config;

mod block;
mod config;
mod xorg;

struct Slot {
    name: &'static str,
    block: Box<dyn Block>,
    prev: Option<String>,
    opts: Options,
    due: Instant,
}

fn rebuild(cfg: &Config, live: Vec<Slot>, home: &str) -> Vec<Slot> {
    let mut old: HashMap<&'static str, Slot> = live.into_iter().map(|s| (s.name, s)).collect();
    let now = Instant::now();

    cfg.blocks
        .iter()
        .filter_map(|name| {
            let Some(name) = block::canonical(name) else {
                eprintln!("unknown block {name:?} in config, skipping");
                return None;
            };
            if let Some(mut slot) = old.remove(name) {
                slot.opts = cfg.options(name);
                slot.due = now;
                return Some(slot);
            }
            match block::build(name, home)? {
                Ok(block) => Some(Slot {
                    name,
                    block,
                    prev: None,
                    opts: cfg.options(name),
                    due: now,
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

    let mut shown: Option<String> = None;
    let mut next_tick = Instant::now() + TICK;

    loop {
        let start = Instant::now();

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

        for slot in &mut blocks {
            if slot.due > start {
                continue;
            }
            slot.due = start + slot.block.interval();
            if let Ok(v) = slot.block.run(&slot.opts) {
                slot.prev = v;
            }
        }

        let text = blocks
            .iter()
            .filter_map(|s| s.prev.as_deref())
            .collect::<Vec<_>>()
            .join(&cfg.separator);

        if debug {
            eprintln!("Elapsed: {:.2?}", start.elapsed());
        }

        if shown.as_deref() != Some(text.as_str()) {
            match &window {
                Some(w) => {
                    if let Err(e) = w.set_title(&text) {
                        eprintln!("failed to write to window: {e}");
                    }
                }
                None => println!("{text}"),
            }
            shown = Some(text);
        }

        let now = Instant::now();
        if next_tick <= now {
            next_tick = now + TICK;
        } else {
            std::thread::sleep(next_tick - now);
            next_tick += TICK;
        }
    }
}
