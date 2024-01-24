use crate::prelude::Result;
use crate::room::{ROOMSLIST, Room};
use serde_json::to_vec;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::spawn;

pub(crate) async fn handle_connections(listener: TcpListener) -> Result<()> {
    while let Ok((stream, _)) = listener.accept().await {
        spawn(unsafe { send_roomlist_get_roomchoice(stream) });
    }
    Ok(())
}

async unsafe fn send_roomlist_get_roomchoice(mut stream: TcpStream) {
    let mut list = vec![];
    if let Ok(roomlist_o) = ROOMSLIST.read() {
        if let Some(roomlist) = &*roomlist_o {
            for (id, room) in roomlist {
                list.push((*id, room.get_name()));
            }
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

    match &buf {
        b"ID_" => {
            // handle choice id
            let Ok(choice_id) = stream.read_u64().await else {
                panic!("Error reading choice id");
            };
            if let Ok(roomlist_o) = ROOMSLIST.get_mut() {
                if let Some(ref mut roomlist) = *roomlist_o {
                    roomlist.entry(choice_id).and_modify(|f| f.add_user(stream));
                }
            }
        },
        b"NEW" => {
            let Ok(str_len) = stream.read_u64().await else {
                panic!("Error reading string length");
            };
            let mut str_buf = vec![0u8; str_len as usize];
            stream.read(&mut str_buf).await.unwrap();
            let room_name = String::from_utf8_lossy(&str_buf);
            Room::new_room(stream, room_name.to_string());
        },
        _ => panic!()
    }
}
