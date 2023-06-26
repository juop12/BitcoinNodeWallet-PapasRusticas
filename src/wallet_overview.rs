use gtk::prelude::*;
use gtk::{Builder,Label};

/// Updates the available balance according to the amount passed as argument
pub fn update_available_balance(builder :&Builder, amount :&str) {
    let available_label: Label = builder.object("Available Quantity").unwrap();
    available_label.set_label(amount);
}

/// Updates the sending pending balance according to the amount passed as argument
pub fn update_sending_pending_balance(builder :&Builder, amount :&str) {
    let pending_label: Label = builder.object("Sending Pending Quantity").unwrap();
    pending_label.set_label(amount);
    update_total_balance(builder);
}

/// Updates the receiving pending balance according to the amount passed as argument
pub fn update_receiving_pending_balance(builder :&Builder, amount :&str) {
    let pending_label: Label = builder.object("Receiving Pending Quantity ").unwrap();
    pending_label.set_label(amount);
    update_total_balance(builder);
}

/// Updates the total balance according to the other balances the wallet has.
fn update_total_balance(builder :&Builder){
    let total_label: Label = builder.object("Total Quantity").unwrap();
    let available_label: Label = builder.object("Available Quantity").unwrap();

    let receiving_pending_label: Label = builder.object("Receiving Pending Quantity ").unwrap();

    let available_amount: f64 = available_label.label().parse::<f64>().unwrap();
    let receiving_pending_amount: f64 = receiving_pending_label.label().parse::<f64>().unwrap();
    let total_amount = available_amount + receiving_pending_amount;

    total_label.set_label(total_amount.to_string().as_str())
}
