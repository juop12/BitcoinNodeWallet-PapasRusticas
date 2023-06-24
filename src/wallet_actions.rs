use node::utils::ui_communication_protocol::{BlockInfo,WalletInfo};
use crate::wallet_transactions::add_row;
use gtk::prelude::*;
use gtk::{Builder, Label,TreeStore,ListBox};
use chrono::{NaiveDateTime, TimeZone, Utc};
use crate::wallet_send::update_balance;
use crate::wallet_overview::{update_pending_balance,update_available_balance};
use crate::hex_bytes_to_string::get_string_representation_from_bytes;
use crate::utxo_info_widget::*;
const SATOSHI_TO_BTC: f64 = 100000000.0;


pub fn handle_block_info(block_info: &BlockInfo, builder: &Builder){

    let tx_tree_store: TreeStore = builder.object("Tx Tree Store").unwrap();
    tx_tree_store.clear();
    
    let block_number = block_info.block_number;
    let block_transaction_hashes = &block_info.block_tx_hashes;
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
    let tx_tree_store: TreeStore = builder.object("Tx Tree Store").unwrap();
    let mut i :i32 = 1;
    for transaction_hash in transaction_hashes{
        add_row(&tx_tree_store, &transaction_hash,i);
        i+=1;
    }
}

fn clean_list_box(list_box: &ListBox){
    for widget in list_box.children(){
        list_box.remove(&widget);
    }
}


pub fn handle_wallet_info(wallet_info: &WalletInfo, builder: &Builder){
    let utxo_list : ListBox = builder.object("Wallet UTxO List").unwrap();
    let pending_tx_list : ListBox = builder.object("Pending Transactions List").unwrap();
    clean_list_box(&utxo_list);
    clean_list_box(&pending_tx_list);
    let available_balance: f64 = wallet_info.available_balance as f64 / SATOSHI_TO_BTC;
    //let pending_balance:  f64 = wallet_info.pending_balance as f64 / SATOSHI_TO_BTC;
    let pending_balance: f64 = (wallet_info.sending_pending_balance + wallet_info.receiving_pending_balance) as f64 / SATOSHI_TO_BTC;  //p ver de poner ambos pendings
    update_balance(builder, available_balance.to_string().as_str());
    update_available_balance(builder, available_balance.to_string().as_str());
    update_pending_balance(builder,pending_balance.to_string().as_str());
    
    for utxo in wallet_info.utxos.clone(){
        utxo_list.insert(&build_utxo_info(&utxo),-1);
    }

    for pending_tx in wallet_info.pending_tx.clone(){
        pending_tx_list.insert(&build_pending_tx_info(&pending_tx),-1);
    }
    //meter pending
}
