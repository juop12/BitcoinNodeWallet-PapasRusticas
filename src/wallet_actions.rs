use node::utils::ui_communication_protocol::{BlockInfo,WalletInfo};
use crate::wallet_transactions::{add_row,modify_block_header};
use gtk::prelude::*;
use gtk::{Builder,TreeStore,ListBox, Dialog};
use crate::wallet_send::update_balance;
use crate::wallet_overview::{update_sending_pending_balance, update_receiving_pending_balance, update_available_balance};
use crate::hex_bytes_to_string::get_string_representation_from_bytes;
use crate::utxo_info_widget::*;


const SATOSHI_TO_BTC: f64 = 100000000.0;

/// Receives a BlockInfo and it updates the UI with the information of the block
/// and the transactions in it
pub fn handle_block_info(block_info: &BlockInfo, builder: &Builder){

    let tx_tree_store: TreeStore = builder.object("Tx Tree Store").unwrap();
    tx_tree_store.clear();
    
    let block_number = block_info.block_number;
    let block_transaction_hashes = &block_info.block_tx_hashes;
    let header = &block_info.block_header;

    let tx_str: Vec<String> = block_transaction_hashes.iter().map(|tx| get_string_representation_from_bytes(&mut tx.to_vec())).collect();
    
    modify_block_header(builder, block_number, block_transaction_hashes, header);

    add_transaction_hashes(builder, &tx_str);
}

///receives the hashes of the transactions in the block and it adds them to the UI
fn add_transaction_hashes(builder: &Builder, transaction_hashes: &Vec<String>){
    let tx_tree_store: TreeStore = builder.object("Tx Tree Store").unwrap();
    let mut i :i32 = 1;
    for transaction_hash in transaction_hashes{
        add_row(&tx_tree_store, &transaction_hash,i);
        i+=1;
    }
}

/// Cleans the box where the transations are displayed
fn clean_list_box(list_box: &ListBox){
    for widget in list_box.children(){
        list_box.remove(&widget);
    }
}

/// Receives a WalletInfo and it updates the UI with the information of the wallet
/// such as balance, utxos and pending transactions
pub fn handle_wallet_info(wallet_info: &WalletInfo, builder: &Builder){
    let utxo_list : ListBox = builder.object("Wallet UTxO List").unwrap();
    let pending_tx_list : ListBox = builder.object("Pending Transactions List").unwrap();
    clean_list_box(&utxo_list);
    clean_list_box(&pending_tx_list);
    let available_balance: f64 = wallet_info.available_balance as f64 / SATOSHI_TO_BTC;
    let sending_pending_balance: f64 = wallet_info.sending_pending_balance as f64 / SATOSHI_TO_BTC;  //p ver de poner ambos pendings
    let receiving_pending_balance: f64 = wallet_info.receiving_pending_balance as f64 / SATOSHI_TO_BTC;
    update_balance(builder, available_balance.to_string().as_str());
    update_available_balance(builder, available_balance.to_string().as_str());
    update_sending_pending_balance(builder,sending_pending_balance.to_string().as_str());
    update_receiving_pending_balance(builder,receiving_pending_balance.to_string().as_str());
    
    for utxo in wallet_info.utxos.clone(){
        utxo_list.insert(&build_utxo_info(&utxo),-1);
    }

    for pending_tx in wallet_info.pending_tx.clone(){
        pending_tx_list.insert(&build_pending_tx_info(&pending_tx),-1);
    }
}

/// Shows the success message of a transaction well sent
pub fn handle_tx_sent(builder: &Builder) {
    let tx_sent_dialog: Dialog = builder.object("Succesful Send Dialog").unwrap();
    tx_sent_dialog.run();
}