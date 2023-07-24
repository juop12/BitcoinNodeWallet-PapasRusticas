use crate::utils::node_status::NodeStatus;
use gtk::prelude::*;
use gtk::{Application, Box, Builder, Label, ProgressBar, Window};
use node::utils::ui_communication_protocol::LoadingScreenInfo;
use std::sync::{Arc, Mutex};

/// Shows the block download progress bar and labels associated to it
/// when the block download is started
fn show_block_download_progress(builder: &Builder, total_blocks: usize) {
    let block_download_box: Box = builder
        .object("Block Downloader Box")
        .expect("Couldn't find Block Downloader Box");
    let total_amount_label: Label = builder
        .object("Total Block Number")
        .expect("Couldn't find Total Block Number Label");
    total_amount_label.set_text(format!("{total_blocks}").as_str());
    block_download_box.show();
}

/// Updates the block download progress bar and labels associated to it
/// with the amount of blocks downloaded
fn update_block_download_progress(builder: &Builder, blocks: usize) {
    let downloaded_amount_label: Label = builder
        .object("Current Block Number")
        .expect("Couldn't find Current Block Number Label");
    let total_amount_label: Label = builder
        .object("Total Block Number")
        .expect("Couldn't find Total Block Number Label");
    let progress_bar: ProgressBar = builder
        .object("Block Download Progress Bar")
        .expect("Couldn't find Block Download Progress Bar");
    downloaded_amount_label.set_text(format!("{blocks}").as_str());
    let total_amount = total_amount_label.label().parse::<usize>().unwrap_or(0);
    if total_amount == 0 {
        progress_bar.set_fraction(1.0);
    } else {
        progress_bar.set_fraction(blocks as f64 / total_amount as f64);
    }
}

/// Hides the block download progress bar and labels associated to it
/// when the block download is finished
fn hide_block_download_progress(builder: &Builder) {
    let block_download_box: Box = builder
        .object("Block Downloader Box")
        .expect("Couldn't find Block Download Box");
    block_download_box.hide();
}

/// Shows the loading screen and connects the delete event to hide the window and
/// set the node status to terminated
pub fn show_loading_screen(
    builder: &Builder,
    app: &Application,
    node_status: Arc<Mutex<NodeStatus>>,
) {
    let loading_window: Window = builder
        .object("Loading Screen Window")
        .expect("Couldn't find Loading Screen Window");
    let block_download_box: Box = builder
        .object("Block Downloader Box")
        .expect("Couldn't find Block Downloader Box");
    loading_window.set_title("Loading Screen");
    loading_window.set_application(Some(app));
    let loading_window_clone = loading_window.clone();
    loading_window.connect_delete_event(move |_, _| {
        loading_window_clone.hide();
        if let Ok(mut current_status) = node_status.lock() {
            if *current_status == NodeStatus::Initializing {
                *current_status = NodeStatus::Terminated;
            }
        }
        Inhibit(false)
    });
    loading_window.show_all();
    block_download_box.hide();
}

/// Updates the label of the loading screen with the progress of the node
pub fn update_loading_screen_label(builder: &Builder, progress: String) {
    let log_label: Label = builder
        .object("Log Label")
        .expect("Couldn't find Log Label");
    log_label.set_text(&progress);
}

/// Calls the corresponding handle function according to the message received
pub fn handle_loading_screen_update(builder: &Builder, message: LoadingScreenInfo) {
    match message {
        LoadingScreenInfo::UpdateLabel(progress) => update_loading_screen_label(builder, progress),
        LoadingScreenInfo::DownloadedBlocks(blocks) => {
            update_block_download_progress(builder, blocks)
        }
        LoadingScreenInfo::StartedBlockDownload(total_blocks) => {
            show_block_download_progress(builder, total_blocks)
        }
        LoadingScreenInfo::FinishedBlockDownload => hide_block_download_progress(builder),
    }
}
