//! Flow/Control operators (14 total)
//! - Control: Switch, Select, Gate, Loop, ForEach (5)
//! - State: Delay, Previous, Changed, Trigger, Once, Counter (6)
//! - Context: GetFloatVar, SetFloatVar, GetIntVar (3)

use crate::registry::OperatorRegistry;

mod control;
mod state;
mod context;

pub use control::*;
pub use state::*;
pub use context::*;

pub fn register_all(registry: &OperatorRegistry) {
    control::register(registry);
    state::register(registry);
    context::register(registry);
}
