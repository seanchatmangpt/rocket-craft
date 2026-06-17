fn main() {
    // When the `napi` feature is enabled, Cargo sets CARGO_FEATURE_NAPI=1.
    // We emit a cfg flag so that conditional compilation inside the crate can
    // key off `#[cfg(napi)]` in addition to `#[cfg(feature = "napi")]`.
    // We do NOT call `napi_build::setup()` here because napi-build is an
    // optional build-dependency; calling it unconditionally would require it
    // to always be present. Instead the napi-build crate is listed as an
    // optional build-dep for CI tooling and future npm-publish scaffolding.
    if std::env::var("CARGO_FEATURE_NAPI").is_ok() {
        println!("cargo:rustc-cfg=napi");
        println!("cargo:rerun-if-env-changed=CARGO_FEATURE_NAPI");
    }
}
