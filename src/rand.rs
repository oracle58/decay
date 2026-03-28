use crate::core::Store;

/// Seedable xorshift64 random number generator.
pub struct Rng(u64);

impl Store for Rng {}

impl Rng {
    pub fn new() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(98765);
        Self::from_seed(seed)
    }

    pub fn from_seed(seed: u64) -> Self {
        Self(seed | 1)
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    pub fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() & 0xFFFFFF) as f32 / 16777216.0
    }

    pub fn range(&mut self, min: i32, max: i32) -> i32 {
        if min >= max {
            return min;
        }
        min + (self.next_u64() % (max - min + 1) as u64) as i32
    }

    pub fn range_f32(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }

    pub fn pick<'a, T>(&mut self, slice: &'a [T]) -> &'a T {
        assert!(!slice.is_empty(), "Rng::pick: empty slice");
        &slice[self.next_u64() as usize % slice.len()]
    }

    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        let len = slice.len();
        for i in (1..len).rev() {
            let j = self.next_u64() as usize % (i + 1);
            slice.swap(i, j);
        }
    }

    pub fn chance(&mut self, probability: f32) -> bool {
        self.next_f32() < probability
    }
}
