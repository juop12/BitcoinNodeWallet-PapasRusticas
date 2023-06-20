use gtk::prelude::*;
use gtk::{TreeView, Builder, Box,TreeViewColumn, TreeStore,Fixed,TreeIter,glib};


use crate::UiError;

const PROGRESS_COLUMN: u32 = 0;
const DATE_COLUMN: u32 = 1;
const TYPE_COLUMN: u32 = 2;
const BLOCK_HEADER_COLUMN: u32 = 3;
const AMOUNT_COLUMN: u32 = 4;


pub fn add_row(tx_tree_store: TreeStore, date: String, block_header: String, amount: String){
    let tree_iter = tx_tree_store.append(None);
    tx_tree_store.set_value(&tree_iter, PROGRESS_COLUMN, &glib::Value::from(100));
    tx_tree_store.set_value(&tree_iter, DATE_COLUMN, &glib::Value::from(&date));
    tx_tree_store.set_value(&tree_iter, TYPE_COLUMN, &glib::Value::from("Mined"));
    tx_tree_store.set_value(&tree_iter, BLOCK_HEADER_COLUMN, &glib::Value::from(block_header));
    tx_tree_store.set_value(&tree_iter, AMOUNT_COLUMN, &glib::Value::from(&amount));
}

fn build_columns(builder: &Builder)-> Result<Vec<TreeViewColumn>,UiError>{
    let mut columns: Vec<TreeViewColumn> = Vec::new();
    let progress_column: TreeViewColumn = build_column(builder, String::from("Progress"))?;
    let date_column: TreeViewColumn = build_column(builder, String::from("Date"))?;
    let type_column : TreeViewColumn = build_column(builder, String::from("Type"))?; 
    let label_column: TreeViewColumn = build_column(builder, String::from("Label"))?;
    let amount_column: TreeViewColumn = build_column(builder, String::from("Amount"))?;
    columns.push(progress_column);
    columns.push(date_column);
    columns.push(type_column);
    columns.push(label_column);
    columns.push(amount_column);
    Ok(columns)
}



fn build_column(builder :&Builder, name: String)-> Result<TreeViewColumn,UiError>{
    let column:TreeViewColumn =match builder.object(name.as_str()){
        Some(column) => column,
        None => return Err(UiError::FailedToFindObject),
    };
    Ok(column)
}
