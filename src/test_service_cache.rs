use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::sync::oneshot::Sender;

use crate::{callback::Response, message::Signal, process::Process, registry::Registry};

#[derive(Default)]
struct AppState {
    map: HashMap<i32, i32>,
}

#[derive(Debug)]
pub enum CacheReq {
    Set(i32, i32),
    Get(i32),
}

#[derive(Debug)]
pub enum CacheResp {
    Ok,
    Failed(i32),
    Get(Option<i32>),
}

async fn handle_call(
    request: CacheReq,
    from: Sender<CacheResp>,
    state: AppState,
) -> Response<AppState, CacheResp> {
    if let CacheReq::Get(k) = request {
        let result = state.map.get(&k).cloned();
        return Response::reply(CacheResp::Get(result), from, state);
    };
    Response::no_reply(state)
}

async fn handle_cast(request: CacheReq, mut state: AppState) -> Response<AppState, CacheResp> {
    if let CacheReq::Set(k, v) = request {
        state.map.insert(k, v);
    }
    Response::no_reply(state)
}

async fn handle_signal(_: Signal, state: AppState) -> Response<AppState, CacheResp> {
    println!("==> signal.");
    Response::no_reply(state)
}

async fn handle_terminate(info: Signal, state: AppState) -> Response<AppState, CacheResp> {
    println!("==> Terminate: {:?}", info);
    Response::no_reply(state)
}

pub async fn start_cache(registry: Arc<Registry>) {
    let (connection, _) = Process::new(
        registry.clone(),
        100,
        Some("Cache_Service".to_owned()),
        handle_call,
        handle_cast,
        handle_signal,
        handle_terminate,
    )
    .spawn();

    let now = Instant::now();
    // --------------------------------------------

    for counter in 0..700 {
        let timeout = Duration::from_secs(5);
        let _ = connection
            .cast(CacheReq::Set(counter, counter), timeout)
            .await;
    }

    // --------------------------------------------
    let nnow = Instant::now();
    println!("==> {:?}", nnow.duration_since(now));

    println!("");
    println!("===============================================");
    println!("");

    let now = Instant::now();
    // --------------------------------------------

    for counter in 0..100 {
        match registry.lookup::<CacheReq, CacheResp>(&"Cache_Service".to_owned()) {
            Err(e) => println!("==> {:?}", e),
            Ok(_) => {
                
            }
        }
    }

    // --------------------------------------------
    let nnow = Instant::now();
    println!("==> {:?}", nnow.duration_since(now));
}
