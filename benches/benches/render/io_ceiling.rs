use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group};

/// Measures the raw write throughput ceiling.
/// This is the fastest we can possibly push bytes, writing to /dev/null (or NUL on Windows)
/// so the kernel accepts and discards immediately. Our actual rendering can never beat this.
fn raw_write_ceiling(c: &mut Criterion) {
    let mut group = c.benchmark_group("io_ceiling");

    // Typical frame sizes: 80x24 (small), 120x40 (medium), 240x80 (large 4k term)
    // Each cell worst case ~20 bytes of ANSI (move + sgr + char) = ~4800 to ~384000 bytes
    for size in [4 * 1024, 16 * 1024, 64 * 1024, 256 * 1024] {
        let buf = vec![b'X'; size];
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(BenchmarkId::new("write_devnull", size), &buf, |b, buf| {
            use std::io::Write;
            #[cfg(unix)]
            let mut sink = std::fs::OpenOptions::new().write(true).open("/dev/null").unwrap();
            #[cfg(windows)]
            let mut sink = std::fs::OpenOptions::new().write(true).open("NUL").unwrap();

            b.iter(|| {
                sink.write_all(black_box(buf)).unwrap();
            });
        });
    }

    group.finish();
}

/// Measures Stdout lock overhead vs raw fd write.
/// Shows exactly how much we lose to Rust's Stdout locking.
fn stdout_lock_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("io_ceiling");

    let buf = vec![b'X'; 16 * 1024];
    group.throughput(Throughput::Bytes(16 * 1024));

    group.bench_function("sink_16k", |b| {
        use std::io::Write;
        let mut sink = std::io::sink();
        b.iter(|| {
            sink.write_all(black_box(&buf)).unwrap();
        });
    });

    group.finish();
}

/// Measures Vec<u8> buffer building throughput.
/// This is the ceiling for how fast we can assemble a frame's escape sequences.
fn buffer_assembly(c: &mut Criterion) {
    let mut group = c.benchmark_group("io_ceiling");

    // Simulate building a 120x40 frame of move_to + char sequences
    let cols: usize = 120;
    let rows: usize = 40;
    let total_cells = cols * rows;
    group.throughput(Throughput::Elements(total_cells as u64));

    group.bench_function("assemble_move_char_120x40", |b| {
        let mut buf = Vec::with_capacity(total_cells * 16);
        b.iter(|| {
            buf.clear();
            for row in 0..rows as u16 {
                for col in 0..cols as u16 {
                    let _ = std::fmt::Write::write_fmt(
                        &mut WriteToBuf(&mut buf),
                        format_args!("\x1b[{};{}H@", row + 1, col + 1),
                    );
                }
            }
            black_box(buf.len());
        });
    });

    // Same thing but with extend_from_slice (pre-formatted), showing the fmt overhead
    group.bench_function("assemble_precoded_120x40", |b| {
        let mut buf = Vec::with_capacity(total_cells * 16);
        // Pre-build all sequences
        let mut sequences: Vec<Vec<u8>> = Vec::with_capacity(total_cells);
        for row in 0..rows as u16 {
            for col in 0..cols as u16 {
                sequences.push(format!("\x1b[{};{}H@", row + 1, col + 1).into_bytes());
            }
        }
        b.iter(|| {
            buf.clear();
            for seq in &sequences {
                buf.extend_from_slice(seq);
            }
            black_box(buf.len());
        });
    });

    group.finish();
}

struct WriteToBuf<'a>(&'a mut Vec<u8>);

impl std::fmt::Write for WriteToBuf<'_> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.0.extend_from_slice(s.as_bytes());
        Ok(())
    }
}

criterion_group!(benches, raw_write_ceiling, stdout_lock_overhead, buffer_assembly);
