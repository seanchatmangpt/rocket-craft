//! Fixture — owns a real collaborator and optionally runs teardown on drop.
//! Models the Chicago-style TDD philosophy of using real objects, not mocks.

/// Holds a value `T` and an optional teardown function executed on [`Drop`].
pub struct Fixture<T> {
    value: T,
    teardown: Option<Box<dyn FnOnce(&mut T)>>,
}

impl<T> Fixture<T> {
    /// Create a `Fixture` wrapping `value` with no teardown.
    pub fn new(value: T) -> Self {
        Self { value, teardown: None }
    }

    /// Create a `Fixture` wrapping `value`; `teardown` is called with a
    /// mutable reference to the value when the fixture is dropped.
    pub fn with_teardown(value: T, teardown: impl FnOnce(&mut T) + 'static) -> Self {
        Self { value, teardown: Some(Box::new(teardown)) }
    }

    /// Immutable access to the wrapped value.
    pub fn get(&self) -> &T {
        &self.value
    }

    /// Mutable access to the wrapped value.
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T> Drop for Fixture<T> {
    fn drop(&mut self) {
        if let Some(f) = self.teardown.take() {
            f(&mut self.value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn get_returns_value() {
        let f: Fixture<i32> = Fixture::new(42);
        assert_eq!(*f.get(), 42);
    }

    #[test]
    fn get_mut_can_modify_value() {
        let mut f: Fixture<i32> = Fixture::new(10);
        *f.get_mut() += 5;
        assert_eq!(*f.get(), 15);
    }

    #[test]
    fn drop_runs_teardown() {
        let flag = Arc::new(Mutex::new(false));
        let flag_clone = Arc::clone(&flag);

        {
            let _f = Fixture::with_teardown(42i32, move |_v| {
                *flag_clone.lock().unwrap() = true;
            });
        } // dropped here

        assert!(*flag.lock().unwrap(), "teardown should have run on drop");
    }

    #[test]
    fn no_teardown_fixture_drops_cleanly() {
        let _f: Fixture<String> = Fixture::new("hello".to_string());
        // Just make sure it doesn't panic on drop
    }
}
