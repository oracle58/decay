# Decay

[![Crates.io](https://img.shields.io/crates/v/decay)](https://crates.io/crates/decay)

---
TUI framework for Rust with zero dependencies.
Component tree, lifecycle hooks, reactive stores, double-buffered rendering, and direct platform I/O.

## Overview

| Crate | Purpose |
|--------|---------|
| `core` | Application context, component tree, nodes, stores, lifecycle scheduling (7 stages), command buffer, events, tree traversal, state machine |
| `ui` | Retained-mode UI: nodes, text, buttons, focus/keyboard nav, z-index, anchor layout, panels, separators, progress bars, spinners, animated text, scroll views, theming |
| `term` | Double-buffered framebuffer with cell-level diffing. Direct Win32/Unix syscalls. Bold, dim, italic, underline, strikethrough. Mouse input. 24-bit color. |
| `anim` | Tweens, keyframe tracks, easing functions |
| `time` | `Timer` node, `DeltaTime` store |
| `rand` | Seedable xorshift64: range, pick, shuffle, chance |
| `serde` | Minimal binary serialize/deserialize. No serde dependency. |
| `intro` | Animated decay tree intro sequence |

## Install

Full:
```toml
[dependencies]
decay = "0.2"
```

Features:
```toml
[dependencies]
decay = { version = "0.2", default-features = false, features = ["ui"] }
```

Available features: `full` (default), `core`, `term`, `ui`, `progress`, `anim`, `time`, `rand`, `serde`, `intro`

## Examples

**Simple** - minimal TUI app with styled text and keyboard handling:
```
cargo run --example simple
```

**Dashboard** - panels, progress bars, spinners, buttons, animated text:
```
cargo run --example dashboard
```

## Platform support

| Platform | Status            | Backend |
|----------|-------------------|---------|
| Windows  | Full              | Win32 Console API (kernel32 FFI) |
| Linux    | Full              | termios + ioctl |
| macOS    | Untested          | termios + ioctl |

Direct FFI to platform APIs. No libc wrapper crate needed.

## Benchmarks

Measured with [Criterion](https://github.com/bheisler/criterion.rs), release builds.

### Rendering

Cell-level diffing between front and back buffers (the rendering hot path):

| Scenario | 80x24 | 120x40 | 240x80 |
|----------|-------|--------|--------|
| Identical (no changes) | 5.5 us | 13.0 us | 51.7 us |
| Sparse (5% dirty) | 6.3 us | 15.5 us | 62.0 us |
| Fully dirty (100%) | 103 us | 259 us | 1.03 ms |

Typical frames change under 5% of cells. A 120x40 terminal diffs in ~15us, well within the 16ms frame budget.

### I/O overhead

How close is decay's frame assembly to the OS write ceiling?

| Stage | 120x40 | Notes |
|-------|--------|-------|
| OS write ceiling (NUL sink) | 220 ns | Raw kernel throughput, theoretical max |
| ANSI sequence assembly (precoded) | 11.9 us | Memcpy pre-built escape sequences |
| ANSI sequence assembly (fmt) | 122 us | Building escape sequences via `write!` |
| Decay framebuffer diff (sparse) | 15.5 us | Actual rendering path |

The bottleneck is escape sequence formatting, not I/O. Decay's diff engine skips unchanged cells, so real frames hit closer to the precoded path than the full fmt path. The OS write itself is negligible.

## License

Apache-2.0
