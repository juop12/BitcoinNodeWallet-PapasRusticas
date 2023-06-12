use gtk::prelude::*;

use crate::UiError;

pub struct WalletSend{
    pub overview_fixed: gtk::Fixed,
}

impl WalletSend{
    pub fn new(builder: &gtk::Builder) -> Result<Self,UiError> {
        let overview_fixed: gtk::Fixed = match builder.object("overviewFixed") {
            Some(overview_fixed) => overview_fixed,
            None => return Err(UiError::FailedToFindObject)
        };
        Ok(Self {
            overview_fixed,
        })
    }
}