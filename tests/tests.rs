mod test {
    use proyecto::node::*;

    /* #[test]
    fn integration_test_1_after_creating_a_node_it_connects_with_other_nodes() {
        let node = Node::new();
        assert!(node.get_tcp_streams().len() > 1);
    } */

    #[test]
    fn integration_test_2_initial_block_download_does_not_fail()-> Result<(),NodeError> {
        let node = Node::new();
        let mut j = 0;
        let aux = node.get_tcp_streams();
        match node.initial_block_download(&aux[0]){
            Ok(_) => {
                println!("Fue {j}");
                return Ok(());
            },
            Err(_) => {},
        }
        panic!("Ningun nodo funciono")
    }
}
