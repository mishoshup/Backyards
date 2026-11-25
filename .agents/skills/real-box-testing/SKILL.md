---
name: real-box-testing
description: Procedure for tests that start a real garden through BoxLite, including cleanup and measurement. Use when adding or changing adapter behavior, default garden settings or resource limits.
---

# Real garden testing

## Procedure

1. Confirm the host is macOS on Apple silicon. Otherwise skip the test and record the reason.
2. Use a temporary state directory with mode 700. Never use the real BoxLite home.
3. Create the garden from the default `GardenSpecification`, run a command, assert its output and exit code, then remove the garden.
4. Assert the restrictive defaults on the running garden:
   - No network device is present.
   - An outbound request fails.
   - A published port listens on `127.0.0.1` only.
   - Memory and CPU limits hold under load.
5. Record start time and memory use. Compare them with the baseline (start time about 2.4 s, memory about 0.3 GB per garden). Investigate every regression.
6. Run cleanup unconditionally, including after a failure. Afterwards `exists` returns false, no shim process remains, and the temporary directory is removed.

## Failure criteria

A failed cleanup is a test failure.
