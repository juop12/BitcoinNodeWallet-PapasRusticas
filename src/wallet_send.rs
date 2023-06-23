use gtk::prelude::*;
use gtk::{Builder,Label, SpinButton, Button, Entry, Adjustment};

use crate::UiError;


pub fn update_balance(balance :&Builder, amount :&str) {
    let balance_label: Label = match balance.object("Balance Amount"){
        Some(balance_label) => balance_label,
        None => return,
    };
    balance_label.set_label(amount);
}

fn update_total_amount(builder: &Builder) {
    let total_amount_label: Label = builder.object("Total Amount Label").unwrap();
    let send_amount: SpinButton = builder.object("Send Amount").unwrap();
    let fee_amount: SpinButton = builder.object("Fee Amount").unwrap();
    
    let total_amount = send_amount.value() + fee_amount.value();
    total_amount_label.set_label(&total_amount.to_string());
}

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

fn use_available_balance(available_balance_label: &SpinButton, balance_amount: &Label) {
    
    let new_value = match balance_amount.label().parse::<f64>(){
        Ok(value) => value,
        Err(_) => return,
    };
    available_balance_label.set_value(new_value);
}

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

fn handle_transaction_sending(address: &str, amount: f64, fee: f64, balance: f64){
    // fijarse que address sea valida
    if amount + fee > balance {
        //Poner ventana de error
        return;
    }
}

fn activate_send_button(builder: &Builder) {
    let address_entry: Entry = builder.object("Pay To Entry").unwrap();
    let amount: Adjustment = builder.object("Send Amount").unwrap();
    let fee: Adjustment = builder.object("Fee Amount").unwrap();
    let send_button: Button = builder.object("Send Button").unwrap();
    let balance_label: Label = builder.object("Balance Amount").unwrap();
    send_button.connect_clicked(move |_| {
        let address = address_entry.text();
        let amount = amount.value();
        let fee = fee.value();
        let balance_amount = balance_label.label().parse::<f64>().unwrap();
        handle_transaction_sending(address.as_str(), amount, fee, balance_amount);
    });

}