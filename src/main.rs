use std::{io::Write, net::SocketAddr, sync::Arc, time::Duration};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::mpsc::{channel, Receiver, Sender},
};

use openssl::pkey::{Public, Private};
use openssl::rsa::{Rsa, Padding};

use crossterm::event::{self, Event, KeyCode, KeyEvent};

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
            if let Ok(Event::Key(KeyEvent { code: KeyCode::Esc, .. })) = event::read() {
                return
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    });

    let (tx_master, mut rx_master) = channel::<String>(25);
    let sv_rsa = Rsa::generate(2048)?;
    let rsa_arc = Arc::new(sv_rsa);
    let tx_arc = Arc::new(tx_master);
    let mut senders = vec![];
    let mut tasks = vec![];

    loop {
        tokio::select! {
            Ok((stream, addr)) = listener.accept() => {
                println!("Incoming connection from {addr}");
                let (tx, rx) = channel::<String>(25);
                let tx_clone = Arc::clone(&tx_arc);
                let rsa_clone = Arc::clone(&rsa_arc);

                tasks.push((tokio::spawn(async move {
                    handle_connection(stream, addr, rx, tx_clone, rsa_clone).await;
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
    mut stream: TcpStream,
    addr: SocketAddr,
    mut rx: Receiver<String>,
    tx: Arc<Sender<String>>,
    sv_rsa: Arc<Rsa<Private>>,
) {
    let mut cl_rsa: Option<Rsa<Public>> = None;
    loop {
        let mut typ_buf = [0u8; 3];
        let mut len_buf = [0u8; 4];
        eprintln!("Bing");
        tokio::select! {
            result = stream.read_exact(&mut typ_buf) => {
                match result {
                    Ok(0) => break,
                    Ok(_) => {
                        let typ = String::from_utf8_lossy(&typ_buf).to_string();
                        stream.read_exact(&mut len_buf).await.unwrap();
                        match typ.as_str() {
                            "MSG" => {
                                let msg_len = u32::from_be_bytes(len_buf) as usize;
                                let mut msg_buf = vec![0u8; msg_len];
                                if stream.read_exact(&mut msg_buf).await.is_err() {
                                    break;
                                }
                                let mut dec_msg: Vec<u8> = vec![];
                                sv_rsa.private_decrypt(&msg_buf, &mut dec_msg, Padding::PKCS1).unwrap();
                                let msg = String::from_utf8_lossy(&dec_msg).to_string();
                                tx.send(msg).await.unwrap();
                            },
                            "KEY" => { // Works
                                let key_len = u32::from_be_bytes(len_buf) as usize;
                                let mut key_buf = vec![0u8; key_len];
                                if stream.read_exact(&mut key_buf).await.is_err() {
                                    break;
                                }
                                cl_rsa = Some(Rsa::public_key_from_der(&key_buf).unwrap());
                            },
                            _ => eprintln!("Error reading from stream: {result:?}"),
                        }
                    },
                    Err(e) => {
                        eprintln!("Error reading from client: {e:?}");
                        break;
                    }
                }
            },
            Some(msg) = rx.recv() => {
                eprintln!("{msg:?}");
                let msg_len = msg.len();
                len_buf = u32::to_be_bytes(msg_len as u32);
                let full_msg = [&len_buf, msg.as_bytes()].concat();
                if let Err(e) = stream.write_all(&full_msg).await {
                    eprintln!("ERROR: {e:?}");
                    _ = stream.shutdown().await;
                    return;
                };

                if let Some(r) = cl_rsa.as_ref() {
                    let mut enc_msg = vec![];
                    let msg_bytes = msg.as_bytes();
                    r.public_encrypt(msg_bytes, &mut enc_msg, Padding::PKCS1).unwrap();
                    let msg_len = (enc_msg.len() as u32).to_be_bytes(); // Write the length header.
                    let header = "MSG".as_bytes();
                    stream.write_all(&[header, &msg_len, &enc_msg].concat()).await.unwrap();
                    stream.flush().await.unwrap();
                }
            }
        }
    }

    println!("Connection from {addr} has been closed");
}
