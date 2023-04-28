use tokio::sync::oneshot;
use crate::url::Url;



pub enum MessageType {
    Message,
    Event,
    Signal
}


#[derive(Clone, Debug)]
pub enum Signal {
    Normal,
    Exit(Url, Description),
    Disconnect(Url, Description)
}

#[derive(Clone, Debug)]
pub enum Description {
    Normal
}



pub enum Message<Request, Response> {

    Data {
        request: Request,
        reply_to: Option<oneshot::Sender<Response>>
    },

    SignalData {
        info: Signal
    },

}

impl<Request, Response> Message<Request, Response> {
    
    #[inline]
    pub fn new_call(request: Request, reply_to: oneshot::Sender<Response>) -> Self {
        Message::Data { 
            request, 
            reply_to: Some(reply_to) 
        }
    }

    #[inline]
    pub fn new_event(request: Request) -> Self {
        Message::Data { 
            request, 
            reply_to: None
        }
    }

    #[inline]
    pub fn new_signal(info: Signal) -> Self {
        Message::SignalData { 
            info
        }
    }


    #[inline]
    pub fn get_type(&self) -> MessageType {
        match self {
            Message::Data { request: _, reply_to } => {
                if reply_to.is_some() {
                    return MessageType::Message
                } 
                return MessageType::Event
            }
            Message::SignalData { info: _ } => {
                return MessageType::Signal
            }
        }
        
    }


    #[inline]
    pub fn unwrap(self) -> Request {
        match self {
            Message::Data { request: req, reply_to: _ } => req,
            Message::SignalData { info: _ } => panic!("unexpected")
        }
    }

}

