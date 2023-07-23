/// Represents the different states of the node to perform different actions
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum NodeStatus {
    /// The node is currently running
    Initializing,
    /// The node is currently stopped
    Running,
    /// The node is currently paused
    Terminated,
}