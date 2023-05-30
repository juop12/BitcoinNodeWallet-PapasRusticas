mod test {
    use proyecto::node::*;
    use proyecto::utils::btc_errors::NodeError;
    use proyecto::utils::config::*;


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
        };

        let node = Node::new(config)?;
        assert!(node.get_tcp_streams().len() > 1);
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
            headers_path: String::from("tests_txt/integration_test_2_headers.bin"),
            blocks_path: String::from("tests_txt/integration_test_2_blocks.bin"),
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
    fn integration_test_3_can_get_utxo_set() -> Result<(), NodeError> {
        let config = Config {
            version: 70015,
            dns_port: 18333,
            local_host: [127, 0, 0, 1],
            local_port: 1001,
            log_path: String::from("tests_txt/integration_test_3_log.txt"),
            begin_time: BEGIN_TIME_EPOCH,
            headers_path: String::from("tests_txt/integration_test_3_headers.bin"),
            blocks_path: String::from("tests_txt/integration_test_3_blocks.bin"),
        };

        let mut node = Node::new(config)?;
        
        node.initial_block_download()?;

        let utxo_set = node.create_utxo_set();

        assert!(utxo_set.len() > 0);
        Ok(())
    }
}
