use crate::connection::handle_connections;
use std::collections::HashMap;
use room::ROOMSLIST;
use tokio::net::TcpListener;
use prelude::*;

mod prelude;
mod connection;
mod room;

const IP: &str = "0.0.0.0:42530";

#[tokio::main]
async fn main() -> Result<()> {
    unsafe { setup() };
    let Ok(listener) = TcpListener::bind(IP).await else {
        return Err("Failed to bind to port!");
    };

    // Create tasks to handle connections
    let jh = tokio::spawn(handle_connections(listener));
    
    // Create tasks to handle rooms

    jh.abort();
    Ok(())
}

unsafe fn setup() {
    if let Ok(inner) = ROOMSLIST.get_mut() {
        *inner = Some(HashMap::new())
    }
}
