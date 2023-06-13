use gtk::prelude::*;
use gtk::{TreeView, Builder, Box,TreeViewColumn, TreeStore,Fixed,TreeIter,glib};


use crate::UiError;

const PROGRESS_COLUMN: u32 = 0;
const DATE_COLUMN: u32 = 1;
const TYPE_COLUMN: u32 = 2;
const LABEL_COLUMN: u32 = 3;
const AMOUNT_COLUMN: u32 = 4;


pub struct WalletTransactions{
    transactions_fixed: Fixed,
    tx_tree: TreeView,
    tx_tree_store: TreeStore,
    tx_tree_columns: Vec<TreeViewColumn>
}

impl WalletTransactions{
    pub fn new(builder: &Builder) -> Result<WalletTransactions, UiError> {
        let transactions_fixed: gtk::Fixed = match builder.object("Transactions") {
            Some(transactions_fixed) => transactions_fixed,
            None => return Err(UiError::FailedToFindObject),
        };
        let tx_tree: gtk::TreeView = match builder.object("TxTree") {
            Some(tx_tree) => tx_tree,
            None => return Err(UiError::FailedToFindObject),
        };
        let tx_tree_store: gtk::TreeStore = match builder.object("TxTreeStore") {
            Some(tx_tree_store) => tx_tree_store,
            None => return Err(UiError::FailedToFindObject),
        };
        let tx_tree_columns: Vec<TreeViewColumn> = build_columns(builder)?;
        Ok(Self {
            transactions_fixed,
            tx_tree,
            tx_tree_store,
            tx_tree_columns,
        })
        
    }
    pub fn add_row(&self, progress: String, date: String, tx_type: String, label: String, amount: String){
        let tree_iter = self.tx_tree_store.append(None);
        self.tx_tree_store.set_value(&tree_iter, PROGRESS_COLUMN, &glib::Value::from(&progress));
        self.tx_tree_store.set_value(&tree_iter, DATE_COLUMN, &glib::Value::from(&date));
        self.tx_tree_store.set_value(&tree_iter, TYPE_COLUMN, &glib::Value::from(tx_type));
        self.tx_tree_store.set_value(&tree_iter, LABEL_COLUMN, &glib::Value::from(label));
        self.tx_tree_store.set_value(&tree_iter, AMOUNT_COLUMN, &glib::Value::from(&amount));
    }


    

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
