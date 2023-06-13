use gtk::prelude::*;
use gtk::{glib,Application, ApplicationWindow, Builder, Box};

use crate::user_options::UiUserOptions;
use crate::wallet_sections::UiWalletSections;


const PROGRESS_COLUMN: u32 = 0;
const DATE_COLUMN: u32 = 1;
const TYPE_COLUMN: u32 = 2;
const LABEL_COLUMN: u32 = 3;
const AMOUNT_COLUMN: u32 = 4;



pub struct Ui {
    pub window: ApplicationWindow,

}

pub struct UiElements {
    pub window_box: Box,
    pub user_options: UiUserOptions,
    pub wallet_sections: UiWalletSections,
}

pub enum UiError {
    FailedToBuildUi,
    FailedToFindObject,
}
    

impl Ui{
    pub fn create_app() {
        // Initialise gtk components
    if gtk::init().is_err() {
        println!("Unable to load GTK.");
        return;
    }
    let glade_src = include_str!("ui.glade");

    // Load glade file
    let builder = Builder::from_string(&glade_src);

    let tx_tree_store: gtk::TreeStore = match builder.object("TxTreeStore"){
        Some(tree_store) => tree_store,
        None => return,
    };

    add_row(tx_tree_store, String::from("A"), String::from("A"), String::from("A"), String::from("A"), String::from("A"));
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
}

impl UiElements{
    fn new(builder: &gtk::Builder) -> Result<Self,UiError> {
        
        let window_box: Box = match builder.object("Container"){
            Some(window_box) => window_box,
            None => return Err(UiError::FailedToFindObject),
        };
        let user_options = UiUserOptions::new(builder)?;
        let wallet_sections = UiWalletSections::new(builder)?;
        
        Ok(Self {
            window_box,
            user_options,
            wallet_sections,
        })
    }

    fn add_tx(&self, progress: String, date: String, tx_type: String, label: String, amount: String){
        self.wallet_sections.transactions_tab.add_row(progress, date, tx_type, label,amount);
    }
}
// fn cambiarle_el_color_al_separator(builde: &Builder){
//     let separator: gtk::Separator = match builde.object("labelSeparator"){
//         Some(separator) => separator,
//         None => return,
//     };
//     let style_context = separator.style_context();
//     style_context.add_class("my-separator");
//     let css_provider = gtk::CssProvider::new();
//     css_provider.load_from_data(b".my-separator { background-color: gold; }");

//     gtk::StyleContext::add_provider_for_screen(
//         &separator.screen().expect("Error retrieving screen"),
//         &css_provider,
//         gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
//     );

// }

pub fn add_row(tx_tree_store: gtk::TreeStore, progress: String, date: String, tx_type: String, label: String, amount: String){
    let tree_iter = tx_tree_store.append(None);
    tx_tree_store.set_value(&tree_iter, PROGRESS_COLUMN, &glib::Value::from(&progress));
    tx_tree_store.set_value(&tree_iter, DATE_COLUMN, &glib::Value::from(&date));
    tx_tree_store.set_value(&tree_iter, TYPE_COLUMN, &glib::Value::from(tx_type));
    tx_tree_store.set_value(&tree_iter, LABEL_COLUMN, &glib::Value::from(label));
    tx_tree_store.set_value(&tree_iter, AMOUNT_COLUMN, &glib::Value::from(&amount));
}