use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use libpulse_binding::{
    callbacks::ListResult,
    context::{
        Context, FlagSet, State,
        introspect::{Introspector, SinkInfo},
        subscribe::{Facility, InterestMaskSet, Operation},
    },
    mainloop::threaded::Mainloop,
    proplist::{Proplist, properties},
    volume::Volume,
};

const DEFAULT_SINK: &str = "@DEFAULT_SINK@";

const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
const CONNECT_POLL: Duration = Duration::from_millis(10);

#[derive(Debug, Default)]
struct SinkState {
    volume: u32,
    mute: bool,
}

pub struct Pulse {
    context: Context,
    mainloop: Mainloop,

    state: Arc<RwLock<SinkState>>,
}

fn store_sink(state: &RwLock<SinkState>, result: &ListResult<&SinkInfo<'_>>) {
    let ListResult::Item(item) = result else {
        return;
    };

    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    let volume = ((item.volume.avg().0 as f32 / Volume::NORMAL.0 as f32) * 100.).round() as u32;

    if let Ok(mut w) = state.write() {
        *w = SinkState {
            volume,
            mute: item.mute,
        };
    }
}

fn query_default_sink(introspect: &Introspector, state: &Arc<RwLock<SinkState>>) {
    let state = state.clone();
    introspect.get_sink_info_by_name(DEFAULT_SINK, move |res| store_sink(&state, &res));
}

impl Pulse {
    pub fn new() -> Result<Self, anyhow::Error> {
        let mut proplist =
            Proplist::new().ok_or_else(|| anyhow::anyhow!("Failed to init Proplist"))?;
        proplist
            .set_str(properties::APPLICATION_NAME, "ministatus")
            .map_err(|()| anyhow::anyhow!("Failed to set APPLICATION_NAME"))?;

        let mainloop = Mainloop::new().ok_or_else(|| anyhow::anyhow!("Failed to init Mainloop"))?;
        let context = Context::new_with_proplist(&mainloop, "ministatus context", &proplist)
            .ok_or_else(|| anyhow::anyhow!("Failed to init Context"))?;

        let mut s = Self {
            context,
            mainloop,
            state: Arc::new(RwLock::new(SinkState::default())),
        };
        s.connect()?;
        s.subscribe();

        Ok(s)
    }

    fn connect(&mut self) -> Result<(), anyhow::Error> {
        self.context.connect(None, FlagSet::NOFLAGS, None)?;

        self.mainloop.lock();
        if let Err(e) = self.mainloop.start() {
            self.mainloop.unlock();
            return Err(e.into());
        }

        let deadline = Instant::now() + CONNECT_TIMEOUT;
        let outcome = loop {
            match self.context.get_state() {
                State::Ready => break Ok(()),
                State::Failed | State::Terminated => {
                    break Err(anyhow::anyhow!("Pulse session terminated"));
                }
                _ if Instant::now() >= deadline => {
                    break Err(anyhow::anyhow!(
                        "Pulse server did not answer within {CONNECT_TIMEOUT:?}"
                    ));
                }
                _ => {
                    self.mainloop.unlock();
                    std::thread::sleep(CONNECT_POLL);
                    self.mainloop.lock();
                }
            }
        };
        self.mainloop.unlock();
        outcome
    }

    fn subscribe(&mut self) {
        self.mainloop.lock();

        let introspect = self.context.introspect();
        query_default_sink(&introspect, &self.state);

        let state = self.state.clone();
        self.context
            .subscribe(InterestMaskSet::SERVER | InterestMaskSet::SINK, |_| ());
        self.context
            .set_subscribe_callback(Some(Box::new(move |fac, op, _| {
                if op == Some(Operation::Changed)
                    && matches!(fac, Some(Facility::Server | Facility::Sink))
                {
                    query_default_sink(&introspect, &state);
                }
            })));

        self.mainloop.unlock();
    }
}

impl Drop for Pulse {
    fn drop(&mut self) {
        self.context.disconnect();
        self.mainloop.stop();
    }
}

fn render(s: &SinkState) -> String {
    if s.mute {
        return "🔇".into();
    }
    let symbol = if s.volume > 70 {
        "🔊"
    } else if s.volume > 30 {
        "🔉"
    } else {
        "🔈"
    };
    format!("{symbol} {}%", s.volume)
}

impl super::Block for Pulse {
    fn run(&mut self, _: &super::Options) -> Result<Option<String>, anyhow::Error> {
        let Ok(state) = self.state.read() else {
            return Ok(None);
        };
        Ok(Some(render(&state)))
    }
}

#[cfg(test)]
mod tests {
    use super::{SinkState, render};

    #[test]
    fn mute_hides_the_level() {
        assert_eq!(
            render(&SinkState {
                volume: 80,
                mute: true
            }),
            "🔇"
        );
    }

    #[test]
    fn the_speaker_icon_grows_with_the_volume() {
        let at = |volume| {
            render(&SinkState {
                volume,
                mute: false,
            })
        };
        assert_eq!(at(0), "🔈 0%");
        assert_eq!(at(30), "🔈 30%");
        assert_eq!(at(31), "🔉 31%");
        assert_eq!(at(70), "🔉 70%");
        assert_eq!(at(71), "🔊 71%");
        assert_eq!(at(100), "🔊 100%");
    }
}
