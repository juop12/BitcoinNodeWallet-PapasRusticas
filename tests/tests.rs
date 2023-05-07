mod test {
    use proyecto::node::*;
    use proyecto::config::*;

    #[test]
    fn integration_test_1_after_creating_a_node_it_connects_with_other_nodes() {
        let config = Config {
            version: 70015,
            dns_port: 18333,
            local_host: [127,0,0,1],
            local_port: 1001,
            log_path: String::from("src/node_log.txt"),
        };

        let node = Node::new(config);
        assert!(node.get_tcp_streams().len() > 1);
    }
}
