mod test {
    use proyecto::utils::config::*;
    use proyecto::node::*;
    use proyecto::utils::log::*;
    use proyecto::utils::btc_errors::NodeError;


    const BEGIN_TIME_EPOCH: u32 = 1681084800; // 2023-04-10


     #[test]
    fn integration_test_1_after_creating_a_node_it_connects_with_other_nodes() -> Result<(), NodeError>  {
        let config = Config {
            version: 70015,
            dns_port: 18333,
            local_host: [127,0,0,1],
            local_port: 1001,
            log_path: String::from("src/test_log.txt"),
            begin_time: BEGIN_TIME_EPOCH,
        };

        let node = Node::new(config)?;
        assert!(node.get_tcp_streams().len() > 1);
        Ok(())
    } 

    #[test]
    fn integration_test_2_initial_block_download_does_not_fail()-> Result<(),NodeError> {
         let config = Config {
            version: 70015,
            dns_port: 18333,
            local_host: [127,0,0,1],
            local_port: 1001,
            log_path: String::from("src/node_log.txt"),
            begin_time: BEGIN_TIME_EPOCH,
        };
      
        let mut node = Node::new(config)?;
      
        match node.initial_block_download(){
            Ok(_) => {
                return Ok(());
            },
            Err(err) => Err(err),
        }
    }
}
