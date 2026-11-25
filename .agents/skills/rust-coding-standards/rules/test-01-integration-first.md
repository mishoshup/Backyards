---
ruleId: TEST-01
enforcedBy: review; CI
source: Project test policy (user requirement)
---

# Integration and regression tests carry the proof

Unit tests are cheap guards. Proof comes from tests that run the real command on a temporary directory, and from adapter tests that start a real garden on macOS with Apple silicon and remove it afterwards.

- Assert values, persisted state, the audit log and the report text.
- A fixed bug gets a regression test that fails before the fix.
- Measure start time and memory for garden tests and record a baseline. A regression is a failure to investigate.
- Every test cleans up the gardens, directories and processes it created, including on failure.
