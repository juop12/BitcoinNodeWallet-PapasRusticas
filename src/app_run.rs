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
    thread::JoinHandle,
};

type SafeGlibReceiver = Arc<Mutex<Option<GlibReceiver<UIResponse>>>>;

fn ui_response_to_message(builder: Builder,
    action: UIResponse,
    app: Application,
    sender: &Sender<UIRequest>,
    window_running: Arc<Mutex<bool>>,
    update_wallet_join_handle: Arc<Mutex<Option<JoinHandle<()>>>>
){
    match action {
        UIResponse::ResultOFTXProof(result) => handle_result_of_tx_proof(&builder, result),
        UIResponse::WalletInfo(wallet_info) => handle_wallet_info(&wallet_info, &builder),
        UIResponse::BlockInfo(block_info) => handle_block_info(&block_info, &builder),
        UIResponse::FinishedInitializingNode => {
            match start_window(&app, &builder, &sender, window_running.clone()){
                Ok(join_handle) => {
                    update_wallet_join_handle.lock().unwrap().replace(join_handle);
                },
                Err(error) => {
                    handle_ui_error(&builder, error);
                    return;
                }
            }    
        }
        UIResponse::WalletFinished => handle_app_finished(&app, window_running.clone()),
        UIResponse::TxSent => handle_tx_sent(&builder),
        UIResponse::WalletError(error) => handle_wallet_error(&builder, error, window_running),
        UIResponse::ErrorInitializingNode => handle_initialization_error(&builder, &app),
        UIResponse::LoadingScreenUpdate(progress) => handle_loading_screen_update(&builder, progress),
    }
}

/// Called by the main program, it initializes the UI and starts the main loop
/// creating a thread for the node and running the UI in the main thread, and
/// creating glib channels for the node to communicate with the UI and mpsc
/// channels for the UI to communicate with the node.
fn run_app(
    app: &Application,
    glade_src: &str,
    sender: Sender<UIRequest>,
    safe_receiver: SafeGlibReceiver,
) {
    // Create Window
    let builder = Builder::from_string(glade_src);

    let glib_receiver = match safe_receiver.lock() {
        Ok(mut receiver) => receiver.take(),
        Err(_) => return,
    };
    let glib_receiver = match glib_receiver {
        Some(receiver) => receiver,
        None => return,
    };
    let main_window_running = Arc::new(Mutex::new(false));
    show_loading_screen(&builder, app);
    let sender_clone = sender.clone();
    let builder_clone = builder.clone();
    let app_cloned = app.clone();
    let update_wallet_join_handle: Arc<Mutex<Option<JoinHandle<()>>>> = Arc::new(Mutex::from(None));
    let join_handle_for_finishing = update_wallet_join_handle.clone();
    glib_receiver.attach(None, move |action| {
        ui_response_to_message(builder.clone(), action, app_cloned.clone(), &sender, main_window_running.clone(), update_wallet_join_handle.clone());
        glib::Continue(true)
    });
    
    app.connect_shutdown(move |_| {
        if sender_clone.send(UIRequest::EndOfProgram).is_ok() {
            let window = builder_clone.object::<Window>("Main Window").expect("Couldn't find main window");
            window.close();
            match join_handle_for_finishing.lock().unwrap().take() {
                Some(join_handle) => {
                    if let Err(error) = join_handle.join() {
                        eprintln!("Error joining thread: {:#?}", error);
                    };
                }
                None => (),
            }
        };
    });
}

/// Closes the initial loading window when the node finishes initializing
fn close_loading_window(builder: &Builder) {
    let window: Window = builder.object("Loading Screen Window").expect("Failed to find main window");
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
) -> Result<JoinHandle<()>, UiError> {
    match running.lock() {
        Ok(mut running) => *running = true,
        Err(_) => return Err(UiError::FailedToBuildUi)
    }
    initialize_elements(builder, sender); 
    close_loading_window(builder);
    let window: Window = builder.object("Main Window").expect("Failed to find main window");
    window.set_application(Some(app));
    window.show_all();
    Ok(send_ui_update_request(sender, running))
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
    let safe_receiver: SafeGlibReceiver = Arc::new(Mutex::from(Some(glib_receiver)));
    let (sender, receiver) = mpsc::channel::<UIRequest>();
    let join_handle = thread::spawn(move || run(args, glib_sender.clone(), receiver));

    application.connect_activate(move |app| run_app(app, glade_src, sender.clone(), safe_receiver.clone()));

    let vector: Vec<String> = Vec::new();
    application.run_with_args(&vector);

    if let Err(error) = join_handle.join() {
        eprintln!("Error joining thread: {:#?}", error);
    };
}
