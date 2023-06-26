use gtk::prelude::*;
use gtk::{Builder, Application, Dialog, Label, Button};


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

pub fn handle_initialization_error(builder: &Builder, app: &Application){
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