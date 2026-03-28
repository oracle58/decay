use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group};

#[derive(Clone, Copy, PartialEq, Eq)]
struct Cell {
    ch: char,
    fg: u8,
    bg: u8,
}

impl Cell {
    const EMPTY: Self = Self { ch: ' ', fg: 7, bg: 0 };
    const WALL: Self = Self { ch: '#', fg: 15, bg: 0 };
    const PLAYER: Self = Self { ch: '@', fg: 11, bg: 0 };
}

fn make_grid(cols: usize, rows: usize, fill: Cell) -> Vec<Cell> {
    vec![fill; cols * rows]
}

/// Diff two identical frames. Best case: nothing changed, should be near zero cost.
/// This is the floor. If our diff is slow on identical frames we're doing something wrong.
fn diff_identical(c: &mut Criterion) {
    let mut group = c.benchmark_group("framebuffer_diff");

    for &(cols, rows) in &[(80, 24), (120, 40), (240, 80)] {
        let size = cols * rows;
        let front = make_grid(cols, rows, Cell::EMPTY);
        let back = make_grid(cols, rows, Cell::EMPTY);
        group.throughput(Throughput::Elements(size as u64));

        group.bench_with_input(
            BenchmarkId::new("identical", format!("{cols}x{rows}")),
            &(cols, rows),
            |b, _| {
                b.iter(|| {
                    let mut dirty = 0u32;
                    for i in 0..size {
                        if black_box(front[i]) != black_box(back[i]) {
                            dirty += 1;
                        }
                    }
                    black_box(dirty)
                });
            },
        );
    }

    group.finish();
}

/// Diff two fully different frames. Worst case: every cell changed.
/// Measures raw diff + escape sequence emission throughput.
fn diff_fully_dirty(c: &mut Criterion) {
    let mut group = c.benchmark_group("framebuffer_diff");

    for &(cols, rows) in &[(80, 24), (120, 40), (240, 80)] {
        let size = cols * rows;
        let front = make_grid(cols, rows, Cell::EMPTY);
        let back = make_grid(cols, rows, Cell::WALL);
        let mut out = Vec::with_capacity(size * 20);
        group.throughput(Throughput::Elements(size as u64));

        group.bench_with_input(
            BenchmarkId::new("fully_dirty", format!("{cols}x{rows}")),
            &(cols, rows),
            |b, &(cols, _rows)| {
                b.iter(|| {
                    out.clear();
                    for i in 0..size {
                        if front[i] != back[i] {
                            let row = (i / cols) as u16;
                            let col = (i % cols) as u16;
                            let _ = std::fmt::Write::write_fmt(
                                &mut WriteToBuf(&mut out),
                                format_args!(
                                    "\x1b[{};{}H\x1b[38;5;{}m\x1b[48;5;{}m{}",
                                    row + 1,
                                    col + 1,
                                    back[i].fg,
                                    back[i].bg,
                                    back[i].ch,
                                ),
                            );
                        }
                    }
                    black_box(out.len())
                });
            },
        );
    }

    group.finish();
}

/// Sparse dirty: ~5% of cells changed. Typical game frame.
/// This is the realistic case and the one that matters most.
fn diff_sparse_dirty(c: &mut Criterion) {
    let mut group = c.benchmark_group("framebuffer_diff");

    for &(cols, rows) in &[(80, 24), (120, 40), (240, 80)] {
        let size = cols * rows;
        let front = make_grid(cols, rows, Cell::EMPTY);
        let mut back = make_grid(cols, rows, Cell::EMPTY);
        // Dirty ~5% of cells in a deterministic pattern
        for i in (0..size).step_by(20) {
            back[i] = Cell::PLAYER;
        }
        let mut out = Vec::with_capacity(size * 2);
        group.throughput(Throughput::Elements(size as u64));

        group.bench_with_input(
            BenchmarkId::new("sparse_5pct", format!("{cols}x{rows}")),
            &(cols, rows),
            |b, &(cols, _rows)| {
                b.iter(|| {
                    out.clear();
                    for i in 0..size {
                        if front[i] != back[i] {
                            let row = (i / cols) as u16;
                            let col = (i % cols) as u16;
                            let _ = std::fmt::Write::write_fmt(
                                &mut WriteToBuf(&mut out),
                                format_args!(
                                    "\x1b[{};{}H\x1b[38;5;{}m\x1b[48;5;{}m{}",
                                    row + 1,
                                    col + 1,
                                    back[i].fg,
                                    back[i].bg,
                                    back[i].ch,
                                ),
                            );
                        }
                    }
                    black_box(out.len())
                });
            },
        );
    }

    group.finish();
}

struct WriteToBuf<'a>(&'a mut Vec<u8>);

impl std::fmt::Write for WriteToBuf<'_> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.0.extend_from_slice(s.as_bytes());
        Ok(())
    }
}

criterion_group!(benches, diff_identical, diff_fully_dirty, diff_sparse_dirty);
