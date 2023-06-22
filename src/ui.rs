use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, ApplicationInhibitFlags, Builder, Box, Button, Dialog, Window};
use std::sync::{mpsc, mpsc::Receiver, mpsc::Sender};
use std::thread;
use glib::{Sender as GlibSender, Receiver as GlibReceiver};
use std::sync::mpsc::{Sender as S, Receiver as R};
use crate::wallet_transactions::add_row;
use crate::wallet_overview::update_available_balance;
use crate::wallet_overview::update_pending_balance;
use crate::wallet_send::update_balance;
use crate::wallet_send::activate_use_available_balance;
use crate::wallet_send::activate_clear_all_button;
use node::run::*;
use node::utils::ui_communication::{UIWalletCommunicationProtocol as UiActions};

pub enum UiError {
    FailedToBuildUi,
    FailedToFindObject,
}

pub fn start_app(args: Vec<String>){
    let glade_src = include_str!("ui.glade");
    let application = Application::builder().build();
    application.connect_activate(move |app| run_app(app, glade_src, args.clone()));
    let vector: Vec<String> = Vec::new();
    application.run_with_args(&vector);
}

fn run_app(app: &Application, glade_src: &str, args: Vec<String>){
    // Create Window
    let builder = Builder::from_string(glade_src);
    initialize_elements(&builder);
    start_window(app, &builder);
    let (glib_sender, glib_receiver) = glib::MainContext::channel::<UiActions>(glib::PRIORITY_DEFAULT);
    let (sender, receiver) = mpsc::channel::<String>(); //p por ahora String, despues le definimos bien el tipo de dato
    thread::spawn(move || {run(args, glib_sender.clone(), receiver)});
    glib_receiver.attach(None, |action| {
        match action {
            UiActions::NodeRunningError(error) => {glib::Continue(true)},
            _ => glib::Continue(true),
        };
        glib::Continue(true)
    });
    // Show the window and call the main() loop of gtk
    //window.show_all();
}

fn start_window(app: &Application, builder: &Builder) {
    let window: Window = builder.object("Ventana").unwrap();
    // window.connect_delete_event(|_, _| {
    //     app.quit();
    //     Inhibit(true)
    // });
    window.set_application(Some(app));
    window.show_all();
}

fn initialize_elements(builder: &Builder){
    activate_wallet_adder(builder);
    activate_use_available_balance(builder);
    activate_clear_all_button(builder);
}

//Editar para hacer pruebas con diferentes valores
fn add_examples(builder: &Builder){
    update_available_balance(&builder, "15.00");
    update_pending_balance(&builder, "5.01");
    add_tx(&builder, "lorem ipsum".to_string());
    update_balance(&builder, "69.420");
}

pub fn add_tx(builder: &Builder, transaction: String) {
    let tx_tree_store = builder.object("TxTreeStore").unwrap();
    let date = "0".to_string();
    let amount = "0".to_string();
    add_row(tx_tree_store, date, transaction, amount)
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