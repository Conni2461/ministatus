use std::sync::{mpsc, Arc, RwLock};

use libpulse_binding::{
    callbacks::ListResult,
    context::{
        introspect::SinkInfo,
        subscribe::{Facility, InterestMaskSet},
        Context, FlagSet, State,
    },
    mainloop::threaded::Mainloop,
    proplist::{properties, Proplist},
    volume::Volume,
};

use crate::shared::Shared;

#[derive(Debug)]
struct TxState {
    pub volume: i32,
    pub mute: bool,
}

pub struct Pulse {
    mainloop: Shared<Mainloop>,
    context: Shared<Context>,

    state: Arc<RwLock<TxState>>,
}

impl Pulse {
    pub fn new() -> Result<Box<Self>, anyhow::Error> {
        let mut proplist = Proplist::new().ok_or(anyhow::anyhow!("Failed to init Proplist"))?;
        proplist
            .set_str(properties::APPLICATION_NAME, "ministatus")
            .map_err(|_| anyhow::anyhow!("Failed to set APPLICATION_NAME"))?;

        let mainloop =
            Shared::new(Mainloop::new().ok_or(anyhow::anyhow!("Failed to init Mainloop"))?);

        let context = Shared::new(
            Context::new_with_proplist(&*mainloop.borrow(), "ministatus context", &proplist)
                .ok_or(anyhow::anyhow!("Failed to init Context"))?,
        );

        let s = Self {
            mainloop,
            context,
            state: Arc::new(RwLock::new(TxState {
                volume: 0,
                mute: false,
            })),
        };
        s.connect()?;

        Ok(Box::new(s))
    }

    fn connect(&self) -> Result<(), anyhow::Error> {
        let mut mainloop = self.mainloop.borrow_mut();
        let mut ctx = self.context.borrow_mut();

        let mainloop_shr_ref = self.mainloop.clone_rc();
        let ctx_shr_ref = self.context.clone_rc();

        ctx.set_state_callback(Some(Box::new(move || {
            match unsafe { (*ctx_shr_ref.as_ptr()).get_state() } {
                State::Ready | State::Failed | State::Terminated => unsafe {
                    (*mainloop_shr_ref.as_ptr()).signal(false);
                },
                _ => {}
            }
        })));

        ctx.connect(None, FlagSet::NOFLAGS, None)?;

        mainloop.lock();
        mainloop.start()?;

        loop {
            match ctx.get_state() {
                State::Ready => {
                    ctx.set_state_callback(None);
                    mainloop.unlock();
                    break;
                }
                State::Failed | State::Terminated => {
                    eprintln!("Context state failed/terminated, quitting...");
                    mainloop.unlock();
                    mainloop.stop();
                    panic!("Pulse session terminated.");
                }
                _ => {
                    mainloop.wait();
                }
            }
        }

        drop(ctx);
        drop(mainloop);

        self.subscribe();

        Ok(())
    }

    fn subscribe(&self) {
        fn tx_sink(tx: &mpsc::Sender<TxState>, result: ListResult<&SinkInfo<'_>>) {
            if let ListResult::Item(item) = result {
                if item.state.is_running() {
                    tx.send(TxState {
                        volume: ((item.volume.avg().0 as f32 / Volume::NORMAL.0 as f32) * 100.)
                            as i32,
                        mute: item.mute,
                    })
                    .unwrap();
                }
            };
        }

        let mut mainloop = self.mainloop.borrow_mut();
        let mut ctx = self.context.borrow_mut();
        mainloop.lock();

        let introspect = ctx.introspect();

        let (tx, rx) = mpsc::channel::<TxState>();

        let tx2 = tx.clone();
        introspect.get_sink_info_list(move |res| tx_sink(&tx2, res));

        ctx.subscribe(InterestMaskSet::SERVER | InterestMaskSet::SINK, |_| ());
        ctx.set_subscribe_callback(Some(Box::new(move |fac, _, index| {
            let tx = tx.clone();

            match fac {
                // TODO
                // Some(Facility::Server) => {
                //     introspect.get_server_info(move |res| tx_server(&sc_tx, res));
                // }
                Some(Facility::Sink) => {
                    introspect.get_sink_info_by_index(index, move |res| tx_sink(&tx, res));
                }
                _ => (),
            };
        })));

        mainloop.unlock();

        let state = self.state.clone();
        std::thread::spawn(move || loop {
            let state = state.clone();
            if let Ok(o) = rx.recv() {
                if let Ok(mut w) = state.write() {
                    *w = o;
                }
            }
        });
    }

    fn cleanup(&self) {
        let mut ctx = self.context.borrow_mut();
        let mut mainloop = self.mainloop.borrow_mut();

        ctx.disconnect();
        mainloop.stop();
    }
}

impl Drop for Pulse {
    fn drop(&mut self) {
        self.cleanup();
    }
}

impl super::Block for Pulse {
    fn run(&self) -> Result<Option<String>, anyhow::Error> {
        let r = self.state.read().unwrap();
        if r.mute {
            return Ok(Some("🔇".into()));
        }
        let symbol = if r.volume > 70 {
            "🔊"
        } else if r.volume > 30 {
            "🔉"
        } else {
            "🔈"
        };
        Ok(Some(format!("{symbol} {}%", r.volume)))
    }
}
