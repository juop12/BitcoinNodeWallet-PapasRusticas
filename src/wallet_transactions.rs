use gtk::prelude::*;
use gtk::{ Builder,  TreeStore,glib,Label};
use node::blocks::BlockHeader;
use chrono::{NaiveDateTime, TimeZone, Utc};

use crate::hex_bytes_to_string::get_string_representation_from_bytes;

const INDEX_COLUMN: u32 = 0;
const TX_HASH_COLUMN: u32 = 1;

pub fn add_row(tx_tree_store: &TreeStore, tx_hash: &str, index: i32){
    let tree_iter = tx_tree_store.append(None);
    tx_tree_store.set_value(&tree_iter, INDEX_COLUMN, &glib::Value::from(index.to_string()));
    tx_tree_store.set_value(&tree_iter, TX_HASH_COLUMN, &glib::Value::from(tx_hash));
}

pub fn modify_block_header(builder: &Builder, block_number: usize, tx_hashes: &Vec<[u8;32]>, header: &BlockHeader){

    let header_hash_label: Label = builder.object("Header Hash").unwrap();
    let prev_header_hash_label: Label = builder.object("Previous Header Hash").unwrap();
    let merkle_root_label: Label = builder.object("Merkle Root").unwrap();
    let date_label: Label = builder.object("Date").unwrap();
    let tx_count_label: Label = builder.object("Transaction Count").unwrap();
    let header_label: Label = builder.object("Block Header Frame Label").unwrap();

    let header_hash_str = get_string_representation_from_bytes(&mut header.hash().to_vec());
    let prev_header_hash_str = get_string_representation_from_bytes(&mut header.prev_hash.to_vec());
    let merkle_root_str = get_string_representation_from_bytes(&mut header.merkle_root_hash.to_vec());

    let date = Utc.from_utc_datetime(&NaiveDateTime::from_timestamp_opt(header.time as i64, 0).unwrap());
    header_hash_label.set_label(&header_hash_str);
    prev_header_hash_label.set_label(&prev_header_hash_str);
    merkle_root_label.set_label(&merkle_root_str);
    date_label.set_label(date.to_string().as_str());
    tx_count_label.set_label(tx_hashes.len().to_string().as_str());
    header_label.set_label(format!("CURRENT BLOCK HEADER: BLOCK N° #{}", block_number).as_str());
}
