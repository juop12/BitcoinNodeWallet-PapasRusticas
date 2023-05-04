mod test {
    use proyecto::node::*;

    #[test]
    fn integration_test_1_after_creating_a_node_it_connects_with_other_nodes() {
        let node = Node::new();
        assert!(node.get_tcp_streams().len() > 1);
    }
}
