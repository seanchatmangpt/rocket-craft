/// A shared memory bus backed by a `Vec<i32>` for the native target.
///
/// On WASM the same interface would be backed by a `SharedArrayBuffer` +
/// Atomics, but those require COOP/COEP headers and `wasm32` compile target.
/// The cfg gates below isolate that path while keeping the native build
/// (and its unit tests) free of browser dependencies.
pub struct SharedMemoryBus {
    buffer: Vec<i32>,
    size: usize,
}

impl SharedMemoryBus {
    pub fn new(size: usize) -> Self {
        SharedMemoryBus {
            buffer: vec![0i32; size],
            size,
        }
    }

    pub fn write_i32(&mut self, offset: usize, value: i32) -> Result<(), BusError> {
        if offset >= self.size {
            return Err(BusError::OutOfBounds { offset, size: self.size });
        }
        self.buffer[offset] = value;
        Ok(())
    }

    pub fn read_i32(&self, offset: usize) -> Result<i32, BusError> {
        if offset >= self.size {
            return Err(BusError::OutOfBounds { offset, size: self.size });
        }
        Ok(self.buffer[offset])
    }

    /// Atomic compare-and-swap simulation (single-threaded native).
    /// Returns `true` if the swap was performed (i.e. the value at `offset`
    /// equalled `expected` and was replaced with `new_val`).
    pub fn compare_exchange(&mut self, offset: usize, expected: i32, new_val: i32) -> bool {
        if offset >= self.size {
            return false;
        }
        if self.buffer[offset] == expected {
            self.buffer[offset] = new_val;
            true
        } else {
            false
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

// ---------------------------------------------------------------------------
// BusError
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum BusError {
    OutOfBounds { offset: usize, size: usize },
}

impl std::fmt::Display for BusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BusError::OutOfBounds { offset, size } => {
                write!(f, "offset {} is out of bounds for bus of size {}", offset, size)
            }
        }
    }
}

impl std::error::Error for BusError {}
