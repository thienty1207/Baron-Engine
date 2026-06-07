#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentAdapter {
    Codex,
    Claude,
    Generic,
}

impl AgentAdapter {
    pub fn flag(self) -> &'static str {
        match self {
            AgentAdapter::Codex => "--codex",
            AgentAdapter::Claude => "--claude",
            AgentAdapter::Generic => "--agent",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_initial_adapter_flags() {
        assert_eq!(AgentAdapter::Codex.flag(), "--codex");
        assert_eq!(AgentAdapter::Claude.flag(), "--claude");
        assert_eq!(AgentAdapter::Generic.flag(), "--agent");
    }
}
