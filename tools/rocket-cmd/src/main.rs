// wasmer 4.4.0 references __rust_probestack from wasmer-vm; Rust 1.85+ no
// longer emits it automatically. On Linux x86_64 the OS handles guard pages,
// so a no-op stub is safe for debug/development builds.
#[cfg(target_arch = "x86_64")]
core::arch::global_asm!(
    ".globl __rust_probestack",
    "__rust_probestack:",
    "ret",
);

mod compliance;
pub mod lock;
pub mod inspect;
pub mod registry;
mod verbs; // must be imported so linkme slice is populated

fn main() -> clap_noun_verb::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stdout)
        .with_target(false)
        .without_time()
        .with_level(false)
        .init();

    clap_noun_verb::cli::run()
}
