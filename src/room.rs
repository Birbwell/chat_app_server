use std::sync::RwLock;
use tokio::net::TcpStream;

pub(crate) static mut ROOMSLIST: RwLock<Vec<Room>> = RwLock::new(vec![]);

pub(crate) struct Room {
    id: u64,
    name: String,
    members: Vec<TcpStream>,
    pwd: Option<String> // storing as plaintext for now, for simplicity. WILL CHANGE
}

impl Room {
    pub(crate) fn get_name(&self) -> String {
        self.name.clone()
    }

    pub(crate) fn get_id(&self) -> u64 {
        self.id
    }
}
