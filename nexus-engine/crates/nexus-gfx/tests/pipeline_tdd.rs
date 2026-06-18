//! TDD tests for the `nexus-gfx` render pipeline using `chicago-tdd-tools`.
//!
//! All tests operate on the descriptor / configuration layer only — no GPU
//! device is required.  The `gpu` feature is intentionally NOT enabled here
//! so that CI can run these tests without physical or virtual GPU hardware.

use chicago_tdd_tools::TestEnvironment;
use nexus_gfx::pipeline::{
    BlendMode, CullMode, DepthCompare, PipelineSet, RenderPipeline, Uninitialized,
};

// ---------------------------------------------------------------------------
// Test 1 — default builder values
// ---------------------------------------------------------------------------

#[test]
fn test_pipeline_defaults() {
    let _env = TestEnvironment::new().expect("TestEnvironment::new");

    let pipeline = RenderPipeline::<Uninitialized>::new(
        "default_test",
        "vs_source",
        "fs_source",
    );

    assert_eq!(pipeline.cull_mode, CullMode::Back,  "default cull_mode must be Back");
    assert!(pipeline.depth_write,                    "default depth_write must be true");
    assert_eq!(pipeline.blend_mode, BlendMode::Opaque, "default blend_mode must be Opaque");
    assert_eq!(pipeline.depth_compare, DepthCompare::Less, "default depth_compare must be Less");
    assert_eq!(pipeline.label, "default_test");
}

// ---------------------------------------------------------------------------
// Test 2 — with_blend sets blend mode before compile
// ---------------------------------------------------------------------------

#[test]
fn test_with_blend_sets_mode() {
    let _env = TestEnvironment::new().expect("TestEnvironment::new");

    let pipeline = RenderPipeline::<Uninitialized>::new("blend_test", "vs", "fs")
        .with_blend(BlendMode::AlphaBlend);

    assert_eq!(
        pipeline.blend_mode,
        BlendMode::AlphaBlend,
        "with_blend should update blend_mode on the builder"
    );
}

// ---------------------------------------------------------------------------
// Test 3 — compile() transitions typestate; label is preserved
// ---------------------------------------------------------------------------

#[test]
fn test_compile_transitions_typestate_and_preserves_label() {
    let _env = TestEnvironment::new().expect("TestEnvironment::new");

    let label = "opaque_suit";
    let compiled = RenderPipeline::<Uninitialized>::new(label, "suit.vert.wgsl", "suit.frag.wgsl")
        .compile();

    // The return type is `RenderPipeline<Compiled>` — if it weren't, `label()`
    // wouldn't be callable (it only exists on Compiled).
    assert_eq!(compiled.label(), label, "compiled label must match the original");
}

// ---------------------------------------------------------------------------
// Test 4 — PipelineSet::build() creates all 5 named pipelines
// ---------------------------------------------------------------------------

#[test]
fn test_pipeline_set_build_creates_all_five() {
    let _env = TestEnvironment::new().expect("TestEnvironment::new");

    let set = PipelineSet::build();

    assert_eq!(set.opaque.label(),       "opaque");
    assert_eq!(set.transparent.label(),  "transparent");
    assert_eq!(set.beam_effects.label(), "beam");
    assert_eq!(set.ui.label(),           "ui");
    assert_eq!(set.shadow.label(),       "shadow");
}

// ---------------------------------------------------------------------------
// Test 5 — without gpu feature, compile() succeeds and label() is correct
// ---------------------------------------------------------------------------

#[test]
fn test_compile_without_gpu_feature_succeeds() {
    let _env = TestEnvironment::new().expect("TestEnvironment::new");

    // This test deliberately exercises the no-device compile path.
    // When the `gpu` feature is disabled there is no gpu_pipeline field at all,
    // so this test is valid in both configurations.
    let compiled = RenderPipeline::<Uninitialized>::new(
        "no_gpu_pipeline",
        "@vertex fn vs_main() -> @builtin(position) vec4<f32> { return vec4<f32>(0.0, 0.0, 0.0, 1.0); }",
        "@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0, 0.0, 0.0, 1.0); }",
    )
    .compile();

    assert_eq!(
        compiled.label(),
        "no_gpu_pipeline",
        "compile() without a device must still return the correct label"
    );
}
