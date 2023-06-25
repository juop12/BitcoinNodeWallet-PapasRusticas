use gtk::prelude::*;
use gtk::{Application, Box, ComboBoxText, Builder, Entry, Dialog, Button, Label, Window};
use node::wallet;
use crate::UiError;
use crate::wallet_persistance::*;
use std::sync::mpsc::Sender;
use node::utils::ui_communication_protocol::UIToWalletCommunication as UIRequest;

const PRIV_KEY_LEN_BASE_58: usize = 52;

pub enum WalletAdderError {
    ErrorInvalidPrivateKey,
    ErrorEmptyName,
}

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

fn show_wallet_adder_error(builder: &Builder, error: WalletAdderError) {
    let wallet_adder_error_dialog: Dialog = builder.object("Wallet Adder Error Dialog").unwrap();
    let wallet_adder_error_label: Label = builder.object("Wallet Adder Error Label").unwrap();
    wallet_adder_error_dialog.set_title("Error Adding Wallet");
    match error {
        WalletAdderError::ErrorInvalidPrivateKey => {
            wallet_adder_error_label.set_text("Error adding the new Wallet: Invalid Private Key");
        },
        WalletAdderError::ErrorEmptyName => {
            wallet_adder_error_label.set_text("Error adding the new Wallet: The name can't be empty");
        },
    };
    wallet_adder_error_dialog.show_all();
    wallet_adder_error_dialog.run();
}

fn handle_success_add_wallet(builder: &Builder, sender: &Sender<UIRequest>) {
    let wallet_adder_success_dialog: Dialog = builder.object("Wallet Adder Success Dialog").unwrap();
    let wallet_adder_success_button: Button = builder.object("Wallet Adder Success Button").unwrap();
    let wallet_selector: ComboBoxText = builder.object("Wallet Switcher").unwrap();
    let priv_key: Entry = builder.object("Wallet Adder Private Key Entry").unwrap();
    let name: Entry = builder.object("Wallet Adder Name Entry").unwrap();
    let priv_key_text = priv_key.text().to_string();
    let priv_key_text_clone = priv_key_text.clone();
    let name_text = name.text().to_string();
    sender.send(UIRequest::ChangeWallet(priv_key_text)).unwrap();
    sender.send(UIRequest::LastBlockInfo).unwrap();
    name.set_text("");
    priv_key.set_text("");
    wallet_adder_success_dialog.show_all();
    wallet_adder_success_dialog.run();
    wallet_adder_success_button.connect_clicked(move |_| {
        wallet_adder_success_dialog.hide();
    });
    wallet_selector.append(Some(&priv_key_text_clone),&name_text);
    let num_wallets = wallet_selector.model().unwrap().iter_n_children(None);
    wallet_selector.set_active(Some((num_wallets - 1) as u32));
    if let Err(_) = save_wallet_in_disk(&priv_key_text_clone, &name_text){
        println!("Error saving wallet in disk");
    };
}

fn handle_add_wallet(builder: &Builder, sender: &Sender<UIRequest>) {
    match add_wallet(builder) {
        Ok(_) => handle_success_add_wallet(builder, sender),
        Err(e) => show_wallet_adder_error(builder, e),
    }
}

fn handle_initial_login(builder: &Builder, sender: &Sender<UIRequest>, app: &Application) {
    let wallet_adder: Dialog = builder.object("Wallet Adder Dialog").unwrap();
    let cancel_button: Button = builder.object("Wallet Adder Cancel Button").unwrap();
    let main_window: Window = builder.object("Ventana").unwrap();
    let app_clone = app.clone();
    let wallet_adder_clone = wallet_adder.clone();
    wallet_adder.set_title("Initial Login");
    wallet_adder.show_all();
    wallet_adder.run();
    //handle_add_wallet(builder, sender);
}

pub fn initialize_wallet_selector(builder: &Builder, sender: &Sender<UIRequest>,app: &Application){
    let wallet_selector: ComboBoxText = builder.object("Wallet Switcher").unwrap();

    if let Err(error) = get_saved_wallets_from_disk(&wallet_selector){
        if let UiError::WalletsCSVWasEmpty = error{
            handle_initial_login(builder, sender, app);
        } else {
            //Poner ventana de error
        }
    }

    wallet_selector.set_active(Some(0));
}

pub fn initialize_wallet_adder_actions(builder: &Builder, sender: &Sender<UIRequest>){
    let wallet_adder: Dialog = builder.object("Wallet Adder Dialog").unwrap();
    let cancel_button: Button = builder.object("Wallet Adder Cancel Button").unwrap();
    let add_button: Button = builder.object("Wallet Adder Add Button").unwrap();
    let invalid_wallet_button: Button = builder.object("Wallet Adder Error Button").unwrap();
    let invalid_wallet_dialog: Dialog = builder.object("Wallet Adder Error Dialog").unwrap();
    let success_dialog: Dialog = builder.object("Wallet Adder Success Dialog").unwrap();
    let success_button: Button = builder.object("Wallet Adder Success Button").unwrap();

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

pub fn initialize_change_wallet(builder: &Builder, sender: &Sender<UIRequest>){
    let wallet_selector: ComboBoxText = builder.object("Wallet Switcher").unwrap();

    let sender_clone = sender.clone();
    wallet_selector.connect_changed(move |combo_box|{
        if let Some(active_text) = combo_box.active_text(){
            println!("{}",active_text);
            match sender_clone.send(UIRequest::ChangeWallet(combo_box.active_id().unwrap().to_string()) ){
                Ok(_) => {},
                Err(e) => println!("Error sending ChangeWallet request: {:?}", e),
            };
            sender_clone.send(UIRequest::LastBlockInfo).unwrap();
        }
    });
}