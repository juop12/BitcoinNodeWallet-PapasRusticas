use gtk::prelude::*;

use crate::UiError;

pub struct WalletSend{
    pub send_fixed: gtk::Fixed,
}

impl WalletSend{
    pub fn new(builder: &gtk::Builder) -> Result<Self,UiError> {
        let send_fixed: gtk::Fixed = match builder.object("Send") {
            Some(send_fixed) => send_fixed,
            None => return Err(UiError::FailedToFindObject)
        };
        Ok(Self {
            send_fixed,
        })
    }
}