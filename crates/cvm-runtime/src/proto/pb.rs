pub mod solidity {
    include!(concat!(env!("OUT_DIR"), "/solidity.rs"));
}
pub mod program {
    include!(concat!(env!("OUT_DIR"), "/cvm.program.rs"));
}

pub mod common {
    pub use cvm::proto::*;
}
