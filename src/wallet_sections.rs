use gtk::prelude::*;
use gtk::{Box,ComboBoxText,Builder,Entry,Dialog,Button};
use crate::UiError;
use crate::wallet_persistance::*;


const PRIV_KEY_LEN_BASE_58: usize = 1;

fn handle_add_wallet(builder: &Builder){
    let name: Entry = builder.object("Wallet Adder Name Entry").unwrap();
    let priv_key: Entry = builder.object("Wallet Adder Private Key Entry").unwrap();
    let invalid_key_dialog: Dialog = builder.object("Invalid Private Key Dialog").unwrap();
    let wallet_selector: ComboBoxText = builder.object("Wallet Switcher").unwrap();

    let priv_key_text = priv_key.text();
    let name_text = name.text();

    if priv_key_text.len() != PRIV_KEY_LEN_BASE_58 {
        invalid_key_dialog.show_all();
        invalid_key_dialog.run();
        return;
    }
    
    let success_dialog: Dialog = builder.object("Wallet Adder Success Dialog").unwrap();
    //let wallet = UIRequest::ChangeWallet(priv_key_text.try_into().unwrap());
    wallet_selector.append(Some(&priv_key_text),&name_text);
    
    name.set_text("");
    priv_key.set_text("");

    success_dialog.run();
    success_dialog.show_all();
    success_dialog.hide();

    if let Err(error) = save_wallet_in_disk(&priv_key_text, &name_text){
        //Poner ventana de error,
        println!("F");

    };
    
    //send wallet to wallet
}
pub fn initialize_wallet_selector(builder: &Builder){
    let wallet_selector: ComboBoxText = builder.object("Wallet Switcher").unwrap();

    if let Err(error) = get_saved_wallets_from_disk(&wallet_selector){
        if let UiError::WalletsCSVWasEmpty = error{
            let wallet_adder: Dialog = builder.object("Wallet Adder Dialog").unwrap();
            wallet_adder.run();
            // Si apretas cancel rompe to.
            // Si apretas la cruz se inicializa sin wallet.
            // Hacer que muera a menos que pongas ok con info valida.
            wallet_adder.show_all();
            wallet_adder.hide();
        } else {
            //Poner ventana de error
        }
    }

    wallet_selector.set_active(Some(0));
}

pub fn initialize_wallet_adder_actions(builder: &Builder) {
    let wallet_adder: Dialog = builder.object("Wallet Adder Dialog").unwrap();
    let cancel_button: Button = builder.object("Wallet Adder Cancel Button").unwrap();
    let add_button: Button = builder.object("Wallet Adder Add Button").unwrap();
    let invalid_key_button: Button = builder.object("Invalid Private Key Button").unwrap();
    let invalid_key_dialog: Dialog = builder.object("Invalid Private Key Dialog").unwrap();
    let success_dialog: Dialog = builder.object("Wallet Adder Success Dialog").unwrap();
    let success_button: Button = builder.object("Wallet Adder Success Button").unwrap();

    let wallet_adder_clone = wallet_adder.clone();
    cancel_button.connect_clicked(move |_| {
        wallet_adder.hide();
    });
    let builder_clone = builder.clone();
    add_button.connect_clicked(move |_| {
        handle_add_wallet(&builder_clone);
    });

    invalid_key_button.connect_clicked(move |_| {
        invalid_key_dialog.hide();
    });

    success_button.connect_clicked(move |_| {
        wallet_adder_clone.hide();
        success_dialog.hide();
    });
    
}
