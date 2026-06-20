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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_bus_reads_zero() {
        let bus = SharedMemoryBus::new(4);
        assert_eq!(bus.read_i32(0).unwrap(), 0);
        assert_eq!(bus.size(), 4);
    }

    #[test]
    fn write_and_read_roundtrips() {
        let mut bus = SharedMemoryBus::new(4);
        bus.write_i32(2, 999).unwrap();
        assert_eq!(bus.read_i32(2).unwrap(), 999);
    }

    #[test]
    fn out_of_bounds_write_returns_err() {
        let mut bus = SharedMemoryBus::new(4);
        assert!(bus.write_i32(4, 1).is_err());
    }

    #[test]
    fn out_of_bounds_read_returns_err() {
        let bus = SharedMemoryBus::new(4);
        assert!(bus.read_i32(10).is_err());
    }

    #[test]
    fn compare_exchange_swaps_when_expected_matches() {
        let mut bus = SharedMemoryBus::new(4);
        bus.write_i32(0, 10).unwrap();
        assert!(bus.compare_exchange(0, 10, 20));
        assert_eq!(bus.read_i32(0).unwrap(), 20);
    }

    #[test]
    fn compare_exchange_no_swap_when_expected_mismatches() {
        let mut bus = SharedMemoryBus::new(4);
        bus.write_i32(0, 5).unwrap();
        assert!(!bus.compare_exchange(0, 99, 20));
        assert_eq!(bus.read_i32(0).unwrap(), 5);
    }

    #[test]
    fn compare_exchange_out_of_bounds_returns_false() {
        let mut bus = SharedMemoryBus::new(4);
        assert!(!bus.compare_exchange(100, 0, 1));
    }
}
