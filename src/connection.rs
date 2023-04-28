use std::time::Duration;

use tokio::sync::{mpsc::{Sender, error::SendTimeoutError}, oneshot};

use crate::message::{Message, Signal};



#[derive(Debug)]
pub enum ServerError<Req> {
    Timeout(Req),
    Closed(Req),
    InternalServerError
}

#[derive(Clone)]
pub struct Connection<Req, Resp> {
    endpoint: Sender<Message<Req, Resp>>
}

impl<Req, Resp> Connection<Req, Resp> {

    pub fn new(endpoint: Sender<Message<Req, Resp>>) -> Self {
        Connection { endpoint }
    }

    pub async fn call(&self, request: Req, timeout: Duration) -> Result<Resp, ServerError<Req>>{
        let (reply_to, rx) = oneshot::channel();
        let msg = Message::new_call(request, reply_to);
        if let Err(e) = self.endpoint.send_timeout(msg, timeout).await {
            match e {
                SendTimeoutError::Timeout(r) => return Err(ServerError::Timeout(r.unwrap())),
                SendTimeoutError::Closed(r) => return Err(ServerError::Closed(r.unwrap()))
            }
        }

        match rx.await {
            Ok(resp) => Ok(resp),
            Err(_) => Err(ServerError::InternalServerError)
        }
    }

    pub async fn cast(&self, request: Req, timeout: Duration) -> Result<(), ServerError<Req>> {
        let msg = Message::new_event(request);
        match self.endpoint.send_timeout(msg, timeout).await {
            Ok(_) => Ok(()),
            Err(e) => {
                match e {
                    SendTimeoutError::Timeout(r) => Err(ServerError::Timeout(r.unwrap())),
                    SendTimeoutError::Closed(r) =>  Err(ServerError::Closed(r.unwrap()))
                }
            }
        }
    }

    pub async fn signal(&self, info: Signal, timeout: Duration) -> Result<(), ServerError<Req>> {
        let msg = Message::new_signal(info);
        match self.endpoint.send_timeout(msg, timeout).await {
            Ok(_) => Ok(()),
            Err(e) => {
                match e {
                    SendTimeoutError::Timeout(_) => Err(ServerError::InternalServerError),
                    SendTimeoutError::Closed(_) =>  Err(ServerError::InternalServerError)
                }
            }
        }
    }

    
}