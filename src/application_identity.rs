pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const GENERATION: &str = "CGACP v1";

pub fn release_name() -> String {
    format!("cirnet-rules-{VERSION}")
}
