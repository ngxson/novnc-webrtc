use lazy_static::lazy_static;
use std::sync::Mutex;
use std::collections::HashMap;

pub const LISTEN_ADDR: u8 = 0;
pub const UPSTREAM_ADDR: u8 = 1;

lazy_static! {
  static ref HASHMAP: Mutex<HashMap<u8, String>> = Mutex::new({
      let mut m = HashMap::new();
      m.insert(LISTEN_ADDR, "".to_string() );
      m.insert(UPSTREAM_ADDR, "".to_string() );
      m
  });
}

pub fn set(key: u8, val: String) {
  HASHMAP.lock().unwrap().remove(&key);
  HASHMAP.lock().unwrap().insert(key, val);
}

pub fn get(key: u8) -> String {
  HASHMAP.lock().unwrap().get(&key).cloned().unwrap_or("".to_string())
}