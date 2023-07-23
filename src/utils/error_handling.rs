use gtk::prelude::*;
use gtk::{Application, Builder, Button, Dialog, Label};
use std::sync::{Arc, Mutex}
;
use node::utils::btc_errors::WalletError;


/// Enum that represents the possible errors that can happen in the UI
#[derive(Debug)]
pub enum UiError {
    FailedToBuildUi,
    FailedToFindObject,
    ErrorReadingFile,
    ErrorWritingFile,
    WalletsCSVWasEmpty,
    ErrorParsingBlockNumber,
    ErrorParsingBlockDate,
    ErrorParsingAmount
}

fn handle_error(builder: &Builder, text: String) {
    let err_button: Button = builder.object("Error Button").expect("Couldn't find error button");
    let err_dialog: Dialog = builder.object("Error Dialog").expect("Couldn't find error dialog");
    let err_label: Label = builder.object("Error Label").expect("Couldn't find error label");
    let err_clone = err_dialog.clone();
    err_label.set_text(text.as_str());
    err_button.connect_clicked(move |_| {
        err_clone.hide();
    });
    err_dialog.set_title("Error");
    err_dialog.show_all();
    err_dialog.run();
}

pub fn handle_ui_error(builder: &Builder, ui_error: UiError) {
    let error_string: String = format!(" An Error Ocurred: {:?}", ui_error);
    handle_error(builder, error_string);
}


pub fn handle_wallet_error(builder: &Builder, wallet_error: WalletError, window_running: Arc<Mutex<bool>>,) {
    let error_string: String;
    if wallet_error == WalletError::ErrorDisconectedFromBlockchain {
        let node_disconnection_label: Label = builder.object("Node Disconnection Label").expect("Couldn't find node disconnection label");
        node_disconnection_label.set_text("Node Disconnected");
        match window_running.lock() {
            Ok(mut mutex) => {
                *mutex = false;
            },
            Err(_) => return,
        };
        error_string = format!("The node was disconnected from the blockchain. Please restart the application.");
    }else{
        error_string = format!(" An Error Ocurred: {:?}", wallet_error);
    }
    handle_error(builder, error_string);
    
}

pub fn handle_initialization_error(builder: &Builder, app: &Application) {
    let err_button: Button = builder.object("Error Button").expect("Couldn't find error button");
    let err_label: Label = builder.object("Error Label").expect("Couldn't find error label");
    let err_dialog: Dialog = builder.object("Error Dialog").expect("Couldn't find error dialog");
    let err_clone = err_dialog.clone();
    err_button.connect_clicked(move |_| {
        err_clone.hide();
    });

    err_label.set_text("There was an error initializing the node");
    err_dialog.set_title("Error Initializing Node");
    err_dialog.show_all();
    err_dialog.run();
    app.quit();
}
