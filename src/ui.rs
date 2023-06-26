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


const SENDER_ERROR: &str = "Error sending message to node through mpsc channel";
//const PRIV_KEY_LEN_BASE_58: usize = 52;


#[derive(Debug)]
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
    let (glib_sender, glib_receiver) = glib::MainContext::channel::<UIResponse>(glib::PRIORITY_DEFAULT);
    let (sender, receiver) = mpsc::channel::<UIRequest>();
    thread::spawn(move || {run(args, glib_sender.clone(), receiver)});
    show_loading_screen(&builder, &sender, &app);
    //initialize_elements(&builder, &sender, app);
    let sender_clone = sender.clone();
    let program_running = Arc::new(Mutex::from(true));
    let program_running_cloned = program_running.clone();
    let builder_clone = builder.clone();
    let app_cloned = app.clone();
    //start_window(app, &builder);

    glib_receiver.attach(None, move|action| {
        match action {
            UIResponse::ResultOFTXProof(result) => handle_result_of_tx_proof(&builder, result),
            UIResponse::WalletInfo(wallet_info) => handle_wallet_info(&wallet_info, &builder),
            UIResponse::BlockInfo(block_info) => handle_block_info(&block_info, &builder),
            UIResponse::FinishedInitializingNode => start_window(&app_cloned, &builder, &sender),
            UIResponse::WalletFinished => app_cloned.quit(),
            UIResponse::TxSent => handle_tx_sent(&builder, &sender),
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

fn close_loading_window(builder: &Builder){
    let window: Window = builder.object("Loading Screen Window").unwrap();
    window.close();
}

fn start_window(app: &Application, builder: &Builder, sender: &Sender<UIRequest>) {
    initialize_elements(&builder, &sender, app);
    close_loading_window(builder);
    let window: Window = builder.object("Ventana").unwrap();
    window.set_application(Some(app));
    window.show_all();
}

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


fn activate_wallet_adder(builder: &Builder){
    let button: Button = builder.object("Wallet Adder").unwrap();
    let wallet_adder: Dialog = builder.object("Wallet Adder Dialog").unwrap();
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
/* pub enum WalletError {
    ErrorHandlingPrivKey,
    ErrorHandlingAddress,
    ErrorSendingTx,
    ErrorCreatingTx,
    ErrorNotEnoughSatoshis,
    ErrorSendingToUI,
    ErrorSettingWallet,
    ErrorFindingBlock,
    ErrorGettingBlockInfo,
    ErrorObtainingTxProof,
    ErrorReceivingFromUI,
    ErrorUpdatingWallet,
} */

pub fn handle_error(builder: &Builder, text: String){
    
    let err_button: Button = builder.object("Error Button").unwrap();
    let err_dialog: Dialog = builder.object("Error Dialog").unwrap();
    let err_label: Label = builder.object("Error Label").unwrap();
    let err_clone = err_dialog.clone();
    err_label.set_text(text.as_str());
    err_button.connect_clicked(move |_| {
        err_clone.hide();
    });
    err_dialog.set_title("Error");
    err_dialog.show_all();
    err_dialog.run();

}

fn handle_initialization_error(builder: &Builder, app: &Application){
    let err_button: Button = builder.object("Error Button").unwrap();
    let err_label: Label = builder.object("Error Label").unwrap();
    let err_dialog: Dialog = builder.object("Error Dialog").unwrap();
    let err_clone = err_dialog.clone(); 
    err_button.connect_clicked(move |_| {
        err_clone.hide();
    });

    err_label.set_text("There was an error initializing the node");
    err_dialog.set_title("Error Initializing Node");
    err_dialog.show_all();
    err_dialog.run();
    app.quit();
}