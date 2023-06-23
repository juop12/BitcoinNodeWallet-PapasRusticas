mod test {
    use node::node::*;
    use node::utils::btc_errors::NodeError;
    use node::utils::config::*;


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
            headers_path: String::from("data/headers.bin"),
            blocks_path: String::from("data/blocks.bin"),

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
}
