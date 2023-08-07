use std::{io::Write, net::SocketAddr, sync::Arc, time::Duration};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::mpsc::{channel, Receiver, Sender},
};

use crossterm::event::{read, Event, KeyCode, KeyEvent};

const IP: &str = "0.0.0.0:42530";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut f = std::fs::OpenOptions::new()
        .append(true)
        .write(true)
        .create(true)
        .open("Log.txt")?;
    let listener = TcpListener::bind(IP).await?;
    println!("Listening on {IP}");

    let mut check = tokio::task::spawn_blocking(|| {
        loop {
            if let Ok(Event::Key(KeyEvent { code: KeyCode::Esc, .. })) = read() {
                return
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    });

    let (tx_master, mut rx_master) = channel::<String>(25);
    let tx_arc = Arc::new(tx_master);
    let mut senders = vec![];
    let mut tasks = vec![];

    loop {
        tokio::select! {
            Ok((stream, addr)) = listener.accept() => {
                println!("Incoming connection from {addr}");
                let (tx, rx) = channel::<String>(25);
                let tx_clone = Arc::clone(&tx_arc);

                tasks.push((tokio::spawn(async move {
                    handle_connection(stream, addr, rx, tx_clone).await;
                }), addr));

                senders.push(tx);
            },
            Some(msg) = rx_master.recv() => {
                writeln!(f, "Message Recieved: {msg}")?;
                let mut rms = vec![];
                for (idx, sender) in senders.iter().enumerate() {
                    let Ok(_) = sender.send(msg.clone()).await else {
                        rms.push(idx);
                        continue;
                    };
                }
                for r in rms {
                    senders.remove(r);
                }
            },
            _ = &mut check => {
                for task in tasks {
                    if !task.0.is_finished() {
                        task.0.abort();
                        println!("Connection from {} has been closed", task.1);
                    }
                    println!("Server shutting down");
                }
                break Ok(());
            },
        }
    }
}

async fn handle_connection(
    mut conn: TcpStream,
    addr: SocketAddr,
    mut rx: Receiver<String>,
    tx_clone: Arc<Sender<String>>,
) {
    loop {
        let mut len_buf = [0u8; 4];
        tokio::select! {
            result = conn.read(&mut len_buf) => {
                match result {
                    Ok(0) => break,
                    Ok(_) => {
                        let msg_len = u32::from_be_bytes(len_buf) as usize;
                        let mut msg_buf = vec![0u8; msg_len];
                        if conn.read_exact(&mut msg_buf).await.is_err() {
                            break;
                        }
                        let msg = String::from_utf8_lossy(&msg_buf).to_string();
                        if let Err(e) = tx_clone.send(msg).await {
                            println!("ERROR: {e:?}");
                            _ = conn.shutdown().await;
                            return;
                        };
                    },
                    Err(e) => {
                        eprintln!("Error reading from client: {e:?}");
                        break;
                    }
                }
            },
            Some(msg) = rx.recv() => {
                let msg_len = msg.len();
                len_buf = u32::to_be_bytes(msg_len as u32);
                let full_msg = [&len_buf, msg.as_bytes()].concat();
                if let Err(e) = conn.write_all(&full_msg).await {
                    eprintln!("ERROR: {e:?}");
                    _ = conn.shutdown().await;
                    return;
                };
            }
        }
    }

    println!("Connection from {addr} has been closed");
}
