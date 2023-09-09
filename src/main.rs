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

    let (tx_master, mut rx_master) = channel::<(Vec<u8>, Vec<u8>)>(25);
    let sv_rsa = Rsa::generate(2048)?;
    let rsa_arc = Arc::new(sv_rsa);
    let tx_arc = Arc::new(tx_master);
    let mut senders = vec![];
    let mut tasks = vec![];

    loop {
        eprintln!("Bing");
        tokio::select! {
            Ok((stream, addr)) = listener.accept() => {
                println!("Incoming connection from {addr}");
                let (tx, rx) = channel::<(Vec<u8>, Vec<u8>)>(25);
                let tx_clone = Arc::clone(&tx_arc);
                let rsa_clone = Arc::clone(&rsa_arc);

                tasks.push((tokio::spawn(async move {
                    handle_connection(stream, addr, rx, tx_clone, rsa_clone).await;
                }), addr));

                senders.push(tx);
            },
            Some(msg) = rx_master.recv() => {
                writeln!(f, "Message Recieved: {msg:?}")?;
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
    mut rx: Receiver<(Vec<u8>, Vec<u8>)>,
    tx: Arc<Sender<(Vec<u8>, Vec<u8>)>>,
    sv_rsa: Arc<Rsa<Private>>,
) {
    let mut cl_rsa: Option<Rsa<Public>> = None;
    loop {
        let mut typ_buf = [0u8; 3];
        let mut len_buf = [0u8; 4];
        tokio::select! {
            result = stream.read_exact(&mut typ_buf) => {
                match result {
                    Ok(0) => break,
                    Ok(_) => {
                        stream.read_exact(&mut len_buf).await.unwrap();
                        match &typ_buf {
                            b"ENC" => {
                                if cl_rsa.is_some() {
                                    let key_len = u32::from_be_bytes(len_buf) as usize;
                                    let mut key = vec![0u8; key_len];
                                    stream.read_exact(&mut key).await.unwrap();

                                    let mut msg_hed = [0u8; 3];
                                    stream.read_exact(&mut msg_hed).await.unwrap();

                                    if &msg_hed == b"MSG" {
                                        let mut msg_len_buf = [0u8; 4];
                                        stream.read_exact(&mut msg_len_buf).await.unwrap();
                                        let msg_len = u32::from_be_bytes(msg_len_buf) as usize;
                                        let mut msg = vec![0u8; msg_len];
                                        stream.read_exact(&mut msg).await.unwrap();

                                        let dec_key = {
                                            let mut t = [0u8; 2048];
                                            let len = sv_rsa.private_decrypt(&key, &mut t, Padding::PKCS1).unwrap();
                                            t[0..len].to_owned()
                                        };
                                        tx.send((dec_key, msg)).await.unwrap();
                                    } else {
                                        eprintln!("An error has occured on connection from {addr:?}");
                                        break;
                                    }
                                }
                            },
                            b"PUB" => {
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
            Some((key, msg)) = rx.recv() => {
                if let Some(c_rsa) = cl_rsa.as_ref() {
                    let enc_key = {
                        let mut t = vec![0u8; 2048 as usize];
                        let len = c_rsa.public_encrypt(&key, &mut t, Padding::PKCS1).unwrap();
                        t[0..len].to_owned()
                    };
                    let key_hed = "ENC".as_bytes();
                    let msg_hed = "MSG".as_bytes();

                    let key_len = (enc_key.len() as u32).to_be_bytes();
                    let msg_len = (msg.len() as u32).to_be_bytes();

                    stream.write_all(&[key_hed, &key_len, &enc_key, msg_hed, &msg_len, &msg].concat()).await.unwrap();
                }
            }
        }
    }

    println!("Connection from {addr} has been closed");
}
