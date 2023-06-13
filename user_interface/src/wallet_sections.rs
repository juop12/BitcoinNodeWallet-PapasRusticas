use gtk::prelude::*;
use gtk::{Box};
use crate::wallet_overview::WalletOverview;
use crate::wallet_send::WalletSend;
use crate::wallet_transactions::WalletTransactions;
use crate::UiError;


pub struct UiWalletSections {
    pub wallet_header: Box,
    pub overview_tab: WalletOverview,
    pub send_tab: WalletSend,
    pub transactions_tab: WalletTransactions,
}

impl UiWalletSections {
    pub fn new(builder: &gtk::Builder) -> Result<Self,UiError> {
        let wallet_header: Box = match builder.object("walletHeader") {
            Some(wallet_header) => wallet_header,
            None => return Err(UiError::FailedToFindObject)
        };
        let overview_tab = WalletOverview::new(builder)?;
        let send_tab = WalletSend::new(builder)?;
        let transactions_tab = WalletTransactions::new(builder)?;
        Ok(Self {
            wallet_header,
            overview_tab,
            send_tab,
            transactions_tab,
        })
    }
}