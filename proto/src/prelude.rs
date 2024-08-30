// Re-export according to alloc::prelude::v1 because it is not yet stabilized
// https://doc.rust-lang.org/src/alloc/prelude/v1.rs.html
#[allow(unused_imports)]
pub use alloc::{
    borrow::ToOwned,
    boxed::Box,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};
pub use core::prelude::v1::*;

#[allow(unused_imports)]
pub use crate::time::{FromMillis, ToMillis};
