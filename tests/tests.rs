mod test {
    //use proyecto::config;
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

    #[test]
    fn integration_test_2_initial_block_download_does_not_fail()-> Result<(),NodeError> {
         let config = Config {
            version: 70015,
            dns_port: 18333,
            local_host: [127,0,0,1],
            local_port: 1001,
            log_path: String::from("src/node_log.txt"),
        };
        let mut node = Node::new(config);
        //let mut j = 0;
        let aux = node.get_tcp_streams();
        match node.initial_block_download(){
            Ok(_) => {
                return Ok(());
            },
            Err(err) => Err(err),
        }
    }
}
