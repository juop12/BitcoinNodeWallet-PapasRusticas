use crate::error_handling::*;
use crate::wallet_persistance::*;
use crate::UiError;
use gtk::prelude::*;
use gtk::{Builder, Button, ComboBoxText, Dialog, Entry, Label};
use node::utils::ui_communication_protocol::UIToWalletCommunication as UIRequest;
use std::sync::mpsc::Sender;

const PRIV_KEY_LEN_BASE_58: usize = 52;
const SENDER_ERROR: &str = "Error sending message to node through mpsc channel";

pub enum WalletAdderError {
    ErrorInvalidPrivateKey,
    ErrorEmptyName,
}

/// Initializes the wallet selector, which lets the user add a wallet introducing a Name
/// and a private key
pub fn activate_wallet_adder(builder: &Builder) {
    let button: Button = builder.object("Wallet Adder").unwrap();
    let wallet_adder: Dialog = builder.object("Wallet Adder Dialog").unwrap();
    wallet_adder.set_title("Add Wallet");
    button.connect_clicked(move |_| {
        wallet_adder.show_all();
        wallet_adder.run();
        wallet_adder.hide();
    });
}

/// Reads the private key and returns a Result representing if ti was possible to add
/// the wallet or not.
fn add_wallet(builder: &Builder) -> Result<(), WalletAdderError> {
    let name: Entry = builder.object("Wallet Adder Name Entry").unwrap();
    let priv_key: Entry = builder.object("Wallet Adder Private Key Entry").unwrap();
    let priv_key_text = priv_key.text();
    let name_text = name.text();

    if name_text.len() == 0 {
        return Err(WalletAdderError::ErrorEmptyName);
    };
    if priv_key_text.len() != PRIV_KEY_LEN_BASE_58 {
        return Err(WalletAdderError::ErrorInvalidPrivateKey);
    };
    Ok(())
}

/// Handles the error cases that could happen when adding a wallet and displays them
/// in a dialog.
fn show_wallet_adder_error(builder: &Builder, error: WalletAdderError) {
    let wallet_adder_error_dialog: Dialog = builder.object("Wallet Adder Error Dialog").unwrap();
    let wallet_adder_error_label: Label = builder.object("Wallet Adder Error Label").unwrap();
    wallet_adder_error_dialog.set_title("Error Adding Wallet");
    match error {
        WalletAdderError::ErrorInvalidPrivateKey => {
            wallet_adder_error_label.set_text("Error adding the new Wallet: Invalid Private Key");
        }
        WalletAdderError::ErrorEmptyName => {
            wallet_adder_error_label
                .set_text("Error adding the new Wallet: The name can't be empty");
        }
    };
    wallet_adder_error_dialog.show_all();
    wallet_adder_error_dialog.run();
}

/// Handles the success case when adding a wallet and displays a dialog. The success case
/// involves adding the wallet to the combo box and changing the active wallet to the new one.
fn handle_success_add_wallet(builder: &Builder, sender: &Sender<UIRequest>) {
    let wallet_adder_success_dialog: Dialog =
        builder.object("Wallet Adder Success Dialog").unwrap();
    wallet_adder_success_dialog.set_title("Success Adding Wallet");
    let wallet_adder_success_button: Button =
        builder.object("Wallet Adder Success Button").unwrap();
    let wallet_selector: ComboBoxText = builder.object("Wallet Switcher").unwrap();
    let priv_key: Entry = builder.object("Wallet Adder Private Key Entry").unwrap();
    let name: Entry = builder.object("Wallet Adder Name Entry").unwrap();
    let priv_key_text = priv_key.text().to_string();
    let priv_key_text_clone = priv_key_text.clone();
    let name_text = name.text().to_string();

    sender
        .send(UIRequest::ChangeWallet(priv_key_text))
        .expect(SENDER_ERROR);
    sender.send(UIRequest::LastBlockInfo).expect(SENDER_ERROR);
    sender.send(UIRequest::UpdateWallet).expect(SENDER_ERROR);

    name.set_text("");
    priv_key.set_text("");
    wallet_adder_success_dialog.show_all();
    wallet_adder_success_dialog.run();
    wallet_adder_success_button.connect_clicked(move |_| {
        wallet_adder_success_dialog.hide();
    });
    wallet_selector.append(Some(&priv_key_text_clone), &name_text);
    let num_wallets = wallet_selector.model().unwrap().iter_n_children(None);
    wallet_selector.set_active(Some((num_wallets - 1) as u32));
    if save_wallet_in_disk(&priv_key_text_clone, &name_text).is_err() {
        println!("Error saving wallet in disk");
    };
}

