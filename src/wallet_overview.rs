use gtk::prelude::*;
use crate::UiError;
use gtk::{Builder,Label};

pub fn update_available_balance(builder :&Builder, amount :&str) {
    let available_label: Label = builder.object("Available Quantity").unwrap();
    available_label.set_label(amount);
}

pub fn update_sending_pending_balance(builder :&Builder, amount :&str) {
    let pending_label: Label = builder.object("Sending Pending Quantity").unwrap();
    pending_label.set_label(amount);
    update_total_balance(builder);
}

pub fn update_receiving_pending_balance(builder :&Builder, amount :&str) {
    let pending_label: Label = builder.object("Receiving Pending Quantity ").unwrap();
    pending_label.set_label(amount);
    update_total_balance(builder);
}

fn update_total_balance(builder :&Builder) -> Result<(),UiError> {
    let total_label: Label = build_label(builder, String::from("Total Quantity"))?;
    let available_label: Label = build_label(builder, String::from("Available Quantity"))?;

    let sending_pending_label: Label = build_label(builder, String::from("Sending Pending Quantity"))?;
    let receiving_pending_label: Label = build_label(builder, String::from("Receiving Pending Quantity "))?;

    let available_amount: f64 = available_label.label().parse::<f64>().unwrap();
    let sending_pending_amount: f64 = sending_pending_label.label().parse::<f64>().unwrap();
    let receiving_pending_amount: f64 = receiving_pending_label.label().parse::<f64>().unwrap();
    let total_amount = available_amount + sending_pending_amount + receiving_pending_amount;

    Ok(total_label.set_label(total_amount.to_string().as_str()))
}



fn build_label(builder :&Builder, name: String)-> Result<Label,UiError>{
    let label:Label =match builder.object(name.as_str()){
        Some(label) => label,
        None => return Err(UiError::FailedToFindObject),
    };
    Ok(label)
}
