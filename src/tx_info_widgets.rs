use node::utils::ui_communication_protocol::{UTxOInfo, TxInfo};
use gtk::prelude::*;
use gtk::{Label , Box, Orientation, Align};
use crate::hex_bytes_to_string::get_string_representation_from_bytes;

const SATOSHI_TO_BTC: f64 = 100000000.0;
const SEPARATOR: &str = "------------------------------------------------------------------------------------------------------------";

/// Builds a Box containing the information of the Wallet UTXOs and it formats it
/// to be displayed correctly in the UI
pub fn build_utxo_info(utxo_info: &UTxOInfo) -> Box {
    let utxo_box = Box::new(Orientation::Vertical, 0);
    let amount_btc: f64 = utxo_info.amount as f64 / SATOSHI_TO_BTC;
    let hash_as_string = get_string_representation_from_bytes(&mut utxo_info.outpoint.hash.to_vec());

    let tx_id_label = Label::new(Some(format!("Hash: {}", hash_as_string).as_str()));
    tx_id_label.set_halign(Align::Start);
    let index_label = Label::new(Some(format!("Index: {}", utxo_info.outpoint.index.to_string()).as_str())); 
    index_label.set_halign(Align::Start);   
    let amount_label = Label::new(Some(format!("Amount: {}", amount_btc.to_string()).as_str()));
    amount_label.set_halign(Align::Start);
    let separator = Label::new(Some(SEPARATOR));
    separator.set_halign(Align::Start);

    utxo_box.set_child(Some(&tx_id_label));
    utxo_box.set_child(Some(&index_label));
    utxo_box.set_child(Some(&amount_label));
    utxo_box.set_child(Some(&separator));
    utxo_box.show_all();
    utxo_box
}


/// Builds a Box containing the information of the Pending Transactions and it formats it
/// to be displayed correctly in the UI
pub fn build_pending_tx_info(pending_tx_info: &TxInfo) -> Box{
    let pending_tx_box = Box::new(Orientation::Vertical, 0);
    let amount_btc: f64 = (pending_tx_info.tx_out_total + pending_tx_info.tx_in_total) as f64 / SATOSHI_TO_BTC;
    let hash_as_string = get_string_representation_from_bytes(&mut pending_tx_info.hash.to_vec());

    let tx_id_label = Label::new(Some(format!("Hash: {}", hash_as_string).as_str()));
    tx_id_label.set_halign(Align::Start);
    let amount_label = Label::new(Some(format!("Amount: {}", amount_btc.to_string()).as_str()));
    amount_label.set_halign(Align::Start);
    let separator = Label::new(Some(SEPARATOR));
    separator.set_halign(Align::Start);

    pending_tx_box.set_child(Some(&tx_id_label));
    pending_tx_box.set_child(Some(&amount_label));
    pending_tx_box.set_child(Some(&separator));
    pending_tx_box.show_all();
    pending_tx_box
}