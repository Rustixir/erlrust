use std::time::Duration;

use tokio::sync::oneshot;

use crate::message::Signal;



pub type HandleCall<State, Req, Resp, F> = fn(Req, oneshot::Sender<Resp>, State) -> F;
pub type HandleCast<State, Req, F>         = fn(Req, State)    -> F;
pub type HandleSignal<State, F>             = fn(Signal, State) -> F;
pub type HandleTerminate<State, F>          = fn(Signal, State) -> F;



pub enum Pattern<State, Resp> {
    Reply(Resp, oneshot::Sender<Resp>, State),
    NoReply(State),
    Stop(Signal, State),
}

pub struct Response<State, Resp> {
    pub pattern: Pattern<State, Resp>,
    pub sleep_for: Option<Duration>,
}

impl<State, Resp> Response<State, Resp> {
    pub fn reply(response: Resp, from: oneshot::Sender<Resp>, state: State) -> Self {
        Response {
            pattern: Pattern::Reply(response, from, state),
            sleep_for: None,
        }
    }
    pub fn no_reply(state: State) -> Self {
        Response {
            pattern: Pattern::NoReply(state),
            sleep_for: None,
        }
    }
    pub fn stop(info: Signal, state: State) -> Self {
        Response {
            pattern: Pattern::Stop(info, state),
            sleep_for: None,
        }
    }
    pub fn with_sleep(&mut self, sleep_for: Duration) {
        self.sleep_for = Some(sleep_for)
    }
}
