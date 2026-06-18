//! Chicago-style assertion macros — no mocks, just real values.

/// Assert that a `Result` is `Ok`, returning the inner value.
///
/// # Examples
/// ```rust
/// use unify_test::assert_ok;
/// let v = assert_ok!(Ok::<i32, &str>(42));
/// assert_eq!(v, 42);
/// ```
#[macro_export]
macro_rules! assert_ok {
    ($result:expr) => {{
        match $result {
            Ok(v) => v,
            Err(e) => panic!("expected Ok, got Err({:?})", e),
        }
    }};
    ($result:expr, $msg:literal) => {{
        match $result {
            Ok(v) => v,
            Err(e) => panic!("{}: expected Ok, got Err({:?})", $msg, e),
        }
    }};
}

/// Assert that a `Result` is `Err`, returning the inner error.
///
/// # Examples
/// ```rust
/// use unify_test::assert_err;
/// let e = assert_err!(Err::<i32, &str>("boom"));
/// assert_eq!(e, "boom");
/// ```
#[macro_export]
macro_rules! assert_err {
    ($result:expr) => {{
        match $result {
            Err(e) => e,
            Ok(v) => panic!("expected Err, got Ok({:?})", v),
        }
    }};
    ($result:expr, $msg:literal) => {{
        match $result {
            Err(e) => e,
            Ok(v) => panic!("{}: expected Err, got Ok({:?})", $msg, v),
        }
    }};
}

/// Assert that a value lies within `[lo, hi]` (inclusive).
///
/// # Examples
/// ```rust
/// use unify_test::assert_in_range;
/// assert_in_range!(5, 1, 10);
/// ```
#[macro_export]
macro_rules! assert_in_range {
    ($val:expr, $lo:expr, $hi:expr) => {{
        let val = $val;
        let lo = $lo;
        let hi = $hi;
        assert!(
            val >= lo && val <= hi,
            "expected {} to be in [{}, {}]",
            val,
            lo,
            hi
        );
    }};
}

/// Assert that a `serde_json::Value` contains a given key at the top level.
///
/// # Examples
/// ```rust
/// use unify_test::assert_json_has;
/// use serde_json::json;
/// assert_json_has!(json!({"name": "alice"}), "name");
/// ```
#[macro_export]
macro_rules! assert_json_has {
    ($json:expr, $key:literal) => {{
        let v: &serde_json::Value = &$json;
        assert!(
            v.get($key).is_some(),
            "expected JSON to have key '{}', got: {}",
            $key,
            v
        );
    }};
}

/// Assert that two strings are equal after trimming leading/trailing whitespace.
///
/// # Examples
/// ```rust
/// use unify_test::assert_eq_trimmed;
/// assert_eq_trimmed!("  hello  ", "hello");
/// ```
#[macro_export]
macro_rules! assert_eq_trimmed {
    ($left:expr, $right:expr) => {{
        let l = $left.trim();
        let r = $right.trim();
        assert_eq!(l, r, "strings differ after trimming: {:?} vs {:?}", l, r);
    }};
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn assert_ok_passes_on_ok() {
        let v = assert_ok!(Ok::<i32, &str>(7));
        assert_eq!(v, 7);
    }

    #[test]
    #[should_panic(expected = "expected Ok")]
    fn assert_ok_panics_on_err() {
        assert_ok!(Err::<i32, &str>("oops"));
    }

    #[test]
    fn assert_err_passes_on_err() {
        let e = assert_err!(Err::<i32, &str>("boom"));
        assert_eq!(e, "boom");
    }

    #[test]
    #[should_panic(expected = "expected Err")]
    fn assert_err_panics_on_ok() {
        assert_err!(Ok::<i32, &str>(1));
    }

    #[test]
    fn assert_in_range_passes_when_in_range() {
        assert_in_range!(5, 1, 10);
        assert_in_range!(1, 1, 10);
        assert_in_range!(10, 1, 10);
    }

    #[test]
    #[should_panic(expected = "expected 11 to be in")]
    fn assert_in_range_panics_when_outside() {
        assert_in_range!(11, 1, 10);
    }

    #[test]
    fn assert_json_has_passes_when_key_exists() {
        assert_json_has!(json!({"name": "alice", "age": 30}), "name");
    }

    #[test]
    #[should_panic(expected = "expected JSON to have key")]
    fn assert_json_has_panics_when_key_missing() {
        assert_json_has!(json!({"name": "alice"}), "email");
    }

    #[test]
    fn assert_eq_trimmed_passes_on_equal_after_trim() {
        assert_eq_trimmed!("  hello  ", "hello");
        assert_eq_trimmed!("world\n", "world");
    }

    #[test]
    #[should_panic(expected = "strings differ after trimming")]
    fn assert_eq_trimmed_panics_on_different() {
        assert_eq_trimmed!("hello", "world");
    }
}
