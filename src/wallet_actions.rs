use crate::hex_bytes_to_string::get_string_representation_from_bytes;
use crate::tx_info_widgets::*;
use crate::wallet_overview::{
    update_available_balance, update_receiving_pending_balance, update_sending_pending_balance,
};
use crate::wallet_send::{update_adjustments_max_value, update_balance};
use crate::wallet_transactions::{add_row, modify_block_header};
use gtk::prelude::*;
use gtk::{Application, Builder, Dialog, ListBox, TreeStore};
use node::utils::ui_communication_protocol::{
    BlockInfo, UIToWalletCommunication as UIRequest, WalletInfo,
};
use std::{
    sync::mpsc::Sender,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

const REFRESH_RATE: Duration = Duration::from_secs(5);
const SATOSHI_TO_BTC: f64 = 100000000.0;

/// Receives a BlockInfo and it updates the UI with the information of the block
/// and the transactions in it
pub fn handle_block_info(block_info: &BlockInfo, builder: &Builder) {
    let tx_tree_store: TreeStore = builder.object("Tx Tree Store").unwrap();
    tx_tree_store.clear();

    let block_number = block_info.block_number;
    let block_transaction_hashes = &block_info.block_tx_hashes;
    let header = &block_info.block_header;

    let tx_str: Vec<String> = block_transaction_hashes
        .iter()
        .map(|tx| get_string_representation_from_bytes(&mut tx.to_vec()))
        .collect();

    modify_block_header(builder, block_number, block_transaction_hashes, header);

    add_transaction_hashes(builder, &tx_str);
}

///receives the hashes of the transactions in the block and it adds them to the UI
fn add_transaction_hashes(builder: &Builder, transaction_hashes: &Vec<String>) {
    let tx_tree_store: TreeStore = builder.object("Tx Tree Store").unwrap();
    let mut i: i32 = 1;
    for transaction_hash in transaction_hashes {
        add_row(&tx_tree_store, transaction_hash, i);
        i += 1;
    }
}

/// Cleans the box where the transations are displayed
fn clean_list_box(list_box: &ListBox) {
    for widget in list_box.children() {
        list_box.remove(&widget);
    }
}

/// Receives a WalletInfo and it updates the UI with the information of the wallet
/// such as balance, utxos and pending transactions
pub fn handle_wallet_info(wallet_info: &WalletInfo, builder: &Builder) {
    let utxo_list: ListBox = builder.object("Wallet UTxO List").unwrap();
    let pending_tx_list: ListBox = builder.object("Pending Transactions List").unwrap();
    clean_list_box(&utxo_list);
    clean_list_box(&pending_tx_list);
    let available_balance: f64 = wallet_info.available_balance as f64 / SATOSHI_TO_BTC;
    let sending_pending_balance: f64 = wallet_info.sending_pending_balance as f64 / SATOSHI_TO_BTC;
    let receiving_pending_balance: f64 =
        wallet_info.receiving_pending_balance as f64 / SATOSHI_TO_BTC;
    update_balance(builder, available_balance.to_string().as_str());
    update_available_balance(builder, available_balance.to_string().as_str());
    update_sending_pending_balance(builder, sending_pending_balance.to_string().as_str());
    update_receiving_pending_balance(builder, receiving_pending_balance.to_string().as_str());
    update_adjustments_max_value(builder);

    for utxo in wallet_info.utxos.clone() {
        utxo_list.insert(&build_utxo_info(&utxo), -1);
    }

    for pending_tx in wallet_info.pending_tx.clone() {
        pending_tx_list.insert(&build_pending_tx_info(&pending_tx), -1);
    }
}

/// Sends a request to the wallet to update the information of the wallet
/// every REFRESH_RATE seconds
pub fn send_ui_update_request(sender: &Sender<UIRequest>, running: Arc<Mutex<bool>>) {
    let sender = sender.clone();
    thread::spawn(move || loop {
        thread::sleep(REFRESH_RATE);
        match running.lock() {
            Ok(program_running) => {
                if !*program_running {
                    break;
                } else {
                    sender
                        .send(UIRequest::UpdateWallet)
                        .expect("Could not send update request");
                }
            }
            Err(_) => return,
        }
    });
}

/// Shows the success message of a transaction well sent
pub fn handle_tx_sent(builder: &Builder) {
    let tx_sent_dialog: Dialog = builder.object("Succesful Send Dialog").unwrap();
    tx_sent_dialog.set_title("Transaction Sending Success");
    tx_sent_dialog.run();
}

pub fn handle_app_finished(app: &Application, running: Arc<Mutex<bool>>) {
    if let Ok(mut program_running) = running.lock() {
        *program_running = false;
        app.quit();
    }
}
