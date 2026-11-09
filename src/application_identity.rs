pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const GENERATION: &str = "OPENRULES";

pub fn release_name() -> String {
    format!("cirnet-rules-{VERSION}")
}
