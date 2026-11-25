---
ruleId: MCP-01
enforcedBy: cargo xtask architecture (mcp crate depends on feature crates only); review
source: rmcp documentation (tool_router, tool)
---

# Tools call the feature crates and hold no logic

An MCP tool maps its parameters to a feature call and maps the result back. The CLI and the MCP server never differ in behavior.

```rust
// Incorrect
#[tool] fn sweep(&self) { remove_old_files(); }

// Correct
#[tool] async fn sweep(&self, Parameters(arguments): Parameters<SweepArguments>) -> Result<Json<SweepReport>, ErrorData> { self.features.sweep(arguments.into()).await.map(Json).map_err(to_error_data) }
```
