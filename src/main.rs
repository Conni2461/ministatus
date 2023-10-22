use std::collections::HashMap;

use futures::future::BoxFuture;
use tokio::process::Command;
use tokio::signal::unix::{signal, SignalKind};
use tokio::time::{sleep, Duration};

mod xorg;

type Module = fn() -> BoxFuture<'static, Result<String, anyhow::Error>>;

async fn clock() -> Result<String, anyhow::Error> {
    Ok(format!(
        "🕛 {}",
        chrono::offset::Local::now().format("%m/%d/%Y %I:%M %p")
    ))
}

async fn volume() -> Result<String, anyhow::Error> {
    Ok(
        String::from_utf8(Command::new("volume").output().await?.stdout)?
            .trim()
            .to_string(),
    )
}

async fn internet() -> Result<String, anyhow::Error> {
    Ok(
        String::from_utf8(Command::new("internet").output().await?.stdout)?
            .trim()
            .to_string(),
    )
}

async fn weather() -> Result<String, anyhow::Error> {
    let w = String::from_utf8(Command::new("weather").output().await?.stdout)?
        .trim()
        .to_string();
    Ok(w)
}

async fn mailbox() -> Result<String, anyhow::Error> {
    let mb = String::from_utf8(Command::new("mailbox").output().await?.stdout)?
        .trim()
        .to_string();
    Ok(format!("📰 {mb}"))
}

async fn news() -> Result<String, anyhow::Error> {
    let news = String::from_utf8(Command::new("news").output().await?.stdout)?
        .trim()
        .to_string();
    Ok(format!("📬 {news}"))
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), anyhow::Error> {
    let window = xorg::Window::new();
    let mut signal_recv = signal(SignalKind::user_defined1())?;
    let modules: Vec<Module> = vec![
        || Box::pin(mailbox()),
        || Box::pin(news()),
        || Box::pin(weather()),
        || Box::pin(internet()),
        || Box::pin(volume()),
        || Box::pin(clock()),
    ];
    let mut prev_state: HashMap<usize, String> = HashMap::new();

    loop {
        let mut out: Vec<String> = vec![];
        for (i, m) in modules.iter().enumerate() {
            if let Ok(v) = m().await {
                out.push(v.clone());
                prev_state.insert(i, v);
            } else {
                if let Some(v) = prev_state.get(&i) {
                    out.push(v.clone());
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
