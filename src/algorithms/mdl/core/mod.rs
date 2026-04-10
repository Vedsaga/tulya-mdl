//! `mdl::core` – All MDL algorithm versions.

pub mod types;
pub mod v1;
pub mod v2;
pub mod diagnostics;

pub use types::{MatchTable, convert_mdl_to_mapping};
pub use v1::BlockFamily as MdlV1;
pub use v2::FastBlockFamily as MdlV2;
