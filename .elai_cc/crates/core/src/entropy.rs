use rand::RngCore;
use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;

/// Abstraction over entropy sources.
/// Methods take `&mut self` — wrap in `Arc<Mutex<dyn EntropySource>>` for shared access.
pub trait EntropySource: Send {
    fn next_u64(&mut self) -> u64;
    fn fill_bytes(&mut self, buf: &mut [u8]);
}

// ---------------------------------------------------------------------------
// Production implementation
// ---------------------------------------------------------------------------

/// Delegates to the OS random number generator.
pub struct OsEntropy;

impl EntropySource for OsEntropy {
    fn next_u64(&mut self) -> u64 {
        rand::rngs::OsRng.next_u64()
    }

    fn fill_bytes(&mut self, buf: &mut [u8]) {
        rand::rngs::OsRng.fill_bytes(buf);
    }
}

// ---------------------------------------------------------------------------
// Test / deterministic implementation
// ---------------------------------------------------------------------------

/// Deterministic RNG backed by ChaCha8. Same seed = same sequence.
pub struct SeededRng {
    inner: ChaCha8Rng,
}

impl SeededRng {
    pub fn new(seed: u64) -> Self {
        Self {
            inner: ChaCha8Rng::seed_from_u64(seed),
        }
    }
}

impl EntropySource for SeededRng {
    fn next_u64(&mut self) -> u64 {
        self.inner.next_u64()
    }

    fn fill_bytes(&mut self, buf: &mut [u8]) {
        self.inner.fill_bytes(buf);
    }
}
