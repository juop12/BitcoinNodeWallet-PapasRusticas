use gtk::prelude::*;
use gtk::{Application, Builder, Button, Dialog, Entry,Window, Adjustment, Label, SpinButton };
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use glib::{Sender as GlibSender, Receiver as GlibReceiver};
use crate::activate_adjustments;
use crate::wallet_sections::{initialize_wallet_adder_actions,initialize_wallet_selector};
use crate::wallet_send::{update_balance, update_adjustments_max_value, activate_use_available_balance, activate_clear_all_button,activate_send_button};
use crate::wallet_actions::*;
use node::run::*;
use node::utils::ui_communication_protocol::{UIToWalletCommunication as UIRequest, WalletToUICommunication as UIResponse, };

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
    let (sender, receiver) = mpsc::channel::<UIRequest>();
    initialize_elements(&builder, &sender);
    let join_handle = Arc::new(Mutex::from(thread::spawn(move || {run(args, glib_sender.clone(), receiver)})));
    let sender_clone = sender.clone();
    let program_running = Arc::new(Mutex::from(true));
    let program_running_cloned = program_running.clone();
    let builder_clone = builder.clone();
    let app_cloned = app.clone();
    glib_receiver.attach(None, move|action| {
        match action {
            UIResponse::NodeRunningError(_error) => {},
            UIResponse::BlockInfo(block_info) => handle_block_info(&block_info, &builder),
            UIResponse::WalletInfo(wallet_info) => handle_wallet_info(&wallet_info, &builder),
            UIResponse::TxSent => handle_tx_sent(&builder, &sender),
            UIResponse::WalletFinished => app_cloned.quit(),
            _ => {},
        }
        glib::Continue(true)
    });
    app.connect_shutdown(move |_| {
        sender_clone.send(UIRequest::EndOfProgram).unwrap();
        let window = builder_clone.object::<Window>("Ventana").unwrap();
        window.close();
        *program_running_cloned.lock().unwrap() = false;
    });
    
}

fn start_window(app: &Application, builder: &Builder) {
    let window: Window = builder.object("Ventana").unwrap();
    window.set_application(Some(app));
    window.show_all();
}

fn initialize_elements(builder: &Builder, sender: &Sender<UIRequest>){
    activate_wallet_adder(builder);
    activate_use_available_balance(builder);
    activate_clear_all_button(builder);
    activate_adjustments(builder);
    update_adjustments_max_value(builder);
    initialize_wallet_adder_actions(builder, sender);
    initialize_wallet_selector(builder);
    connect_block_switcher_buttons(builder, sender);
    activate_send_button(builder, sender);
}

fn connect_block_switcher_buttons(builder: &Builder, sender: &Sender<UIRequest>){
    let next_button: Button = builder.object("Next Block Button").unwrap();
    let previous_button: Button = builder.object("Previous Block Button").unwrap();
    let sender_clone = sender.clone();
    let sender_clone_2 = sender.clone();
    next_button.connect_clicked(move |_| {
        sender_clone_2.send(UIRequest::NextBlockInfo).unwrap();
    });

    previous_button.connect_clicked(move |_| {
        sender_clone.send(UIRequest::PrevBlockInfo).unwrap();
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


pub fn start_app(args: Vec<String>){
    let glade_src = include_str!("ui.glade");
    let application = Application::builder().build();
    application.connect_activate(move |app| run_app(app, glade_src, args.clone()));
    let vector: Vec<String> = Vec::new();
    application.run_with_args(&vector);
    
}