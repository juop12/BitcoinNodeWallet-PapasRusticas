use gtk::prelude::*;
use gtk::{Builder,Label, SpinButton, Button, Entry, Adjustment,Dialog};
use node::utils::ui_communication_protocol::UIToWalletCommunication as UIRequest;
use std::sync::mpsc::Sender;
const ADDRESS_LEN: usize = 34;
const BITCOIN_TO_SATOSHIS: f64 = 100000000.0;

/// Updates the balance label with the new balance.
pub fn update_balance(balance :&Builder, amount :&str) {
    let balance_label: Label = match balance.object("Balance Amount"){
        Some(balance_label) => balance_label,
        None => return,
    };
    balance_label.set_label(amount);
}

/// Updates the total amount balance according to the amount passed as argument
fn update_total_amount(builder: &Builder) {
    let total_amount_label: Label = builder.object("Total Amount Label").unwrap();
    let send_amount: SpinButton = builder.object("Send Amount").unwrap();
    let fee_amount: SpinButton = builder.object("Fee Amount").unwrap();
    
    let sth_amount = (send_amount.value() * BITCOIN_TO_SATOSHIS).round() + (fee_amount.value() * BITCOIN_TO_SATOSHIS).round();
    let total_amount = sth_amount as f64 / BITCOIN_TO_SATOSHIS;

    total_amount_label.set_label(&total_amount.to_string());
}

/// Connects the signals of the send amount and fee amount spin buttons to the update_total_amount function.
/// This function is called when the user changes the value of the spin buttons.
pub fn activate_adjustments(builder: &Builder){

    let send_amount: SpinButton = builder.object("Send Amount").unwrap();
    let fee_amount: SpinButton = builder.object("Fee Amount").unwrap();
    let mut builder_clone = builder.clone();
    send_amount.connect_value_changed(move |_| {
        update_total_amount(&builder_clone);
    });
    
    builder_clone = builder.clone();
    fee_amount.connect_value_changed(move |_| {
        update_total_amount(&builder_clone);
    });
}

/// Updates the sending value of the wallet according to the available balance
/// the user has
fn use_available_balance(available_balance_label: &SpinButton, balance_amount: &Label) {
    
    let new_value = match balance_amount.label().parse::<f64>(){
        Ok(value) => value,
        Err(_) => return,
    };
    available_balance_label.set_value(new_value);
}

/// Connects the signal of the use available balance button to the use_available_balance function.
pub fn activate_use_available_balance(builder: &Builder){
    let button: Button = match builder.object("Use Available Balance"){
        Some(button) => button,
        None => return,
    };
    let available_balance_label: SpinButton = match builder.object("Send Amount"){
        Some(available_balance_label) => available_balance_label,
        None => return ,
    };
    let balance_amount: Label = match builder.object("Balance Amount"){
        Some(balance_label) => balance_label,
        None => return,
    };
    button.connect_clicked(move |_| {
        use_available_balance(&available_balance_label,&balance_amount);
    });
}

/// Connects the corresponding signals to the clear all button to clear all the fields
pub fn activate_clear_all_button(builder: &Builder){
    let button: Button = match builder.object("Clear All Button"){
        Some(button) => button,
        None => return,
    };
    let available_balance_button: SpinButton = match builder.object("Send Amount"){
        Some(available_balance_label) => available_balance_label,
        None => return ,
    };
    let fee_button: SpinButton = match builder.object("Fee Amount"){
        Some(fee_button) => fee_button,
        None => return ,
    };
    let pay_to_entry: Entry = match builder.object("Pay To Entry"){
        Some(pay_to_entry) => pay_to_entry,
        None => return,
    };
    
    button.connect_clicked(move |_| {
        available_balance_button.set_value(0.0);
        fee_button.set_value(0.0);
        pay_to_entry.set_text("");
    });
}

/// Handles the transaction sending process. It checks if the address is valid and if the amount is valid.
/// If the fields are not correct, it shows an error dialog.
/// If the fields are correct, it sends a CreateTx message to the wallet.
fn handle_transaction_sending(builder: &Builder,address: &str, amount: f64, fee: f64, balance: f64, sender: &Sender<UIRequest>){
    
    if address.len() != ADDRESS_LEN {
        let error_dialog: Dialog = builder.object("Invalid Address Dialog").unwrap();
        error_dialog.run();
        error_dialog.hide();
        return;
    }
    if amount + fee > balance {
        let error_dialog: Dialog = builder.object("Invalid Amount Dialog").unwrap();
        error_dialog.run();
        error_dialog.hide();
        return;
    }
    let amount_in_sth = (amount * BITCOIN_TO_SATOSHIS) as i64;
    let fee_in_sth = (fee * BITCOIN_TO_SATOSHIS) as i64;
    sender.send(UIRequest::CreateTx(amount_in_sth, fee_in_sth, address.to_string())).unwrap();
    
}

/// Connects the signal of the dialogs to the hide function for each one
fn activate_dialogs(builder: &Builder){
    let error_address_dialog: Dialog = builder.object("Invalid Address Dialog").unwrap();
    let error_adress_button: Button = builder.object("Invalid Address Button").unwrap();
    error_adress_button.connect_clicked(move |_| {
        error_address_dialog.hide();
    });

    let error_amount_dialog: Dialog = builder.object("Invalid Amount Dialog").unwrap();
    let error_amount_button: Button = builder.object("Invalid Amount Button").unwrap();
    error_amount_button.connect_clicked(move |_| {
        error_amount_dialog.hide();
    });
    
    let succesful_send_dialog: Dialog = builder.object("Succesful Send Dialog").unwrap();
    let succesful_send_button: Button = builder.object("Succesful Send Button").unwrap();
    succesful_send_button.connect_clicked(move |_| {
        succesful_send_dialog.hide();
    });
}

/// Connects the signal of the send button to the handle_transaction_sending function.
pub fn activate_send_button(builder: &Builder, sender: &Sender<UIRequest>) {
    let address_entry: Entry = builder.object("Pay To Entry").unwrap();
    let amount: SpinButton = builder.object("Send Amount").unwrap();
    let fee: SpinButton = builder.object("Fee Amount").unwrap();
    let send_button: Button = builder.object("Send Button").unwrap();
    let balance_label: Label = builder.object("Balance Amount").unwrap();
    let builder_clone = builder.clone();
    let sender_clone = sender.clone();
    send_button.connect_clicked(move |_| {
        let address = address_entry.text();
        let amount = amount.value();
        let fee = fee.value();
        let balance_amount = balance_label.label().parse::<f64>().unwrap();
        handle_transaction_sending(&builder_clone,address.as_str(), amount, fee, balance_amount, &sender_clone);
    });
    activate_dialogs(builder);
}

/// Sets the maximum value of the adjustments to the balance amount.
pub fn update_adjustments_max_value(builder: &Builder){
    let balance_amount: Label = match builder.object("BalanceAmount"){
        Some(balance_label) => balance_label,
        None => return,
    };
    let send_amount_adjustment: Adjustment = match builder.object("Amount Adjustment"){
        Some(adjustment) => adjustment,
        None => return,
    };
    let fee_amount_adjustment: Adjustment = match builder.object("Fee Adjustment"){
        Some(adjustment) => adjustment,
        None => return,
    };
    send_amount_adjustment.set_upper(balance_amount.label().parse::<f64>().unwrap_or(0.0));
    fee_amount_adjustment.set_upper(balance_amount.label().parse::<f64>().unwrap_or(0.0));
}