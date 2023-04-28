use std::{future::Future, sync::Arc};

use tokio::{
    sync::mpsc::{self, channel},
    task::JoinHandle,
};

use crate::{
    callback::{HandleCall, HandleCast, HandleSignal, HandleTerminate, Pattern, Response},
    connection::Connection,
    message::{Message, Signal},
    registry::Registry,
};

pub struct Process<State, Req, Resp, Fm, Fe, Fs, Ft>
where
    State: Send + Sync + 'static,
    Req: Send + Sync + 'static,
    Resp: Send + Sync + 'static,
    Fm: Future<Output = Response<State, Resp>> + Send + Sync + 'static,
    Fe: Future<Output = Response<State, Resp>> + Send + Sync + 'static,
    Fs: Future<Output = Response<State, Resp>> + Send + Sync + 'static,
    Ft: Future<Output = Response<State, Resp>> + Send + Sync + 'static,
{
    init_state: Option<State>,
    trap_exit: bool,
    register_name: Option<String>,

    pub channel: mpsc::Receiver<Message<Req, Resp>>,
    pub sender: mpsc::Sender<Message<Req, Resp>>,
    pub registry: Arc<Registry>,
    pub handle_call: HandleCall<State, Req, Resp, Fm>,
    pub handle_cast: HandleCast<State, Req, Fe>,
    pub handle_signal: HandleSignal<State, Fs>,
    pub handle_terminate: HandleTerminate<State, Ft>,
}

impl<State, Req, Resp, Fm, Fe, Fs, Ft> Process<State, Req, Resp, Fm, Fe, Fs, Ft>
where
    State: Default + Send + Sync + 'static,
    Req: Send + Sync + 'static,
    Resp: Send + Sync + 'static,
    Fm: Future<Output = Response<State, Resp>> + Send + Sync + 'static,
    Fe: Future<Output = Response<State, Resp>> + Send + Sync + 'static,
    Fs: Future<Output = Response<State, Resp>> + Send + Sync + 'static,
    Ft: Future<Output = Response<State, Resp>> + Send + Sync + 'static,
{
    pub fn new(
        registry: Arc<Registry>,
        buff_size: usize,
        register_name: Option<String>,
        handle_call: HandleCall<State, Req, Resp, Fm>,
        handle_cast: HandleCast<State, Req, Fe>,
        handle_signal: HandleSignal<State, Fs>,
        handle_terminate: HandleTerminate<State, Ft>,
    ) -> Self {
        let (sender, channel) = channel(buff_size);
        Process {
            init_state: None,
            trap_exit: false,
            channel,
            sender,
            registry,
            register_name,
            handle_call,
            handle_cast,
            handle_signal,
            handle_terminate,
        }
    }

    //  **************************************
    //    must register to registry with name optional
    //  **************************************

    pub fn spawn(mut self) -> (Connection<Req, Resp>, JoinHandle<Signal>) {
        let sender_clone = self.sender.clone();
        let state = self.init_state.unwrap_or(State::default());
        self.init_state = None;
        self.register_name
            .as_ref()
            .map(|name| self.registry.register(name.to_owned(), self.sender.clone()));
        let jh = tokio::spawn(async move { self.start(state).await });
        (Connection::new(sender_clone), jh)
    }

    // start listen on channel and process incomming message
    async fn start(mut self, mut state: State) -> Signal {
        while let Some(msg) = self.channel.recv().await {
            match self.process(msg, state).await {
                Ok(resp) => {
                    match resp.pattern {
                        Pattern::Reply(response_msg, reply_to, s) => {
                            let _ = reply_to.send(response_msg);
                            state = s
                        }
                        Pattern::NoReply(s) => state = s,
                        Pattern::Stop(info, s) => {
                            (self.handle_terminate)(info.clone(), s).await;
                            self.register_name
                                .map(|name| self.registry.unregister(&name));
                            return info;
                        }
                    }
                    if let Some(duration) = resp.sleep_for {
                        tokio::time::sleep(duration).await
                    }
                }
                Err((info, s)) => {
                    state = s;
                    let _ = (self.handle_terminate)(info.clone(), state).await;
                    self.register_name
                        .map(|name| self.registry.unregister(&name));
                    return info;
                }
            }
        }
        return Signal::Normal;
    }

    async fn process(
        &self,
        msg: Message<Req, Resp>,
        state: State,
    ) -> Result<Response<State, Resp>, (Signal, State)> {
        let res = match msg {
            Message::Data { request, reply_to } => match reply_to {
                Some(reply_to) => (self.handle_call)(request, reply_to, state).await,
                None => (self.handle_cast)(request, state).await,
            },
            Message::SignalData { info } => {
                if !self.trap_exit {
                    return Err((info, state));
                }

                (self.handle_signal)(info, state).await
            }
        };

        Ok(res)
    }
}
