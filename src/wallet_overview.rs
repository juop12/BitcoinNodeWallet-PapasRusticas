use gtk::prelude::*;
use crate::UiError;
use gtk::{Builder,Label};

pub fn update_available_balance(builder :&Builder, amount :&str) {
    let available_label: Label = match builder.object("availableQuantity"){
        Some(available_label) => available_label,
        None => return,
    };
    available_label.set_label(amount);
}

pub fn update_pending_balance(builder :&Builder, amount :&str) {
    let pending_label: Label = match builder.object("pendingQuantity"){
        Some(pending_label) => pending_label,
        None => return,
    };
    pending_label.set_label(amount);
    update_total_balance(builder);
}

fn update_total_balance(builder :&Builder) -> Result<(),UiError> {
    let total_label: Label = build_label(builder, String::from("totalQuantity"))?;
    let available_label: Label = build_label(builder, String::from("availableQuantity"))?;
    let pending_label: Label = build_label(builder, String::from("pendingQuantity"))?;
    let available_amount: f64 = match available_label.label().parse::<f64>(){
        Ok(value) => value,
        Err(_) => return Err(UiError::FailedToFindObject)
    };
    let pending_amount: f64 = match pending_label.label().parse::<f64>(){
        Ok(value) => value,
        Err(_) => return Err(UiError::FailedToFindObject)
    };
    
    let total_amount = available_amount + pending_amount;

    Ok(total_label.set_label(total_amount.to_string().as_str()))
}



fn build_label(builder :&Builder, name: String)-> Result<Label,UiError>{
    let label:Label =match builder.object(name.as_str()){
        Some(label) => label,
        None => return Err(UiError::FailedToFindObject),
    };
    Ok(label)
}
