use std::collections::HashMap;

use room::ROOMSLIST;
use tokio::net::TcpListener;
use prelude::*;

mod prelude;
mod connection;
mod room;

const IP: &str = "0.0.0.0:42530";
const SYMM: usize = 32;

#[tokio::main]
async fn main() -> Result<()> {
    unsafe { setup() };
    let mut listener = TcpListener::bind(IP).await?;
    // Create tasks to handle connections
    // Create tasks to handle rooms
    Ok(())
}

unsafe fn setup() {
    if let Ok(inner) = ROOMSLIST.get_mut() {
        *inner = Some(HashMap::new())
    }
}
