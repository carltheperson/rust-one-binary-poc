use std::path::PathBuf;

use sled::{Db, IVec, Tree};

use crate::Item;

const ITEMS_TREE: &str = "items";

pub fn open_db() -> Db {
    let mut path = PathBuf::from("./data");
    std::fs::create_dir_all(&path).ok();
    path.push("app_db");
    sled::open(path).expect("failed to open sled database")
}

fn items_tree(db: &Db) -> Tree {
    db.open_tree(ITEMS_TREE).expect("failed to open items tree")
}

pub fn ensure_item(db: &Db, item: &Item) {
    let tree = items_tree(db);
    let key = item.id.to_be_bytes();
    if tree.get(&key).unwrap().is_some() {
        return;
    }
    let value = item.name.as_bytes();
    tree.insert(key, value).unwrap();
    tree.flush().unwrap();
}

pub fn get_item(db: &Db, id: i32) -> Option<Item> {
    let tree = items_tree(db);
    let key = id.to_be_bytes();
    let Some(bytes) = tree.get(key).unwrap() else {
        return None;
    };
    let name = String::from_utf8(bytes.to_vec()).unwrap();
    Some(Item { id, name })
}

pub fn list_items(db: &Db) -> Vec<Item> {
    let tree = items_tree(db);
    let mut items = Vec::new();
    for kv in tree.iter() {
        let (k, v): (IVec, IVec) = kv.unwrap();
        let key_bytes: [u8; 4] = k.as_ref().try_into().unwrap();
        let id = i32::from_be_bytes(key_bytes);
        let name = String::from_utf8(v.to_vec()).unwrap();
        items.push(Item { id, name });
    }
    items
}
