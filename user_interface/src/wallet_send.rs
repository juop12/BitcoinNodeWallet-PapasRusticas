use gtk::prelude::*;
use gtk::{Builder,Label, SpinButton, Button};

use crate::UiError;


pub fn update_balance(balance :&Builder, amount :&str) {
    let balance_label: Label = match balance.object("BalanceAmount"){
        Some(balance_label) => balance_label,
        None => return,
    };
    balance_label.set_label(amount);
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
    let available_balance_label: SpinButton = match builder.object("SendAmount"){
        Some(available_balance_label) => available_balance_label,
        None => return ,
    };
    let balance_amount: Label = match builder.object("BalanceAmount"){
        Some(balance_label) => balance_label,
        None => return,
    };
    button.connect_clicked(move |_| {
        use_available_balance(&available_balance_label,&balance_amount);
    });
}