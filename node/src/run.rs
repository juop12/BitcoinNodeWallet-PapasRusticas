use crate::utils::ui_communication_protocol::{
    UIRequest, UIResponse,
};
use crate::{node::*, utils::config::*, utils::WalletError, wallet::*};
use glib::Sender as GlibSender;
use std::sync::mpsc;

/// Creates a new node correctely and sets workers for receiving messages from other peers.
/// If an error occurs it returns None.
pub fn initialize_node(args: Vec<String>, sender_to_ui: GlibSender<UIResponse>) -> Option<Node> {
    if args.len() != 2 {
        eprintln!("Cantidad de argumentos inválida");
        return None;
    }

    let config = match Config::from_path(args[1].as_str()) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("Error ConfigError: {:?}", error);
            return None;
        }
    };

    let mut node = match Node::new(config, sender_to_ui) {
        Ok(node) => node,
        Err(error) => {
            eprintln!("Error creando el nodo NodeError: {:?}", error);
            return None;
        }
    };

    if let Err(error) = node.initial_block_download() {
        eprintln!("Error IBD: {:?}", error);
        return None;
    };

    if let Err(error) = node.create_utxo_set() {
        eprintln!("Error creating UTXO set: {:?}", error);
        return None;
    };

    node.start_receiving_messages();

    Some(node)
}

/// Makes sure that the wallet functionality starts running after the first
/// ever wallet is requested and created correctely. If an error h
fn get_first_wallet(
    node: &mut Node,
    receiver: &mpsc::Receiver<UIRequest>,
) -> Result<Option<Wallet>, WalletError> {
    loop {
        let ui_request = receiver
            .recv()
            .map_err(|_| WalletError::ErrorReceivingFromUI)?;

        match ui_request {
            UIRequest::ChangeWallet(priv_key) => {
                let mut wallet = Wallet::from(priv_key)?;

                node.set_wallet(&mut wallet)
                    .map_err(|_| WalletError::ErrorSettingWallet)?;

                return Ok(Some(wallet));
            }
            UIRequest::EndOfProgram => return Ok(None),
            _ => {}
        }
    }
}

/// Wallet receives messages from the ui and handles them.
fn run_main_loop(
    node: &mut Node,
    mut wallet: Wallet,
    receiver: &mpsc::Receiver<UIRequest>,
    sender_to_ui: &GlibSender<UIResponse>,
) -> Result<(), WalletError> {
    let mut program_running = true;

    while program_running {
        let request = receiver
            .recv()
            .map_err(|_| WalletError::ErrorReceivingFromUI)?;
        wallet = wallet.handle_ui_request(node, request, sender_to_ui, &mut program_running)?;
    }

    Ok(())
}

pub fn run(
    args: Vec<String>,
    sender_to_ui: GlibSender<UIResponse>,
    receiver: mpsc::Receiver<UIRequest>,
) {
    let mut node = match initialize_node(args, sender_to_ui.clone()) {
        Some(node) => node,
        None => return exit_program(sender_to_ui, UIResponse::ErrorInitializingNode),
    };

    if let Err(error) = sender_to_ui.send(UIResponse::FinishedInitializingNode) {
        return eprintln!("Error sending_to_ui: {:?}", error);
    }

    node.logger.log("node, running".to_string());

    let wallet = match get_first_wallet(&mut node, &receiver) {
        Ok(wallet) => match wallet {
            Some(wallet) => wallet,
            None => return exit_program(sender_to_ui, UIResponse::WalletFinished),
        },
        Err(error) => return exit_program(sender_to_ui, UIResponse::WalletError(error)),
    };

    if let Err(error) = run_main_loop(&mut node, wallet, &receiver, &sender_to_ui) {
        return exit_program(sender_to_ui, UIResponse::WalletError(error));
    };

    node.logger.log("program finished gracefully".to_string());
    exit_program(sender_to_ui, UIResponse::WalletFinished);
}

/// Handles UIResponse messages before exiting the program.
fn exit_program(sender_to_ui: GlibSender<UIResponse>, message: UIResponse) {
    match message {
        UIResponse::WalletError(error) => {
            if let WalletError::ErrorSendingToUI = error {
                eprintln!("Error sending_to_ui: {:?}", error);
            }
        }
        _ => {
            if let Err(error) = sender_to_ui.send(message) {
                eprintln!("Error sending_to_ui: {:?}", error);
            }
        }
    }
}
