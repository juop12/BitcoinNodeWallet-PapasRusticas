
use std::ops::Add;

use gtk::prelude::*;
use gtk::{glib,Application, ApplicationWindow, Button, Label, Frame, Box, Fixed, Separator, Paned, Notebook, Align,Orientation,ColumnView, ListStore};



const APP_ID: &str = "com.github.gtk-rs.HelloWorld";    
const FIXED_WIDTH: i32 = 800;
const FIXED_HEIGHT: i32 = 600;

const LAST_UPDATE_TIME: &str = "6 days";

pub struct Ui {
    pub app: Application,
    pub window: ApplicationWindow,
    pub label: Label,
    pub button: Button,
}

impl Ui{
    pub fn crear_app() -> glib::ExitCode{
       let app = Application::builder().application_id(APP_ID).build();

        app.connect_activate(build_ui);

        app.run()
    }
}

fn build_ui(app: &Application){

    let tabs_notebook = Notebook::new();

    let general_box = Box::new(Orientation::Vertical, 0);

    tabs_notebook.append_page(&build_overview_tab(), Some(&Label::new(Some("Overview"))));
    tabs_notebook.append_page(&build_send_tab(), Some(&Label::new(Some("Send"))));
    tabs_notebook.append_page(&build_receive_tab(), Some(&Label::new(Some("Receive"))));
    tabs_notebook.append_page(&build_transactions_tab(), Some(&Label::new(Some("Transactions"))));


    let last_update_label = Label::new(Some("Time since last update:"));
    last_update_label.set_halign(Align::Start);
    let date_label = Label::new(Some(LAST_UPDATE_TIME));

    let paned = Paned::builder().start_child(&last_update_label).end_child(&date_label).build();
    paned.set_size_request(1000, 20);

    general_box.append(&tabs_notebook);
    general_box.append(&paned);


    let window = ApplicationWindow::builder()
        .application(app)
        .title("Bitcoin Ui")
        .child(&general_box)
        .build();

    window.set_default_size(1000, 620);
    window.present()

}

fn build_overview_tab() -> Box {
    let tab_box = Box::new(Orientation::Vertical, 0);
    
    let fixed = Fixed::builder().build();
    fixed.set_size_request(FIXED_WIDTH, FIXED_HEIGHT);
    fixed.set_halign(Align::Center);
    fixed.set_valign(Align::Center);

    fixed.put(&build_balance_frame(), 50.0, 50.0);
    fixed.put(&build_recent_transactions_frame(), 400.0, 50.0);

    tab_box.append(&fixed);
    tab_box
}

fn build_balance_frame() -> Frame {
    let available_label = Label::new(Some("Available:"));
    let pending_label = Label::new(Some("Pending:"));
    let immature_label = Label::new(Some("Immature:"));
    let separator = Separator::new(Orientation::Horizontal);
    let total_label = Label::new(Some("Total:"));
    

    let frame_box = Box::new(Orientation::Vertical, 20);
    frame_box.set_halign(Align::Start);
    frame_box.append(&available_label);
    frame_box.append(&pending_label);
    frame_box.append(&immature_label);
    frame_box.append(&separator);
    frame_box.append(&total_label);
    

    let frame = Frame::builder().label("Balances").child(&frame_box).build();
    frame.set_size_request(200, 200);

    frame
}

fn build_recent_transactions_frame() -> Frame {
    let frame_box = Box::new(Orientation::Vertical, 20);
    frame_box.set_halign(Align::Start);

    let label = Label::new(Some("ToDo!"));

    frame_box.append(&label);

    let tr_frame = Frame::builder().label("Recent Transactions").child(&frame_box).build();

    tr_frame
}

fn build_send_tab() -> Box {
    let tab_box = Box::new(Orientation::Vertical, 0);
    let fixed = Fixed::builder().build();
    fixed.set_size_request(FIXED_WIDTH, FIXED_HEIGHT);
    fixed.set_halign(Align::Center);
    fixed.set_valign(Align::Center);

    let send_list_box = gtk::ListBox::new();
    send_list_box.set_size_request(FIXED_WIDTH, 500);
    send_list_box.set_halign(Align::Center);
    send_list_box.set_valign(Align::Start);

    let bitcoin_adress_label = Label::new(Some("Bitcoin Address:"));
    bitcoin_adress_label.set_halign(Align::Start);
    let adress_paned = Paned::builder().start_child(&bitcoin_adress_label).end_child(&gtk::Entry::new()).build();
    adress_paned.set_size_request(FIXED_WIDTH, 20);
    adress_paned.set_halign(Align::Start);
    send_list_box.append(&adress_paned);

    let adress_label = Label::new(Some("Label:"));
    adress_label.set_halign(Align::Start);
    let adress_label_paned = Paned::builder().start_child(&adress_label).end_child(&gtk::Entry::new()).build();
    adress_label_paned.set_size_request(FIXED_WIDTH, 20);
    send_list_box.append(&adress_label_paned);

    let ammount_label = Label::new(Some("Ammount:"));
    ammount_label.set_halign(Align::Start);
    let ammount_paned = Paned::builder().start_child(&ammount_label).end_child(&gtk::Entry::new()).build();
    ammount_paned.set_size_request(FIXED_WIDTH, 20);
    ammount_paned.set_halign(Align::Start);
    send_list_box.append(&ammount_paned);


    fixed.put(&send_list_box, 0.0, 0.0);

    tab_box.append(&fixed);
    tab_box
}

fn build_receive_tab() -> Box {
    let tab_box = Box::new(Orientation::Vertical, 0);

    let fixed = Fixed::builder().build();
    fixed.set_size_request(FIXED_WIDTH, FIXED_HEIGHT);
    fixed.set_halign(Align::Center);
    fixed.set_valign(Align::Center);

    tab_box.append(&fixed);
    tab_box
}



fn build_transactions_tab() -> Box {
    let tab_box = Box::new(Orientation::Vertical, 0);

    let fixed = Fixed::builder().build();
    fixed.set_size_request(FIXED_WIDTH, FIXED_HEIGHT);
    fixed.set_halign(Align::Center);
    fixed.set_valign(Align::Center);

    tab_box.append(&fixed);
    tab_box
}

// fn build_transactions_column() -> ColumnView {
    
//     let view = ColumnView::new();

//     view
// }




#[cfg(test)]
mod tests {
    use super::*;
        
    #[test]
    fn test_ui_1_crear_app() {
        Ui::crear_app();
    }
}