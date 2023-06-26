use gio::glib::value::FromValue;
use gtk::{prelude::*, TreeIter};
use gtk::{ Builder, TreeStore,glib,Label,Button,TreeSelection,Dialog};
use node::blocks::BlockHeader;
use chrono::{NaiveDateTime, TimeZone, Utc};
use node::wallet::get_bytes_from_hex;

use std::sync::mpsc::Sender;
use node::utils::ui_communication_protocol::UIToWalletCommunication as UIRequest;

use crate::hex_bytes_to_string::get_string_representation_from_bytes;

const INDEX_COLUMN: u32 = 0;
const TX_HASH_COLUMN: u32 = 1;
const SENDER_ERROR: &str = "Error sending message to Node/Wallet thread";

/// Adds a row to the transaction tree store.
pub fn add_row(tx_tree_store: &TreeStore, tx_hash: &str, index: i32){
    let tree_iter = tx_tree_store.append(None);
    tx_tree_store.set_value(&tree_iter, INDEX_COLUMN, &glib::Value::from(index.to_string()));
    tx_tree_store.set_value(&tree_iter, TX_HASH_COLUMN, &glib::Value::from(tx_hash));
}

/// Receives all the information of a block and it updates the UI with it
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

/// Connects the button to the function that will request the merkle proof
/// checking if the user has selected a transaction and converting the hash
/// to a byte array.
pub fn initialize_merkle_proof_button(builder: &Builder, sender: &Sender<UIRequest>){
    let merkle_button: Button = builder.object("Merkle Proof Button").unwrap();
    let tree_selection: TreeSelection = builder.object("Tx Tree Selection").unwrap();
    let tree_store: TreeStore = builder.object("Tx Tree Store").unwrap();

    let block_number_label: Label = builder.object("Block Header Frame Label").unwrap();
    println!("Block number: {}", block_number_label.label().to_string());
    
    let sender_clone = sender.clone();
    merkle_button.connect_clicked(move |_| {
        let block_number = match block_number_label.label().to_string().split(" ").last() {
            Some(block_number) => block_number[1..].parse::<usize>().unwrap(),
            None => return,
        };
        let (_,tree_iter) = match tree_selection.selected(){
            Some((tree_model, tree_iter)) => (tree_model, tree_iter),
            None => {
                println!("Error while selecting tree iter");
                return;
            }
        };

        let value = tree_store.value(&tree_iter, TX_HASH_COLUMN as i32);
        
        let hash = value.get::<String>().unwrap();
        let hash_bytes = get_bytes_from_hex(hash);
        let array: [u8; 32] = match hash_bytes.try_into(){
            Ok(array) => array,
            Err(_) => return,
        };

        sender_clone.send(UIRequest::ObtainTxProof(array,  block_number-1)).expect(SENDER_ERROR);
    });

}

/// Handles the result of the merkle proof request, showing a dialog with the result.
pub fn handle_result_of_tx_proof(builder: &Builder, result: bool){
    let merkle_failure_dialog: Dialog = builder.object("Merkle Failure Dialog").unwrap();
    let merkle_success_dialog: Dialog = builder.object("Merkle Success Dialog").unwrap();
    activate_buttons(builder);

    if result {
        merkle_success_dialog.run();
        merkle_success_dialog.hide();
    } else {
        merkle_failure_dialog.run();
        merkle_failure_dialog.hide();
    }

}

/// Connects the buttons of the merkle proof result dialogs to the function that
/// will hide them.
fn activate_buttons(builder: &Builder){
    let merkle_failure_dialog: Dialog = builder.object("Merkle Failure Dialog").unwrap();
    let failure_dialog_button: Button = builder.object("Merkle Failure Button").unwrap();
    let merkle_success_dialog: Dialog = builder.object("Merkle Success Dialog").unwrap();
    let success_dialog_button: Button = builder.object("Merkle Success Button").unwrap();

    success_dialog_button.connect_clicked(move |_| {
        merkle_success_dialog.hide();
    });

    failure_dialog_button.connect_clicked(move |_| {
        merkle_failure_dialog.hide();
    });

}
