use std::str;
use node::utils::ui_communication::{BlockInfo};
use crate::wallet_transactions::add_row;
use gtk::prelude::*;
use gtk::{Builder, Label,TreeStore};
use chrono::{NaiveDateTime, TimeZone, Utc};
use crate::wallet_send::update_balance;
use crate::wallet_overview::{update_pending_balance,update_available_balance};
use crate::hex_bytes_to_string::get_string_representation_from_bytes;


pub fn handle_block_info(block_info: &BlockInfo, builder: &Builder){
    let block_number = block_info.block_number;
    let block_transaction_hashes = &block_info.block_tx_hash;
    let header = &block_info.block_header;
    
    let header_hash_label: Label = builder.object("Header Hash").unwrap();
    let prev_header_hash_label: Label = builder.object("Previous Header Hash").unwrap();
    let merkle_root_label: Label = builder.object("Merkle Root").unwrap();
    let date_label: Label = builder.object("Date").unwrap();
    let tx_count_label: Label = builder.object("Transaction Count").unwrap();
    let header_label: Label = builder.object("Block Header Frame Label").unwrap();

    let header_hash_str = get_string_representation_from_bytes(&mut header.hash().to_vec());
    let prev_header_hash_str = get_string_representation_from_bytes(&mut header.prev_hash.to_vec());
    let merkle_root_str = get_string_representation_from_bytes(&mut header.merkle_root_hash.to_vec());
    let tx_str: Vec<String> = block_transaction_hashes.iter().map(|tx| get_string_representation_from_bytes(&mut tx.to_vec())).collect();

    let date = Utc.from_utc_datetime(&NaiveDateTime::from_timestamp_opt(header.time as i64, 0).unwrap());
    header_hash_label.set_label(&header_hash_str);
    prev_header_hash_label.set_label(&prev_header_hash_str);
    merkle_root_label.set_label(&merkle_root_str);
    date_label.set_label(date.to_string().as_str());
    tx_count_label.set_label(block_transaction_hashes.len().to_string().as_str());
    header_label.set_label(format!("CURRENT BLOCK HEADER: BLOCK N° #{}", block_number).as_str());

    add_transaction_hashes(builder, &tx_str);
}

fn add_transaction_hashes(builder: &Builder, transaction_hashes: &Vec<String>){
    let tx_tree_store: TreeStore = builder.object("TxTreeStore").unwrap();
    let mut i :i32 = 1;
    for transaction_hash in transaction_hashes{
        add_row(&tx_tree_store, &transaction_hash,i);
        i+=1;
    }
}

/*
fn pub handle_wallet_info(wallet_info: &WalletInfo, builder: &Builder){
    
    update_balance(builder, wallet_info.availa.as_string());
    update_available_balance(builder, wallet_info.available_balance);
    update_pending_balance(builder,wallet_info.pending_balance)
    //meter utxos
    //meter pending
}

*/