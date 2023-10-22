use std::collections::HashMap;

use futures::future::BoxFuture;
use tokio::process::Command;
use tokio::signal::unix::{signal, SignalKind};
use tokio::time::{sleep, Duration};

mod xorg;

type Output = Result<Option<String>, anyhow::Error>;
type Module = fn() -> BoxFuture<'static, Output>;

async fn clock() -> Output {
    Ok(Some(format!(
        "🕛 {}",
        chrono::offset::Local::now().format("%m/%d/%Y %I:%M %p")
    )))
}

async fn volume() -> Output {
    let v = String::from_utf8(Command::new("volume").output().await?.stdout)?
        .trim()
        .to_string();
    if v.is_empty() {
        Ok(None)
    } else {
        Ok(Some(v))
    }
}

async fn internet() -> Output {
    let i = String::from_utf8(Command::new("internet").output().await?.stdout)?
        .trim()
        .to_string();
    if i.is_empty() {
        Ok(None)
    } else {
        Ok(Some(i))
    }
}

async fn weather() -> Output {
    let w = String::from_utf8(Command::new("weather").output().await?.stdout)?
        .trim()
        .to_string();
    if w.is_empty() {
        Ok(None)
    } else {
        Ok(Some(w))
    }
}

async fn mailbox() -> Output {
    let mb = String::from_utf8(Command::new("mailbox").output().await?.stdout)?
        .trim()
        .to_string();
    if mb.is_empty() {
        Ok(None)
    } else {
        Ok(Some(format!("📬 {mb}")))
    }
}

async fn news() -> Output {
    let news = String::from_utf8(Command::new("news").output().await?.stdout)?
        .trim()
        .to_string();
    if news.is_empty() {
        Ok(None)
    } else {
        Ok(Some(format!("📰 {news}")))
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), anyhow::Error> {
    let window = xorg::Window::new();
    let mut signal_recv = signal(SignalKind::user_defined1())?;
    let modules: Vec<Module> = vec![
        || Box::pin(news()),
        || Box::pin(mailbox()),
        || Box::pin(weather()),
        || Box::pin(internet()),
        || Box::pin(volume()),
        || Box::pin(clock()),
    ];
    let mut prev_state: HashMap<usize, String> = HashMap::new();

    loop {
        let mut out: Vec<String> = vec![];
        for (i, m) in modules.iter().enumerate() {
            match m().await {
                Ok(Some(v)) => {
                    out.push(v.clone());
                    prev_state.insert(i, v);
                }
                _ => {
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
