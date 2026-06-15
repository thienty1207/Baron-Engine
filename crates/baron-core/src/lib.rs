pub mod capability;
pub mod config;
pub mod context;
pub mod firewall;
pub mod harness;
pub mod memory;
pub mod migration;
pub mod plan;
pub mod proof;
pub mod release;
pub mod risk;
pub mod survey;
pub mod trace;
pub mod vault;

pub fn product_name() -> &'static str {
    "Baron Engine"
}

pub fn phase() -> &'static str {
    "phase-8-release-hardening"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_current_phase_identity() {
        assert_eq!(product_name(), "Baron Engine");
        assert_eq!(phase(), "phase-8-release-hardening");
    }
}
