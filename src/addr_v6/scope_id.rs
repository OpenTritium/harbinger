use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Copy, Clone)]
pub struct ScopeId(u32);

impl From<u32> for ScopeId {
    fn from(val: u32) -> Self {
        Self(val)
    }
}

impl From<ScopeId> for u32 {
    fn from(val: ScopeId) -> Self {
        val.0
    }
}
