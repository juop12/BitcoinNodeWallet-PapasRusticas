use gio::builders;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, ApplicationInhibitFlags, Builder, Box, Button, Dialog, Entry,Window, Adjustment, Label, SpinButton };
use std::alloc::handle_alloc_error;
use std::sync::mpsc::{Receiver, Sender};
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;
use glib::{Sender as GlibSender, Receiver as GlibReceiver};
use crate::activate_adjustments;
use crate::wallet_sections::{initialize_wallet_adder_actions,initialize_wallet_selector};
use crate::wallet_transactions::add_row;
use crate::wallet_overview::update_available_balance;
use crate::wallet_overview::update_pending_balance;
use crate::wallet_send::{update_balance, update_adjustments_max_value, activate_use_available_balance, activate_clear_all_button};
use crate::wallet_actions::*;

use node::run::*;
use node::utils::ui_communication::{UIToWalletCommunication as UIRequest, WalletToUICommunication as UIResponse};

//const PRIV_KEY_LEN_BASE_58: usize = 52;
pub enum UiError {
    FailedToBuildUi,
    FailedToFindObject,
    ErrorReadingFile,
    ErrorWritingFile,
    WalletsCSVWasEmpty,
}

fn run_app(app: &Application, glade_src: &str, args: Vec<String>){
    // Create Window
    let builder = Builder::from_string(glade_src);
    start_window(app, &builder);
    let (glib_sender, glib_receiver) = glib::MainContext::channel::<UIResponse>(glib::PRIORITY_DEFAULT);
    let (sender, receiver) = mpsc::channel::<UIRequest>(); //p por ahora String, despues le definimos bien el tipo de dato
    initialize_elements(&builder, sender);
    thread::spawn(move || {run(args, glib_sender.clone(), receiver)});
    glib_receiver.attach(None, move|action| {
        match action {
            UIResponse::NodeRunningError(_error) => {},
            UIResponse::BlockInfo(block_info) => handle_block_info(&block_info, &builder),
            UIResponse::WalletInfo(wallet_info) => handle_wallet_info(&wallet_info, &builder),
            _ => {},
        }
        glib::Continue(true)
    });
}

fn start_window(app: &Application, builder: &Builder) {
    let window: Window = builder.object("Ventana").unwrap();
    window.set_application(Some(app));
    window.show_all();
}

fn initialize_elements(builder: &Builder, sender: Sender<UIRequest>){
    //add_examples(builder);
    activate_wallet_adder(builder);
    activate_use_available_balance(builder);
    activate_clear_all_button(builder);
    activate_adjustments(builder);
    update_adjustments_max_value(builder);
    initialize_wallet_adder_actions(builder);
    initialize_wallet_selector(builder);
    connect_block_switcher_buttons(builder, sender);
}

fn connect_block_switcher_buttons(builder: &Builder, sender: Sender<UIRequest>){
    let next_button: Button = builder.object("Next Block Button").unwrap();
    let previous_button: Button = builder.object("Previous Block Button").unwrap();
    let sender_clone = sender.clone();
    next_button.connect_clicked(move |_| {
        sender.send(UIRequest::NextBlockInfo).unwrap();
    });

    previous_button.connect_clicked(move |_| {
        sender_clone.send(UIRequest::PrevBlockInfo).unwrap();
    });
}

//Editar para hacer pruebas con diferentes valores
fn add_examples(builder: &Builder){
    update_available_balance(builder, "15.00");
    update_pending_balance(builder, "5.01");
    update_balance(builder, "69.420");
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

fn activate_tx_send(builder: &Builder){
    let button: Button = builder.object("Send Button").unwrap();
    let amount_button: SpinButton = builder.object("Send Amount").unwrap();
    let fee_amount_button: SpinButton = builder.object("Fee Amount").unwrap();
    let balance: Label = builder.object("Send Balance Text").unwrap();

    let address_entry: Entry = builder.object("Pay To Entry").unwrap();

    let send_amount : f64 = amount_button.value();
    let fee_amount : f64 = fee_amount_button.value();

    

    let address: String = address_entry.text().to_string();

    button.connect_clicked(move |_| {
        if (send_amount + fee_amount) > balance.text().parse::<f64>().unwrap() {
            //error
        }
        if address.len() != 34 {
            //error
        }
        

    });
}

pub fn start_app(args: Vec<String>){
    let glade_src = include_str!("ui.glade");
    let application = Application::builder().build();
    application.connect_activate(move |app| run_app(app, glade_src, args.clone()));
    let vector: Vec<String> = Vec::new();
    application.run_with_args(&vector);
}