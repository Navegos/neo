pub mod caveman;
pub mod codegraph;
pub mod contextmode;
pub mod headroom;
pub mod helpers;
pub mod ponytail;
pub mod principles;
pub mod projectmem;
pub mod rtk;

use crate::core::register_tool;

pub use caveman::*;
pub use codegraph::*;
pub use contextmode::*;
pub use headroom::*;
pub use helpers::*;
pub use ponytail::*;
pub use principles::*;
pub use projectmem::*;
pub use rtk::*;

pub fn register() {
    register_tool(build_rtk());
    register_tool(build_caveman());
    register_tool(build_ponytail());
    register_tool(build_principles());
    register_tool(build_codegraph());
    register_tool(build_context_mode());
    register_tool(build_headroom());
    register_tool(build_projectmem());
}

