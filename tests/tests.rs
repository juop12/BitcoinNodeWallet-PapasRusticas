mod test {
    use node::node::*;
    use node::run::initialize_node;
    use node::utils::btc_errors::NodeError;
    use node::utils::config::*;
    use node::utils::ui_communication_protocol::UIResponse;
    use node::wallet::Wallet;

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
        let (sx, _rx) = glib::MainContext::channel::<UIResponse>(glib::PRIORITY_DEFAULT);
        let node = Node::new(config, sx)?;
        assert!(node.initial_peers.len() > 1);
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
            headers_path: String::from("data/headers.bin"),
            blocks_path: String::from("data/blocks.bin"),

            ipv6_enabled: false,
        };
        let (sx, _rx) = glib::MainContext::channel::<UIResponse>(glib::PRIORITY_DEFAULT);
        let mut node = Node::new(config, sx)?;

        match node.initial_block_download() {
            Ok(_) => {
                return Ok(());
            }
            Err(err) => Err(err),
        }
    }

    #[test]
    fn test3_set_wallet() {
        let (sx, _rx) = glib::MainContext::channel::<UIResponse>(glib::PRIORITY_DEFAULT);
        let mut node =
            initialize_node(vec!["test".to_string(), "node/src/nodo.conf".to_string()], sx).unwrap();

        let wallet =
            Wallet::from("cTcbayZmdiCxNywGxfLXGLqS2Y8uTNzGktbFXZnkNCR3zeN1XMQC".to_string())
                .unwrap();

        let wallet = wallet
            .handle_change_wallet(
                &mut node,
                "cW4xB3oopcqxK5hACPKpTtsDZHkcnKn4VFih5bH4vZKAkeDaVEPy".to_string(),
            )
            .unwrap();
        assert_eq!(wallet.balance, 70000);
        assert_eq!(wallet.utxos.len(), 1);
    }

    #[test]
    fn test4_block_info() {
        let (sx, _rx) = glib::MainContext::channel::<UIResponse>(glib::PRIORITY_DEFAULT);
        let mut node =
            initialize_node(vec!["test".to_string(), "node/src/nodo.conf".to_string()], sx).unwrap();

        let mut wallet =
            Wallet::from("cW4xB3oopcqxK5hACPKpTtsDZHkcnKn4VFih5bH4vZKAkeDaVEPy".to_string())
                .unwrap();

        if let UIResponse::BlockInfo(block_info) = wallet.handle_last_block_info(&mut node).unwrap()
        {
            let block_headers = node.get_block_headers().unwrap();
            let block_chain = node.get_blockchain().unwrap();
            let block = block_chain
                .get(&block_headers[block_headers.len() - 1].hash())
                .unwrap();
            assert_eq!(block_info.block_header, block.get_header());
            assert_eq!(block_info.block_number, block_headers.len());
            assert_eq!(block_info.block_tx_hashes.len(), block.transactions.len());
            return;
        }
        panic!("Wrong response");
    }

    #[test]
    fn test5_tx_valida() -> Result<(), NodeError> {
        let (sx, _rx) = glib::MainContext::channel::<UIResponse>(glib::PRIORITY_DEFAULT);
        let mut node =
            initialize_node(vec!["test".to_string(), "node/src/nodo.conf".to_string()], sx).unwrap();

        let wallet =
            Wallet::from("cW4xB3oopcqxK5hACPKpTtsDZHkcnKn4VFih5bH4vZKAkeDaVEPy".to_string())
                .unwrap();
        let block_hash = node.get_block_headers()?[2439100 - 1].hash();
        let tx_hash = node
            .get_blockchain()?
            .get(&block_hash)
            .unwrap()
            .get_tx_hashes()[0];
        println!("{:?}", tx_hash);
        if let UIResponse::ResultOFTXProof(result) = wallet
            .handle_obtain_tx_proof(&mut node, tx_hash, 2439100)
            .unwrap()
        {
            assert!(result.is_some());
            return Ok(());
        }

        panic!("Wrong response");
    }
}
