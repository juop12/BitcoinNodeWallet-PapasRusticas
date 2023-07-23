use crate::hex_bytes_to_string::get_string_representation_from_bytes;
use crate::tx_info_widgets::*;
use crate::wallet_overview::{
    update_available_balance, update_receiving_pending_balance, update_sending_pending_balance,
};
use crate::wallet_send::{update_adjustments_max_value, update_balance};
use crate::wallet_transactions::{add_row, modify_block_header};
use gtk::prelude::*;
use gtk::{Application, Builder, Dialog, ListBox, TreeStore, Window};
use node::utils::ui_communication_protocol::{
    BlockInfo, UIRequest, WalletInfo,
};
use std::{
    sync::mpsc::Sender,
    sync::{Arc, Mutex},
    thread,
    thread::JoinHandle,
    time::Duration,
};

const REFRESH_RATE: Duration = Duration::from_secs(5);
const INITIAL_WAIT_INTERVAL: Duration = Duration::from_secs(2);
const SATOSHI_TO_BTC: f64 = 100000000.0;

/// Receives a BlockInfo and it updates the UI with the information of the block
/// and the transactions in it
pub fn handle_block_info(block_info: &BlockInfo, builder: &Builder) {
    let tx_tree_store: TreeStore = builder.object("Tx Tree Store").expect("Tx Tree Store not found");
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
    let tx_tree_store: TreeStore = builder.object("Tx Tree Store").expect("Tx Tree Store not found");
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
    let utxo_list: ListBox = builder.object("Wallet UTxO List").expect("UTxO List not found");
    let pending_tx_list: ListBox = builder.object("Pending Transactions List").expect("Pending Transactions List not found");
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

fn check_if_node_initialized(running: Arc<Mutex<bool>>) {
    let mut node_initialized = false;
    while !node_initialized {
        thread::sleep(INITIAL_WAIT_INTERVAL);
        if let Ok(program_running) = running.lock() {
            if *program_running {
                node_initialized = true;
            }
        }
    }
}

fn request_update_loop(sender: Sender<UIRequest>, running: Arc<Mutex<bool>>) {
    loop {
        thread::sleep(REFRESH_RATE);
        match running.lock() {
            Ok(program_running) => {
                if !*program_running {
                    break;
                } else {
                    if sender
                        .send(UIRequest::UpdateWallet).is_err() {
                            return;
                        }
                }
            }
            Err(_) => return,
        }
    }
}
/// Sends a request to the wallet to update the information of the wallet
/// every REFRESH_RATE seconds
pub fn send_ui_update_request(sender: &Sender<UIRequest>, running: Arc<Mutex<bool>>) -> JoinHandle<()> {
    let sender = sender.clone();
    let join_handle = thread::spawn(move || {
        check_if_node_initialized(running.clone());
        request_update_loop(sender, running);
    });
    join_handle
}

/// Shows the success message of a transaction well sent
pub fn handle_tx_sent(builder: &Builder) {
    let tx_sent_dialog: Dialog = builder.object("Succesful Send Dialog").expect("Succesful Send Dialog not found");
    tx_sent_dialog.set_title("Transaction Sending Success");
    tx_sent_dialog.run();
}

fn close_window(builder: &Builder) {
    let main_window: Window = builder.object("Main Window").expect("Main Window not found");
    main_window.close();
}
/// E
pub fn handle_app_finished(builder: &Builder, app: &Application, running: Arc<Mutex<bool>>) {
    close_window(builder);
    if let Ok(mut program_running) = running.lock() {
        *program_running = false;
        app.quit();
    }
    /*println!("estoy despues del quit");
    match update_wallet_join_handle.lock() {
        Ok(mut join_handle) => {
            println!("pude lockear el join handle");
            match join_handle.take() {
                Some(handle) => {
                    println!("pude tomar el join handle");
                    match handle.join() {
                        Ok(_) => { println!("pude hacer join al update wallet thread"); },
                        Err(_) => println!("Error joining update wallet thread"),
                    }
                },
                None => println!("Error taking update wallet join handle"),
            }
        },
        Err(_) => println!("Error locking update wallet join handle"),
    }*/
}
