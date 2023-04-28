use std::{any::Any, sync::Arc};

use dashmap::DashMap;
use tokio::sync::mpsc::Sender;

use crate::{message::Message, connection::Connection};



#[derive(Debug)]
pub enum RegistryError {
    NotFound,
    WrongType
}

pub enum RouteError<Req, Resp> {
    RegistryError(Message<Req, Resp>, RegistryError),
    Timeout(Message<Req, Resp>),
    Closed(Message<Req, Resp>)
}



pub struct Registry {
    map: DashMap<String, Box<dyn Any + Send + Sync + 'static>>
}


impl Registry {
    
    pub fn new() -> Arc<Self> {
        Arc::new(Registry { map: DashMap::new() })
    }


    // register insert channel to registry
    pub fn register<Req, Resp>(&self, name: String, channel: Sender<Message<Req, Resp>>) 
    where
        Req: Send + Sync + 'static,
        Resp: Send + Sync + 'static
    {
        let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(channel);
        self.map.insert(name, boxed);
    }

    // unregister remvoe channel from registry
    pub fn unregister(&self, name: &String) {
        self.map.remove(name);
    }


    // lookup find channel and return channel cloned
    pub fn lookup<Req, Resp>(&self, name: &String) -> Result<Connection<Req, Resp>, RegistryError> 
    where
        Req: Send + Sync + 'static,
        Resp: Send + Sync + 'static
    {
        match self.map.get(name) {
            Some(rf) => {
                match rf.value().downcast_ref::<Sender<Message<Req, Resp>>>() {
                    Some(ch) => Ok(Connection::new(ch.clone())),
                    None => Err(RegistryError::WrongType)
                }
            }
            None => Err(RegistryError::NotFound)
        }
    }


    // // route find channel and if exist send message to channel
    // pub async fn route<Req, Resp>(&self, name: &String, msg: Message<Req, Resp>, timeout: Duration) -> Result<(), RouteError<Req, Resp>> 
    // where
    //     Req: Send + Sync + 'static,
    //     Resp: Send + Sync + 'static
    // {
    //     match self.lookup(name) {
    //         Ok(chan) => {
    //             match chan.(msg, timeout).await {
    //                 Ok(_) => Ok(()),
    //                 Err(e) => {
    //                     match e {
    //                         SendTimeoutError::Timeout(msg) => {
    //                             Err(RouteError::Timeout(msg))
    //                         }
    //                         SendTimeoutError::Closed(msg) => {
    //                             self.map.remove(name);
    //                             Err(RouteError::Closed(msg))
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //         Err(e) => Err(RouteError::RegistryError(msg, e)),
    //     }
    // }


    pub fn exist(&self, name: &String) -> bool {
        self.map.contains_key(name)
    }

}