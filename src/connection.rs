use crate::prelude::Result;
use crate::room::ROOMSLIST;
use serde_json::to_vec;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::spawn;

async fn handle_connections(listener: TcpListener) -> Result<()> {
    while let Ok((stream, addr)) = listener.accept().await {
        spawn(unsafe { send_roomlist_get_roomchoice(stream, addr) });
    }
    Ok(())
}

async unsafe fn send_roomlist_get_roomchoice(mut stream: TcpStream, addr: SocketAddr) {
    let mut list = vec![];
    if let Ok(roomlist) = ROOMSLIST.read() {
        for room in &*roomlist {
            list.push((room.get_id(), room.get_name()));
        }
    }

    stream.write(b"LST").await.unwrap();
    let json_v = to_vec(&list).unwrap();
    stream.write_u64(json_v.len() as u64).await.unwrap();
    stream.write(&json_v).await.unwrap();

    let mut buf = [0u8; 3];
    let Ok(1..) = stream.read_exact(&mut buf).await else {
        panic!("Error reading header");
    };
    if &buf != b"ID_" {
        panic!("Unexpected header");
    };
    let Ok(choice_id) = stream.read_u64().await else {
        panic!("Error reading choice id");
    };

    
}
