use crate::NodeID;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Input game graph is unbeatable. Change the input data so that the game can be completed.\nThe source scc nodes are: {0:?}")]
    GameUnbeatable(Vec<NodeID>),
}

pub type Result<T> = std::result::Result<T, Error>;
