use gtk::gdk::keys::constants::O;
use gtk::prelude::*;
use gtk::{Application, Builder, Button, Dialog, Window, Label};
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use glib::{Sender as GlibSender, Receiver as GlibReceiver};
use node::utils::btc_errors::WalletError;
use crate::activate_adjustments;
use crate::wallet_adder::{
    initialize_wallet_adder_actions,initialize_wallet_selector, 
    initialize_change_wallet
};
use crate::wallet_send::{
    update_adjustments_max_value, activate_use_available_balance, activate_clear_all_button,
    activate_send_button
};
use crate::wallet_actions::*;
use node::run::*;
use node::utils::ui_communication_protocol::{
    UIToWalletCommunication as UIRequest, WalletToUICommunication as UIResponse
};
use crate::wallet_transactions::{initialize_merkle_proof_button, handle_result_of_tx_proof};
use crate::loading_screen::show_loading_screen;
use crate::error_handling::*;


const SENDER_ERROR: &str = "Error sending message to node through mpsc channel";
//const PRIV_KEY_LEN_BASE_58: usize = 52;

/// Enum that represents the possible errors that can happen in the UI
#[derive(Debug)]
pub enum UiError {
    FailedToBuildUi,
    FailedToFindObject,
    ErrorReadingFile,
    ErrorWritingFile,
    WalletsCSVWasEmpty,
}

/// Called by the main program, it initializes the UI and starts the main loop
/// creating a thread for the node and running the UI in the main thread, and
/// creating glib channels for the node to communicate with the UI and mpsc
/// channels for the UI to communicate with the node.
fn run_app(app: &Application, glade_src: &str, args: Vec<String>){
    let builder = Builder::from_string(glade_src);
    let (glib_sender, glib_receiver) = glib::MainContext::channel::<UIResponse>(glib::PRIORITY_DEFAULT);
    let (sender, receiver) = mpsc::channel::<UIRequest>();
    thread::spawn(move || {run(args, glib_sender.clone(), receiver)});
    show_loading_screen(&builder, &sender, &app);
    let sender_clone = sender.clone();
    let program_running = Arc::new(Mutex::from(true));
    let program_running_cloned = program_running.clone();
    let builder_clone = builder.clone();
    let app_cloned = app.clone();

    glib_receiver.attach(None, move|action| {
        match action {
            UIResponse::ResultOFTXProof(result) => handle_result_of_tx_proof(&builder, result),
            UIResponse::WalletInfo(wallet_info) => handle_wallet_info(&wallet_info, &builder),
            UIResponse::BlockInfo(block_info) => handle_block_info(&block_info, &builder),
            UIResponse::FinishedInitializingNode => start_window(&app_cloned, &builder, &sender),
            UIResponse::WalletFinished => app_cloned.quit(),
            UIResponse::TxSent => handle_tx_sent(&builder),
            UIResponse::WalletError(error) => handle_error(&builder, format!("An Error occured: {:#?}",  error)),
            UIResponse::ErrorInitializingNode => handle_initialization_error(&builder, &app_cloned),
            _ => {},
        }
        glib::Continue(true)
    });

    app.connect_shutdown(move |_| {
        if let Ok(_) = sender_clone.send(UIRequest::EndOfProgram){
            let window = builder_clone.object::<Window>("Ventana").unwrap();
            window.close();
            *program_running_cloned.lock().unwrap() = false;
        };
    });
    
}

/// Closes the initial loading window when the node finishes initializing
fn close_loading_window(builder: &Builder){
    let window: Window = builder.object("Loading Screen Window").unwrap();
    window.close();
}

/// Starts the main elemente of the UI and shows the main window of the program
/// after the node has finished initializing, this window is the one that
/// the user sees
fn start_window(app: &Application, builder: &Builder, sender: &Sender<UIRequest>) {
    initialize_elements(&builder, &sender, app);
    close_loading_window(builder);
    let window: Window = builder.object("Ventana").unwrap();
    window.set_application(Some(app));
    window.show_all();
}

/// Defines the important signals and actions of the UI elements, such as buttons, sliders, 
/// adding wallets, etc
fn initialize_elements(builder: &Builder, sender: &Sender<UIRequest>, app: &Application){
    activate_wallet_adder(builder);
    activate_use_available_balance(builder);
    activate_clear_all_button(builder);
    activate_adjustments(builder);
    update_adjustments_max_value(builder);
    initialize_wallet_adder_actions(builder, sender);
    connect_block_switcher_buttons(builder, sender);
    activate_send_button(builder, sender);
    initialize_wallet_selector(builder, sender, app);
    initialize_change_wallet(builder, sender);
    initialize_merkle_proof_button(builder, sender);
}

/// Connects the buttons that allow the user to switch between blocks
fn connect_block_switcher_buttons(builder: &Builder, sender: &Sender<UIRequest>){
    let next_button: Button = builder.object("Next Block Button").unwrap();
    let previous_button: Button = builder.object("Previous Block Button").unwrap();
    let sender_clone = sender.clone();
    let sender_clone_2 = sender.clone();
    next_button.connect_clicked(move |_| {
        sender_clone_2.send(UIRequest::NextBlockInfo).expect(SENDER_ERROR);
    });

    previous_button.connect_clicked(move |_| {
        sender_clone.send(UIRequest::PrevBlockInfo).expect(SENDER_ERROR);
    });
}

/// Initializes the wallet selector, which lets the user add a wallet introducing a Name
/// and a private key
fn activate_wallet_adder(builder: &Builder){
    let button: Button = builder.object("Wallet Adder").unwrap();
    let wallet_adder: Dialog = builder.object("Wallet Adder Dialog").unwrap();
    button.connect_clicked(move |_| {
        wallet_adder.show_all();
        wallet_adder.run();
        wallet_adder.hide();
    });
}

/// Initializes the application that runs the whole program
pub fn start_app(args: Vec<String>){
    let glade_src = include_str!("ui.glade");
    let application = Application::builder().build();
    application.connect_activate(move |app| run_app(app, glade_src, args.clone()));
    let vector: Vec<String> = Vec::new();
    application.run_with_args(&vector);
    
}