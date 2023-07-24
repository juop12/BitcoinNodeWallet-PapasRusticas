use crate::utils::node_status::NodeStatus;
use gtk::prelude::*;
use gtk::{Builder, Button, Dialog, Label, Window};
use node::utils::btc_errors::WalletError;
use std::sync::{Arc, Mutex};

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
    ErrorParsingAmount,
}

fn handle_error(builder: &Builder, text: String) {
    let err_button: Button = builder
        .object("Error Button")
        .expect("Couldn't find error button");
    let err_dialog: Dialog = builder
        .object("Error Dialog")
        .expect("Couldn't find error dialog");
    let err_label: Label = builder
        .object("Error Label")
        .expect("Couldn't find error label");
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

pub fn handle_wallet_error(
    builder: &Builder,
    wallet_error: WalletError,
    node_status: Arc<Mutex<NodeStatus>>,
) {
    let error_string: String;
    if wallet_error == WalletError::ErrorDisconectedFromBlockchain {
        let node_disconnection_label: Label = builder
            .object("Node Disconnection Label")
            .expect("Couldn't find node disconnection label");
        node_disconnection_label.set_text("Node Disconnected");
        match node_status.lock() {
            Ok(mut current_status) => {
                *current_status = NodeStatus::Terminated;
            }
            Err(_) => return,
        };
        error_string = String::from(
            "The node was disconnected from the blockchain. Please restart the application.",
        );
    } else {
        error_string = format!(" An Error Ocurred: {:?}", wallet_error);
    }
    handle_error(builder, error_string);
}

pub fn handle_initialization_error(builder: &Builder, node_status: Arc<Mutex<NodeStatus>>) {
    match node_status.lock() {
        Ok(mut current_status) => {
            *current_status = NodeStatus::Terminated;
        }
        Err(_) => return,
    };
    let err_button: Button = builder
        .object("Error Button")
        .expect("Couldn't find error button");
    let err_label: Label = builder
        .object("Error Label")
        .expect("Couldn't find error label");
    let err_dialog: Dialog = builder
        .object("Error Dialog")
        .expect("Couldn't find error dialog");
    let err_clone_1 = err_dialog.clone();
    let err_clone_2 = err_dialog.clone();
    let loading_window: Window = builder
        .object("Loading Screen Window")
        .expect("Couldn't find Loading Screen Window");
    let loading_window_clone = loading_window.clone();

    err_button.connect_clicked(move |_| {
        err_clone_1.hide();
        loading_window.close();
    });

    err_dialog.connect_delete_event(move |_, _| {
        err_clone_2.hide();
        loading_window_clone.close();
        Inhibit(false)
    });

    err_label.set_text("There was an error initializing the node");
    err_dialog.set_title("Error Initializing Node");
    err_dialog.show_all();
}
