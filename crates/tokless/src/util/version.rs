pub fn tokless_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub use crate::util::versions::semver_compare;

pub fn semver_gte(a: &str, b: &str) -> bool {
    semver_compare(a, b) >= 0
}

