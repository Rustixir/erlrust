use std::sync::Arc;

use registry::Registry;
use test_service_cache::start_cache;

pub mod process;
pub mod message;
pub mod url;
pub mod callback;
pub mod registry;
pub mod test_service_cache;
pub mod connection;

#[tokio::main]
async fn main() {
    start_cache(Registry::new()).await;
}



