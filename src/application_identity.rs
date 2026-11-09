#[allow(dead_code)]
mod generated {
    include!(concat!(env!("OUT_DIR"), "/identity.rs"));
}
pub use generated::*;

pub fn release_name() -> &'static str {
    RELEASE_NAME
}
