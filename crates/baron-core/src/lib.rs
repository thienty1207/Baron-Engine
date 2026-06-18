pub mod asset_lifecycle;
pub mod automation;
pub mod capability;
pub mod certification;
pub mod config;
pub mod context;
pub mod continuity;
pub mod control_plane;
pub mod firewall;
pub mod harness;
pub mod harness_improvement;
pub mod identity;
pub mod memory;
pub mod migration;
pub mod plan;
pub mod proof;
pub mod release;
pub mod risk;
pub mod session;
pub mod session_replay;
pub mod survey;
pub mod trace;
pub mod vault;

pub fn product_name() -> &'static str {
    "Baron Engine"
}

pub fn phase() -> &'static str {
    "phase-18-20-baron-3-foundation"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_current_phase_identity() {
        assert_eq!(product_name(), "Baron Engine");
        assert_eq!(phase(), "phase-18-20-baron-3-foundation");
    }
}
