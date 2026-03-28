# Decay

[![Crates.io](https://img.shields.io/crates/v/decay)](https://crates.io/crates/decay)

Zero-dependency TUI framework. Talks directly to Win32 and termios. No crossterm, ncurses or libc wrappers.

## Platforms

| Platform | Status   | Backend |
|----------|----------|---------|
| Windows  | Full     | Win32 Console API (kernel32 FFI) |
| Linux    | Full     | termios + ioctl |
| macOS    | Untested | termios + ioctl |

## Install

```toml
[dependencies]
decay = "0.2"
```

Or pick what you need:
```toml
[dependencies]
decay = { version = "0.2", default-features = false, features = ["ui"] }
```

Features: `full` (default), `core`, `term`, `ui`, `progress`, `anim`, `time`, `rand`, `serde`, `intro`

## Examples

Minimal app — styled text and keyboard input:
```
cargo run --example simple
```

Kitchen sink — panels, progress bars, spinners, buttons, animated text:
```
cargo run --example dashboard
```

## Modules

| Module | Function |
|--------|--------------|
| `core` | Node tree, typed stores, 7-stage lifecycle loop, deferred command buffer, input and event plumbing |
| `term` | Double-buffered framebuffer, cell-level diffing, 24-bit color, text attributes, mouse input, direct platform syscalls |
| `ui` | Text, buttons, panels, separators, progress bars, spinners, text input, anchor layout, focus navigation, z-ordering, theming |
| `anim` | Tweens, keyframe tracks, four easing curves |
| `time` | Frame delta tracking, one-shot and repeating timers |
| `rand` | xorshift64 RNG — range, pick, shuffle, weighted chance |
| `serde` | Binary serialize/deserialize for primitives, `Vec`, and `String` |
| `intro` | Startup animation sequence |

## Benchmarks

[Criterion](https://github.com/bheisler/criterion.rs), release builds.

### Framebuffer diff

| Scenario | 80x24 | 120x40 | 240x80 |
|----------|-------|--------|--------|
| No changes | 5.5 us | 13.0 us | 51.7 us |
| 5% dirty | 6.3 us | 15.5 us | 62.0 us |
| 100% dirty | 103 us | 259 us | 1.03 ms |

Most frames touch under 5% of cells. At 120x40 that's ~15 us — well inside a 16 ms frame budget.

### I/O

| Stage | 120x40 | Notes |
|-------|--------|-------|
| OS write (NUL sink) | 220 ns | Kernel throughput ceiling |
| ANSI assembly (precoded) | 11.9 us | Memcpy pre-built sequences |
| ANSI assembly (fmt) | 122 us | `write!` formatting |
| Decay diff (5% dirty) | 15.5 us | Actual render path |

The bottleneck is escape-sequence formatting, not I/O. The diff engine skips unchanged cells, so real frames land closer to the precoded path than the full fmt path.

## License

Apache-2.0
