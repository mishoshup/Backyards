---
name: boxlite-adapter
description: Procedure and verified facts for the BoxLite backend adapter. Use before writing or changing code that creates, runs, limits or removes a garden, or when a BoxLite API question arises.
---

# BoxLite adapter

## Purpose

BoxLite is the only garden backend. Its types are confined to the adapter crate `gardens-boxlite`. The `gardens` crate exposes only the `GardenBackend` trait and the `GardenSpecification` type.

## Verification procedure

The BoxLite documentation site lags the crate. At the time of writing, the Rust SDK reference lists only `NetworkSpec::Isolated`, while crate version 0.14.1 defines `NetworkSpec::Enabled { allow_net }` and `NetworkSpec::Disabled`.

1. Read the source of the pinned crate version:

```bash
D=$(echo ~/.cargo/registry/src/*/boxlite-<version>)
grep -rn "pub enum NetworkSpec\|pub struct BoxOptions" "$D/src"
```

2. Use Context7 (`/websites/boxlite_ai`) for orientation only.
3. State which source was checked.

## Verified facts (boxlite 0.14.1)

| Subject | Fact |
| --- | --- |
| Dependency | `boxlite` from crates.io. No git or Python route. |
| Presets | None exist. |
| Security | `SecurityOptions::enabled()` equals the default: jailer, seccomp where supported, unprivileged uid and gid, closed file descriptors, sanitized environment, resource limits. `SecurityOptions::disabled()` turns all of it off. The field is `BoxOptions.advanced.security`. |
| Construction | `AdvancedBoxOptions` has private fields. Construct it from `default()` and assign `security`. |
| CPU | `cpus` is a floating point value. Pass `1.0` for one CPU. |
| Removal | `auto_remove` defaults to true. `stop()` also removes the box, and a later `remove` returns NotFound. |
| Network | `NetworkSpec::Disabled` starts no network device. `NetworkSpec::Enabled { allow_net }` filters by host or IPv4 rule and does not filter DNS. |
| Ports | `PortSpec.host_ip` defaults to all interfaces. The adapter must set `127.0.0.1`. |
| Options | `BoxOptions` carries `cpus`, `memory_mib`, `disk_size_gb`, `volumes`, `ports` and `network`. |
| Runtime | `BoxliteRuntime::remove(id, force)` deletes a box. `exists(id)` confirms deletion. |

## Requirements

1. The adapter maps `GardenSpecification` to `BoxOptions` in a single function. Defaults come from rule SAFETY-02.
2. Every created garden is held by a guard that removes it on drop and on error.
3. Baseline on the reference Mac: start time about 2.4 s, memory about 0.3 GB per garden. A regression against this baseline is a defect.
4. Real-garden tests run only on macOS with Apple silicon. Elsewhere they are skipped, and the skip reason is recorded.
5. No garden, shim process or state directory remains after a test. Verify with `exists` and the process list.