/// Handles the success and error case while trying to add a wallet.
fn handle_add_wallet(builder: &Builder, sender: &Sender<UIRequest>) {
    match add_wallet(builder) {
        Ok(_) => handle_success_add_wallet(builder, sender),
        Err(e) => show_wallet_adder_error(builder, e),
    }
}

/// Shows the initial login screen where there are no wallets saved in disk.
fn handle_initial_login(builder: &Builder) {
    let wallet_adder: Dialog = builder.object("Wallet Adder Dialog").unwrap();
    wallet_adder.set_title("Initial Login");
    wallet_adder.show_all();
    wallet_adder.run();
}

/// Loads the wallets saved in disk and creates the combobx object with them so
/// the user can select one and change wallets to already existing ones.
pub fn initialize_wallet_selector(builder: &Builder, sender: &Sender<UIRequest>) {
    let wallet_selector: ComboBoxText = builder.object("Wallet Switcher").unwrap();

    match get_saved_wallets_from_disk(&wallet_selector) {
        Ok(wallets) => {
            wallet_selector.set_active(Some(0));
            //println!("Wallets: {:#?}", wallets);
            sender
                .send(UIRequest::ChangeWallet(wallets[0][0].to_string()))
                .expect(SENDER_ERROR);
            sender.send(UIRequest::LastBlockInfo).expect(SENDER_ERROR);
            sender.send(UIRequest::UpdateWallet).expect(SENDER_ERROR);
        }
        Err(error) => {
            match error {
                UiError::WalletsCSVWasEmpty => handle_initial_login(builder),
                _ => handle_error(builder, format!("An Error occured: {:#?}", error)),
            };
        }
    }
}

/// Initializes the actions for the wallet adder dialog.
pub fn initialize_wallet_adder_actions(builder: &Builder, sender: &Sender<UIRequest>) {
    let wallet_adder: Dialog = builder.object("Wallet Adder Dialog").unwrap();
    let cancel_button: Button = builder.object("Wallet Adder Cancel Button").unwrap();
    let add_button: Button = builder.object("Wallet Adder Add Button").unwrap();
    let invalid_wallet_button: Button = builder.object("Wallet Adder Error Button").unwrap();
    let invalid_wallet_dialog: Dialog = builder.object("Wallet Adder Error Dialog").unwrap();
    let success_dialog: Dialog = builder.object("Wallet Adder Success Dialog").unwrap();
    let success_button: Button = builder.object("Wallet Adder Success Button").unwrap();

    success_dialog.set_title("Success Adding Wallet");
    invalid_wallet_dialog.set_title("Error Adding Wallet");

    let sender_clone = sender.clone();
    let wallet_adder_clone = wallet_adder.clone();
    cancel_button.connect_clicked(move |_| {
        wallet_adder.hide();
    });
    let builder_clone = builder.clone();
    add_button.connect_clicked(move |_| {
        handle_add_wallet(&builder_clone, &sender_clone);
    });

    invalid_wallet_button.connect_clicked(move |_| {
        invalid_wallet_dialog.hide();
    });

    success_button.connect_clicked(move |_| {
        wallet_adder_clone.hide();
        success_dialog.hide();
    });
}

/// Initializes the actions for the wallet selector dialog.
pub fn initialize_change_wallet(builder: &Builder, sender: &Sender<UIRequest>) {
    let wallet_selector: ComboBoxText = builder.object("Wallet Switcher").unwrap();

    let sender_clone = sender.clone();
    wallet_selector.connect_changed(move |combo_box| {
        if combo_box.active_text().is_some() {
            match sender_clone.send(UIRequest::ChangeWallet(
                combo_box.active_id().unwrap().to_string(),
            )) {
                Ok(_) => {}
                Err(e) => println!("Error sending ChangeWallet request: {:?}", e),
            };
            sender_clone
                .send(UIRequest::LastBlockInfo)
                .expect(SENDER_ERROR);
            sender_clone
                .send(UIRequest::UpdateWallet)
                .expect(SENDER_ERROR);
        }
    });
}
