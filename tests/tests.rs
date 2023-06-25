mod test {
    use std::sync::mpsc;
    use std::thread;

    use node::blocks::Outpoint;
    use node::node::*;
    use node::run::{run, initialize_node};
    use node::utils::btc_errors::NodeError;
    use node::utils::config::*;
    use node::utils::ui_communication_protocol::{UIToWalletCommunication as UIRequest, WalletToUICommunication as UIResponse};
    use glib::{Sender as GlibSender, Receiver as GlibReceiver};
    use node::wallet::{Wallet, get_bytes_from_hex};


    const BEGIN_TIME_EPOCH: u32 = 1681084800; // 2023-04-10


    #[test]
    fn integration_test_1_after_creating_a_node_it_connects_with_other_nodes(
    ) -> Result<(), NodeError> {
        let config = Config {
            version: 70015,
            dns_port: 18333,
            local_host: [127, 0, 0, 1],
            local_port: 1001,
            log_path: String::from("tests_txt/integration_test_1_log.txt"),
            begin_time: BEGIN_TIME_EPOCH,
            headers_path: String::from("tests_txt/headers.bin"),
            blocks_path: String::from("tests_txt/blocks.bin"),
            ipv6_enabled: false,
        };

        let node = Node::new(config)?;
        assert!(node.tcp_streams.len() > 1);
        Ok(())
    }

    #[test]
    fn integration_test_2_initial_block_download_does_not_fail() -> Result<(), NodeError> {
        let config = Config {
            version: 70015,
            dns_port: 18333,
            local_host: [127, 0, 0, 1],
            local_port: 1001,
            log_path: String::from("tests_txt/integration_test_2_log.txt"),
            begin_time: BEGIN_TIME_EPOCH,
            headers_path: String::from("node/data/headers.bin"),
            blocks_path: String::from("node/data/blocks.bin"),

            ipv6_enabled: false,
        };

        let mut node = Node::new(config)?;

        match node.initial_block_download() {
            Ok(_) => {
                return Ok(());
            }
            Err(err) => Err(err),
        }
    }

    #[test]
    fn test3_set_wallet(){
        
        let mut node = initialize_node(vec!["test".to_string(), "node/src/nodo.conf".to_string()]).unwrap();
        let wallet = Wallet::from("cSDPYr9FfseHx8jbjrnz9ryERswMkv6vKSccomu1ShfrJXj2d65Z".to_string()).unwrap();
        let (glib_sender, _glib_receiver) = glib::MainContext::channel::<UIResponse>(glib::PRIORITY_DEFAULT);

        let wallet = wallet.handle_change_wallet(&mut node, "cPvHucStvVrMmvkPY7pixfnJC6m3hhRRjAWaRDjeghqBae8DG3BB".to_string()).unwrap();
        assert_eq!(wallet.balance, 70000);
        assert_eq!(wallet.utxos.len(), 1);
        //assert_eq!( Vec::from(wallet.utxos.keys().collect::<Vec<&Outpoint>>()[0].hash) , get_bytes_from_hex("4657cacadae490c74a393dd288b94849622e79c819129d89323bac92370b5578".to_string()));
    }
    /*
    #[test]
    fn test4_update(){
        
        let mut node = initialize_node(vec!["test".to_string(), "node/src/nodo.conf".to_string()]).unwrap();
        let wallet = Wallet::from("cSDPYr9FfseHx8jbjrnz9ryERswMkv6vKSccomu1ShfrJXj2d65Z".to_string()).unwrap();
        let (glib_sender, _glib_receiver) = glib::MainContext::channel::<UIResponse>(glib::PRIORITY_DEFAULT);

        let wallet = wallet.handle_change_wallet(&mut node, "cPvHucStvVrMmvkPY7pixfnJC6m3hhRRjAWaRDjeghqBae8DG3BB".to_string()).unwrap();
        if let UIResponse::WalletInfo(wallet_info) = wallet.send_wallet_info(&glib_sender){
            assert_eq!(wallet_info.available_balance, 70000);
            assert_eq!(wallet_info.sending_pending_balance, 0);
            assert_eq!(wallet_info.receiving_pending_balance, 0);
            assert_eq!(wallet_info.utxos[0].amount, 70000);
            assert!(wallet_info.pending_tx.is_empty());
            return 
        }
        panic!("Wrong response");
    }*/

    #[test]
    fn test5_block_info(){
        
        let mut node = initialize_node(vec!["test".to_string(), "node/src/nodo.conf".to_string()]).unwrap();
        let mut wallet = Wallet::from("cSDPYr9FfseHx8jbjrnz9ryERswMkv6vKSccomu1ShfrJXj2d65Z".to_string()).unwrap();

        if let UIResponse::BlockInfo(block_info) = wallet.handle_last_block_info(&mut node).unwrap(){
            let block_headers = node.get_block_headers().unwrap();
            let block_chain = node.get_blockchain().unwrap();
            let block = block_chain.get(&block_headers[block_headers.len()-1].hash()).unwrap();
            assert_eq!(block_info.block_header, block.get_header());
            assert_eq!(block_info.block_number, block_headers.len());
            assert_eq!(block_info.block_tx_hashes.len(), block.transactions.len());
            return 
        }
        panic!("Wrong response");
    }
}