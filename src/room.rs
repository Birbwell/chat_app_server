use std::sync::RwLock;
use tokio::net::TcpStream;
use std::collections::HashMap;
use std::sync::Mutex;

pub(crate) static mut ROOMSLIST: RwLock<Option<HashMap<u64, Room>>> = RwLock::new(None);
static mut ID_COUNTER: Mutex<u64> = Mutex::new(0);

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

    pub(crate) fn add_user(&mut self, new_member: TcpStream) {
        self.members.push(new_member);
    }

    pub(crate) unsafe fn new_room(new_member: TcpStream, name: String) {
        let mut num = 0;
        if let Ok(mut n) = ID_COUNTER.lock() {
            num = *n;
            *n += 1;
        }
        let room = Room { id: num, name, members: vec![new_member], pwd: None };
        if let Ok(mut roomlist_o) = ROOMSLIST.write() {
            if let Some(ref mut roomlist) = &mut *roomlist_o {
                roomlist.insert(num, room);
            }
        }
    }
}
