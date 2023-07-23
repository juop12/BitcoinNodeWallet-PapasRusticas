use chrono::{NaiveDateTime, TimeZone, Utc};
use gtk::prelude::*;
use gtk::{glib, Builder, Button, Dialog, Label, TreeSelection, TreeStore};
use node::blocks::BlockHeader;
use node::wallet::get_bytes_from_hex;
use node::blocks::proof::{HashPair, hash_pairs_for_merkle_tree};
use node::utils::ui_communication_protocol::UIRequest;
use std::sync::mpsc::Sender;

use crate::utils::error_handling::{UiError,handle_ui_error};
use crate::merkle_tree_label::*;
use crate::hex_bytes_to_string::get_string_representation_from_bytes;

const INDEX_COLUMN: u32 = 0;
const TX_HASH_COLUMN: u32 = 1;
const SENDER_ERROR: &str = "Error sending message to Node/Wallet thread";

const LEVEL_COLUMN: u32 = 0;
const HASH_PAIR_COLUMN: u32 = 1;
//const RESULTING_HASH_COLUMN: u32 = 2;

/// Connects the buttons that allow the user to switch between blocks
pub fn connect_block_switcher_buttons(builder: &Builder, sender: &Sender<UIRequest>) {
    let next_button: Button = builder.object("Next Block Button").expect("Couldn't find Next Block Button");
    let previous_button: Button = builder.object("Previous Block Button").expect("Couldn't find Previous Block Button");
    let sender_clone = sender.clone();
    let sender_clone_2 = sender.clone();
    next_button.connect_clicked(move |_| {
        sender_clone_2
            .send(UIRequest::NextBlockInfo)
            .expect(SENDER_ERROR);
    });

    previous_button.connect_clicked(move |_| {
        sender_clone
            .send(UIRequest::PrevBlockInfo)
            .expect(SENDER_ERROR);
    });
}

/// Adds a row to the transaction tree store.
pub fn add_row(tx_tree_store: &TreeStore, tx_hash: &str, index: i32) {
    let tree_iter = tx_tree_store.append(None);
    tx_tree_store.set_value(
        &tree_iter,
        INDEX_COLUMN,
        &glib::Value::from(index.to_string()),
    );
    tx_tree_store.set_value(&tree_iter, TX_HASH_COLUMN, &glib::Value::from(tx_hash));
}

