---
ruleId: MCP-04
enforcedBy: unit tests of both paths
source: rmcp documentation (ErrorData vs CallToolResult::error); MCP specification (tool errors)
---

# Return tool failures as results and bad calls as protocol errors

A malformed call returns `ErrorData::invalid_params`. A tool that ran and failed returns `CallToolResult::error` with text the agent can act on. (rmcp documentation checked.)

```rust
// Incorrect
Err(ErrorData::internal_error("no files", None))

// Correct
Ok(CallToolResult::error(vec![Content::text("no entries matched the retention rule")]))
```
