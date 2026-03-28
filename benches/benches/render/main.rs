mod framebuffer;
mod io_ceiling;

use criterion::criterion_main;

criterion_main!(framebuffer::benches, io_ceiling::benches);