/// Receives all the information of a block and it updates the UI with it
pub fn modify_block_header(
    builder: &Builder,
    block_number: usize,
    tx_hashes: &Vec<[u8; 32]>,
    header: &BlockHeader,
) {
    let header_hash_label: Label = builder.object("Header Hash").expect("Couldn't find Header Hash Label");
    let prev_header_hash_label: Label = builder.object("Previous Header Hash").expect("Couldn't find Previous Header Hash Label");
    let merkle_root_label: Label = builder.object("Merkle Root").expect("Couldn't find Merkle Root Label");
    let date_label: Label = builder.object("Date").expect("Couldn't find Date Label");
    let tx_count_label: Label = builder.object("Transaction Count").expect("Couldn't find Transaction Count Label");
    let header_label: Label = builder.object("Block Header Frame Label").expect("Couldn't find Block Header Frame Label");

    let header_hash_str = get_string_representation_from_bytes(&mut header.hash().to_vec());
    let prev_header_hash_str = get_string_representation_from_bytes(&mut header.prev_hash.to_vec());
    let merkle_root_str =
        get_string_representation_from_bytes(&mut header.merkle_root_hash.to_vec());

    let date = match &NaiveDateTime::from_timestamp_opt(header.time as i64, 0) {
        Some(date) => Utc.from_utc_datetime(date),
        None => {
            handle_ui_error(builder, UiError::ErrorParsingBlockDate);
            return;
        },
    };
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
pub fn initialize_merkle_proof_button(builder: &Builder, sender: &Sender<UIRequest>) {
    let merkle_button: Button = builder.object("Merkle Proof Button").expect("Couldn't find Merkle Proof Button");
    let tree_selection: TreeSelection = builder.object("Tx Tree Selection").expect("Couldn't find Tx Tree Selection");
    let tree_store: TreeStore = builder.object("Tx Tree Store").expect("Couldn't find Tx Tree Store");

    let block_number_label: Label = builder.object("Block Header Frame Label").expect("Couldn't find Block Header Frame Label");

    let sender_clone = sender.clone();
    let builder_clone = builder.clone();
    merkle_button.connect_clicked(move |_| {
        let block_number = match block_number_label.label().to_string().split(' ').last() {
            Some(block_number) => match block_number[1..].parse::<usize>() {
                Ok(block_number) => block_number,
                Err(_) => {
                    handle_ui_error(&builder_clone, UiError::ErrorParsingBlockNumber);
                    return;
                },
            },
            None => return,
        };
        let (_, tree_iter) = match tree_selection.selected() {
            Some((tree_model, tree_iter)) => (tree_model, tree_iter),
            None => return,
        };

        let value = tree_store.value(&tree_iter, TX_HASH_COLUMN as i32);

        let hash = match value.get::<String>() {
            Ok(hash) => hash,
            Err(_) => return,
        };
        let mut hash_bytes = match get_bytes_from_hex(hash) {
            Ok(hash_bytes) => hash_bytes,
            Err(_) => return,
        };
        hash_bytes.reverse();
        let transaction_hash: [u8; 32] = match hash_bytes.try_into() {
            Ok(transaction_hash) => transaction_hash,
            Err(_) => return,
        };

        sender_clone
            .send(UIRequest::ObtainTxProof(transaction_hash, block_number))
            .expect(SENDER_ERROR);
    });
}

fn add_merkle_root_for_tree_store(merkle_path_tree_store: &TreeStore, merkle_root: [u8; 32]) {
    let tree_iter = merkle_path_tree_store.append(None);
    let merkle_root_string = format!("Merkle Root: {}", get_string_representation_from_bytes(&mut merkle_root.to_vec()));
    merkle_path_tree_store.set_value(
        &tree_iter,
        LEVEL_COLUMN,
        &glib::Value::from(0.to_string()),
    );
    merkle_path_tree_store.set_value(
        &tree_iter,
        HASH_PAIR_COLUMN,
        &glib::Value::from(merkle_root_string),
    );
}

fn add_hashes_to_tree_store(merkle_path_tree_store: &TreeStore, path: &Vec<HashPair>) {
    let mut level = path.len();
    for hash_pair in path{
        let resulting_hashes = hash_pairs_for_merkle_tree(hash_pair.left, hash_pair.right);
        let resulting_hashes_str = get_string_representation_from_bytes(&mut resulting_hashes.to_vec());
        let left_hash = format!("Left: {}\n", get_string_representation_from_bytes(&mut hash_pair.left.to_vec()));
        let right_hash = format!("Right: {}\n\n", get_string_representation_from_bytes(&mut hash_pair.right.to_vec()));
        let res_hash = format!("Resulting Hash: {}\n", resulting_hashes_str);
        let display_hashes = left_hash + &right_hash + &res_hash;
        let tree_iter = merkle_path_tree_store.append(None);
        merkle_path_tree_store.set_value(
            &tree_iter,
            LEVEL_COLUMN,
            &glib::Value::from(level.to_string()),
        );
        merkle_path_tree_store.set_value(&tree_iter, HASH_PAIR_COLUMN, &glib::Value::from(display_hashes));
        level -= 1;
    }
}

fn add_merkle_path_rows(builder: &Builder, mut path: Vec<HashPair>, merkle_root: [u8; 32]) {
    let merkle_path_tree_store: TreeStore = builder.object("Merkle Path Store").expect("Merkle Path Store not found");
    
    if path.is_empty(){
        add_merkle_root_for_tree_store(&merkle_path_tree_store, merkle_root);
    } else {
        add_hashes_to_tree_store(&merkle_path_tree_store, &mut path);
    }
    let merkle_tree_label : Label = builder.object("Merkle Tree Label").expect("Merkle Tree Label not found");
    let merkle_tree_text = draw_merkle_proof_of_inclusion_tree(&mut path);
    merkle_tree_label.set_label(merkle_tree_text.as_str());
   
}

/// Handles the result of the merkle proof request, showing a dialog with the result.
pub fn handle_result_of_tx_proof(builder: &Builder, merkle_path: Option<(Vec<HashPair>, [u8; 32])>) {
    let merkle_success_dialog: Dialog = builder.object("Merkle Success Dialog").expect("Couldn't find Merkle Success Dialog");
    let merkle_failure_dialog: Dialog = builder.object("Merkle Failure Dialog").expect("Couldn't find Merkle Failure Dialog");
    activate_buttons(builder);

    if let Some((path, merkle_root)) = merkle_path {
        merkle_success_dialog.set_title("Proof of inclusion Success");
        add_merkle_path_rows(builder, path, merkle_root);
        merkle_success_dialog.run();
        merkle_success_dialog.hide();
    } else {
        merkle_failure_dialog.set_title("Proof of inclusion Failure");
        merkle_failure_dialog.run();
        merkle_failure_dialog.hide();
    }
}

/// Connects the buttons of the merkle proof result dialogs to the function that
/// will hide them.
fn activate_buttons(builder: &Builder) {
    let merkle_failure_dialog: Dialog = builder.object("Merkle Failure Dialog").expect("Couldn't find Merkle Failure Dialog");
    let failure_dialog_button: Button = builder.object("Merkle Failure Button").expect("Couldn't find Merkle Failure Button");
    let merkle_success_dialog: Dialog = builder.object("Merkle Success Dialog").expect("Couldn't find Merkle Success Dialog");
    let success_dialog_button: Button = builder.object("Merkle Success Button").expect("Couldn't find Merkle Success Button");
    let merkle_path_tree_store: TreeStore = builder.object("Merkle Path Store").expect("Couldn't find Merkle Path Store");
    let merkle_success_dialog_clone = merkle_success_dialog.clone();
    let merkle_tree_store_clone = merkle_path_tree_store.clone();

    success_dialog_button.connect_clicked(move |_| {
        merkle_tree_store_clone.clear();
        merkle_success_dialog_clone.hide();
    });
    
    merkle_success_dialog.connect_delete_event(move |_, _| {
        merkle_path_tree_store.clear();
        Inhibit(false)
    });

    failure_dialog_button.connect_clicked(move |_| {
        merkle_failure_dialog.hide();
    });
}
