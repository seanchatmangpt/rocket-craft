fn main() {
    // When the `napi` feature is enabled, Cargo sets CARGO_FEATURE_NAPI=1.
    // We emit a cfg flag so that conditional compilation inside the crate can
    // key off `#[cfg(napi)]` in addition to `#[cfg(feature = "napi")]`.
    // We do NOT call `napi_build::setup()` here because napi-build is an
    // optional build-dependency; calling it unconditionally would require it
    // to always be present. Instead the napi-build crate is listed as an
    // optional build-dep for CI tooling and future npm-publish scaffolding.
    if std::env::var("CARGO_FEATURE_NAPI").is_ok() {
        use std::io::Write;
        let mut stdout = std::io::stdout();
        let _ = writeln!(stdout, "cargo:rustc-cfg=napi");
        let _ = writeln!(stdout, "cargo:rerun-if-env-changed=CARGO_FEATURE_NAPI");
    }
}
