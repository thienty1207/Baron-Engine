pub mod survey;

pub fn product_name() -> &'static str {
    "Baron Engine"
}

pub fn phase() -> &'static str {
    "phase-0-foundation"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_phase_zero_identity() {
        assert_eq!(product_name(), "Baron Engine");
        assert_eq!(phase(), "phase-0-foundation");
    }
}
