mod resources;
mod spawn;

use criterion::criterion_main;

criterion_main!(spawn::benches, resources::benches);
