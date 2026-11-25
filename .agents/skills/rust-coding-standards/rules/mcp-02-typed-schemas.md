---
id: MCP-02
enforcedBy: compiler (`schemars::JsonSchema` derive); schema snapshot test
source: rmcp documentation (Parameters, Json, schemars)
---

# Parameters and results derive JsonSchema

Use `Parameters<T>` with `schemars::JsonSchema` for input and `Json<T>` for structured output, so the tool list shows exact schemas to the agent. (rmcp documentation checked.)

```rust
// Incorrect
fn sweep(&self, text: String)

// Correct
fn sweep(&self, Parameters(arguments): Parameters<SweepArguments>) -> Json<SweepReport>
```
