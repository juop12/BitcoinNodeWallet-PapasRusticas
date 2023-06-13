use gtk::prelude::*;

use crate::UiError;

pub struct WalletTransactions{
    pub overview_fixed: gtk::Fixed,
}

impl WalletTransactions{
    pub fn new(builder: &gtk::Builder) -> Result<WalletTransactions, UiError> {
        let overview_fixed: gtk::Fixed = match builder.object("overviewFixed") {
            Some(overview_fixed) => overview_fixed,
            None => return Err(UiError::FailedToFindObject),
        };
        Ok(Self {
            overview_fixed,
        })
        
    }
}