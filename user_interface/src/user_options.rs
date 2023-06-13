use gtk::prelude::*;
use gtk::{MenuBar};

use crate::UiError;

pub struct UiUserOptions {
    pub menu_bar: MenuBar
}

impl UiUserOptions {
    pub fn new(builder: &gtk::Builder) -> Result<Self,UiError>{
        let menu_bar: MenuBar = match builder.object("menuBar") {
            Some(menu_bar) => menu_bar,
            None => panic!("Failed to find menuBar"),
        };
        Ok(Self {
            menu_bar,
        })
    }
}
