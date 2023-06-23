use gio::builders;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, ApplicationInhibitFlags, Builder, Box, Button, Dialog, Entry,Window, Adjustment, Label, ComboBoxText};
use std::alloc::handle_alloc_error;
use std::sync::mpsc::{Receiver, Sender};
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;
use glib::{Sender as GlibSender, Receiver as GlibReceiver};
use crate::activate_adjustments;
use crate::wallet_transactions::add_row;
use crate::wallet_overview::update_available_balance;
use crate::wallet_overview::update_pending_balance;
use crate::wallet_send::update_balance;
use crate::wallet_send::activate_use_available_balance;
use crate::wallet_send::activate_clear_all_button;
use crate::wallet_persistance::*;
use node::run::*;
use node::utils::ui_communication::{UIToWalletCommunication as UIRequest, WalletToUICommunication as UIResponse};


const GLADE_SRC: &str = "ui.glade";
const PRIV_KEY_LEN_BASE_58: usize = 52;


pub enum UiError {
    FailedToBuildUi,
    FailedToFindObject,
    ErrorReadingFile,
    ErrorWritingFile,
}

fn run_app(app: &Application, glade_src: &str, args: Vec<String>){
    // Create Window
    let builder = Builder::from_string(glade_src);
    initialize_elements(&builder);
    start_window(app, &builder);
    let (glib_sender, glib_receiver) = glib::MainContext::channel::<UIResponse>(glib::PRIORITY_DEFAULT);
    let (sender, receiver) = mpsc::channel::<UIRequest>(); //p por ahora String, despues le definimos bien el tipo de dato
    //thread::spawn(move || {run(args, glib_sender.clone(), receiver)});
    glib_receiver.attach(None, |action| {
        match action {
            UIResponse::NodeRunningError(error) => {glib::Continue(true)},
            _ => glib::Continue(true),
        };
        glib::Continue(true)
    });
    // Show the window and call the main() loop of gtk
    //window.show_all();
}

fn start_window(app: &Application, builder: &Builder) {
    let window: Window = builder.object("Ventana").unwrap();
    window.set_application(Some(app));
    window.show_all();
}

fn initialize_elements(builder: &Builder){
    add_examples(builder);
    activate_wallet_adder(builder);
    activate_use_available_balance(builder);
    activate_clear_all_button(builder);
    activate_adjustments(builder);
    update_adjustments_max_value(builder);
    wallet_adder_actions(builder);
}

fn update_adjustments_max_value(builder: &Builder){
    let balance_amount: Label = match builder.object("BalanceAmount"){
        Some(balance_label) => balance_label,
        None => return,
    };
    let send_amount_adjustment: Adjustment = match builder.object("Amount Adjustment"){
        Some(adjustment) => adjustment,
        None => return,
    };
    let fee_amount_adjustment: Adjustment = match builder.object("Fee Adjustment"){
        Some(adjustment) => adjustment,
        None => return,
    };
    send_amount_adjustment.set_upper(balance_amount.label().parse::<f64>().unwrap_or(0.0));
    fee_amount_adjustment.set_upper(balance_amount.label().parse::<f64>().unwrap_or(0.0));
}

//Editar para hacer pruebas con diferentes valores
fn add_examples(builder: &Builder){
    update_available_balance(&builder, "15.00");
    update_pending_balance(&builder, "5.01");
    add_tx(&builder, "lorem ipsum".to_string());
    update_balance(&builder, "69.420");
    if let Err(_) = get_saved_wallets_from_disk(builder){
        //Poner ventana de error
    }
}

pub fn add_tx(builder: &Builder, transaction: String) {
    let tx_tree_store = builder.object("TxTreeStore").unwrap();
    let date = "0".to_string();
    let amount = "0".to_string();
    add_row(tx_tree_store, date, transaction, amount)
}

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

    if let Err(error) = save_wallet_in_disk(&priv_key_text, &name_text){
        //Poner ventana de error,
        println!("F");

    };
    
    //send wallet to wallet
}

fn wallet_adder_actions(builder: &Builder) {
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

fn activate_wallet_adder(builder: &Builder){
    let button: Button = match builder.object("Wallet Adder"){
        Some(button) => button,
        None => return,
    };
    let wallet_adder: Dialog = match builder.object("Wallet Adder Dialog"){
        Some(wallet_adder) => wallet_adder,
        None => return,
    };
    button.connect_clicked(move |_| {
        wallet_adder.show_all();
        wallet_adder.run();
        wallet_adder.hide();
    });
}

// fn obtain_wallet_update(builder: &Builder, sender: Sender<String>) {
//     //send update request to wallet
    
//     //receive
//     //update_available_balance(builder, amount1);
//     //update_pending_balance(builder, amount2);
    

    
// }


pub fn start_app(args: Vec<String>){
    let glade_src = include_str!("ui.glade");
    let application = Application::builder().build();
    application.connect_activate(move |app| run_app(app, glade_src, args.clone()));
    let vector: Vec<String> = Vec::new();
    application.run_with_args(&vector);
}