use crate::activate_adjustments;
use crate::error_handling::*;
use crate::loading_screen::*;
use crate::wallet_actions::*;
use crate::wallet_adder::*;
use crate::wallet_send::{
    activate_clear_all_button, activate_send_button, activate_use_available_balance,
    update_adjustments_max_value,
};
use crate::wallet_transactions::*;
use glib::Receiver as GlibReceiver;
use gtk::prelude::*;
use gtk::{Application, Builder, Window};
use node::run::*;
use node::utils::ui_communication_protocol::{UIRequest, UIResponse};
use std::{
    sync::{mpsc, mpsc::Sender, Arc, Mutex},
    thread,
};

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
fn run_app(
    app: &Application,
    glade_src: &str,
    sender: Sender<UIRequest>,
    a: Arc<Mutex<Option<GlibReceiver<UIResponse>>>>,
) {
    // Create Window
    let builder = Builder::from_string(glade_src);

    let glib_receiver = a.lock().unwrap().take();
    let glib_receiver = match glib_receiver {
        Some(receiver) => receiver,
        None => return,
    };
    let main_window_running = Arc::new(Mutex::new(false));
    show_loading_screen(&builder, app);
    let sender_clone = sender.clone();
    let builder_clone = builder.clone();
    let app_cloned = app.clone();

    glib_receiver.attach(None, move |action| {
        match action {
            UIResponse::ResultOFTXProof(result) => handle_result_of_tx_proof(&builder, result),
            UIResponse::WalletInfo(wallet_info) => handle_wallet_info(&wallet_info, &builder),
            UIResponse::BlockInfo(block_info) => handle_block_info(&block_info, &builder),
            UIResponse::FinishedInitializingNode => {
                start_window(&app_cloned, &builder, &sender, main_window_running.clone())
            }
            UIResponse::WalletFinished => {
                handle_app_finished(&app_cloned, main_window_running.clone())
            }
            UIResponse::TxSent => handle_tx_sent(&builder),
            UIResponse::WalletError(error) => {
                handle_error(&builder, format!("An Error occured: {:#?}", error))
            }
            UIResponse::ErrorInitializingNode => handle_initialization_error(&builder, &app_cloned),
            UIResponse::LoadingScreenUpdate(progress) => handle_loading_screen_update(&builder, progress),
        }
        glib::Continue(true)
    });

    app.connect_shutdown(move |_| {
        if sender_clone.send(UIRequest::EndOfProgram).is_ok() {
            let window = builder_clone.object::<Window>("Ventana").unwrap();
            window.close();
        };
    });
}

/// Closes the initial loading window when the node finishes initializing
fn close_loading_window(builder: &Builder) {
    let window: Window = builder.object("Loading Screen Window").unwrap();
    window.close();
}

/// Starts the main elemente of the UI and shows the main window of the program
/// after the node has finished initializing, this window is the one that
/// the user sees
fn start_window(
    app: &Application,
    builder: &Builder,
    sender: &Sender<UIRequest>,
    running: Arc<Mutex<bool>>,
) {
    match running.lock() {
        Ok(mut running) => *running = true,
        Err(_) => return,
    }
    initialize_elements(builder, sender);
    close_loading_window(builder);
    let window: Window = builder.object("Ventana").unwrap();
    window.set_application(Some(app));
    send_ui_update_request(sender, running);
    window.show_all();
}

/// Defines the important signals and actions of the UI elements, such as buttons, sliders,
/// adding wallets, etc
fn initialize_elements(builder: &Builder, sender: &Sender<UIRequest>) {
    activate_wallet_adder(builder);
    activate_use_available_balance(builder);
    activate_clear_all_button(builder);
    activate_adjustments(builder);
    initialize_wallet_adder_actions(builder, sender);
    connect_block_switcher_buttons(builder, sender);
    activate_send_button(builder, sender);
    initialize_wallet_selector(builder, sender);
    initialize_change_wallet(builder, sender);
    initialize_merkle_proof_button(builder, sender);
    update_adjustments_max_value(builder);
}

/// Initializes the application that runs the whole program
pub fn start_app(args: Vec<String>) {
    let glade_src = include_str!("glade/ui.glade");
    let application = Application::builder().build();

    let (glib_sender, glib_receiver) =
        glib::MainContext::channel::<UIResponse>(glib::PRIORITY_DEFAULT);
    let arc = Arc::new(Mutex::from(Some(glib_receiver)));
    let (sender, receiver) = mpsc::channel::<UIRequest>();
    let join_handle = thread::spawn(move || run(args, glib_sender.clone(), receiver));

    application.connect_activate(move |app| run_app(app, glade_src, sender.clone(), arc.clone()));

    let vector: Vec<String> = Vec::new();
    application.run_with_args(&vector);

    if let Err(error) = join_handle.join() {
        eprintln!("Error joining thread: {:#?}", error);
    };
}
