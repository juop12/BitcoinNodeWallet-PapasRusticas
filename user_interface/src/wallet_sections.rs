use gtk::prelude::*;
use gtk::{Box};
use crate::wallet_overview::WalletOverview;
use crate::wallet_send::WalletSend;
use crate::wallet_transactions::WalletTransactions;

pub struct UiWalletSections {
    pub wallet_header: Box,
    pub overview_tab: WalletOverview,
    pub send_tab: WalletSend,
    pub transactions_tab: WalletTransactions,
}

