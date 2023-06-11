use gtk::prelude::*;
use gtk::{glib,Application, ApplicationWindow, Button, Label, Frame, Box, Fixed, Separator, Paned, Notebook, Align,Orientation, Builder};



pub struct Ui {
    pub app: Application,
    pub window: ApplicationWindow,
    pub label: Label,
    pub button: Button,
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