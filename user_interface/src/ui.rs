use gtk::prelude::*;
use gtk::{glib,Application, ApplicationWindow, Builder, Box};

use crate::user_options::UiUserOptions;
use crate::wallet_sections::UiWalletSections;

pub struct Ui {
    pub app: Application,
    pub window: ApplicationWindow,
}

pub struct UiWindow {
    pub window: ApplicationWindow,
    pub window_box: Box,
    pub user_options: UiUserOptions,
    pub wallet_sections: UiWalletSections,
}

impl Ui{
    pub fn crear_app() {
        // Initialise gtk components
    if gtk::init().is_err() {
        println!("Unable to load GTK.");
        return;
    }
    let glade_src = include_str!("ui.glade");

    // Load glade file
    let builder = Builder::from_string(&glade_src);

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