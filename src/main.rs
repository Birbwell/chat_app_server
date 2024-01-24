use tokio::net::TcpListener;
use prelude::*;

mod prelude;
mod connection;
mod room;

const IP: &str = "0.0.0.0:42530";
const SYMM: usize = 32;

#[tokio::main]
async fn main() -> Result<()> {
    let mut listener = TcpListener::bind(IP).await?;
    // Create tasks to handle connections
    // Create tasks to handle rooms
    Ok(())
}
