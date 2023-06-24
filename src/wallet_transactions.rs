use gtk::prelude::*;
use gtk::{TreeView, Builder, Box,TreeViewColumn, TreeStore,Fixed,TreeIter,glib};


use crate::UiError;

pub fn add_row(tx_tree_store: &TreeStore, tx_hash: &str, index: i32){
    let tree_iter = tx_tree_store.append(None);
    tx_tree_store.set_value(&tree_iter, 1, &glib::Value::from(index.to_string()));
    tx_tree_store.set_value(&tree_iter, 0, &glib::Value::from(tx_hash));
}
