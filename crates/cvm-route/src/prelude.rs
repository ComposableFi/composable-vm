pub use alloc::{string::String, vec, vec::Vec};

pub use serde::{Deserialize, Serialize};

#[cfg(feature = "parity-scale-codec")]
pub use parity_scale_codec::{Decode, Encode};

#[cfg(feature = "parity-scale-codec")]
pub use scale_info::TypeInfo;
