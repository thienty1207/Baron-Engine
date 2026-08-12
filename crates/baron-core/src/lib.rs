pub mod architecture;
pub mod asset_lifecycle;
pub mod authority;
pub mod automation;
pub mod autopilot;
pub mod capability;
pub mod certification;
pub mod code_graph;
pub mod config;
pub mod context;
pub mod continuity;
pub mod control_plane;
pub mod domain_language;
pub mod execution_receipt;
pub mod firewall;
pub mod graphify;
pub mod harness;
pub mod harness_experiment;
pub mod harness_improvement;
pub mod identity;
pub mod intent;
pub mod memory;
pub mod migration;
pub mod operations;
pub mod plan;
pub mod platform;
pub mod proof;
pub mod release;
pub mod review_gate;
pub mod risk;
pub mod session;
pub mod session_replay;
pub mod state_guard;
pub mod survey;
pub mod trace;
pub mod vault;
pub mod work_shape;

pub fn product_name() -> &'static str {
    "Baron Engine"
}

pub fn phase() -> &'static str {
    "baron-3.3-release"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_current_phase_identity() {
        assert_eq!(product_name(), "Baron Engine");
        assert_eq!(phase(), "baron-3.3-release");
    }
}
