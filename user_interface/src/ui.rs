use gtk::prelude::*;
use gtk::{glib, Application, ApplicationWindow, Builder, Box, Button, Dialog};
use std::sync::{mpsc, mpsc::Receiver, mpsc::Sender};
use gtk::glib::{Sender as S, Receiver as R};

use crate::wallet_transactions::add_row;
use crate::wallet_overview::update_available_balance;
use crate::wallet_overview::update_pending_balance;
use crate::wallet_send::update_balance;
use crate::wallet_send::activate_use_available_balance;
use crate::wallet_send::activate_clear_all_button;
pub use crate::utils::*;

pub enum UiError {
    FailedToBuildUi,
    FailedToFindObject,
}

pub fn start_app(){
    let builder = match obtain_builder(){
        Ok(builder) => builder,
        Err(_) => {
            return;
        },
    };

    initialize_elements(&builder);

    add_examples(&builder);
    start_window(&builder);
}

fn start_window(builder: &Builder){
    // Create Window
    let window: gtk::Window = match builder.object("Ventana"){
        Some(window) => window,
        None => return,
    };
    //cambiarle_el_color_al_separator(&builder);

    // Set close event
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(true)
    });

    // Show the window and call the main() loop of gtk
    window.show_all();
    gtk::main();
}

fn obtain_builder() -> Result<Builder,UiError>{
    // Initialise gtk components
    if gtk::init().is_err() {
        return Err(UiError::FailedToBuildUi)
    }
    let glade_src = include_str!("ui.glade");

    // Load glade file
    Ok(Builder::from_string(&glade_src))
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