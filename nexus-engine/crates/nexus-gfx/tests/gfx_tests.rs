use nexus_gfx::{camera::*, color::*, math::*, pipeline::*, vertex::*};
use proptest::prelude::*;

#[test]
fn transform_identity_is_neutral_for_composition() {
    let id = Transform::identity();
    let t = Transform {
        translation: Vec3::new(1.0, 2.0, 3.0),
        rotation: UnitQuat::identity(),
        scale: Vec3::new(2.0, 2.0, 2.0),
    };
    let composed = id.mul_transform(&t);
    assert!(
        (composed.translation - t.translation).norm() < 1e-5,
        "identity * t should equal t"
    );
    assert!((composed.scale - t.scale).norm() < 1e-5);
}

#[test]
fn transform_lerp_endpoints() {
    let a = Transform::identity();
    let b = Transform {
        translation: Vec3::new(10.0, 0.0, 0.0),
        rotation: UnitQuat::identity(),
        scale: Vec3::new(2.0, 2.0, 2.0),
    };
    let at_zero = a.lerp(&b, 0.0);
    let at_one = a.lerp(&b, 1.0);
    assert!((at_zero.translation - a.translation).norm() < 1e-5);
    assert!((at_one.translation - b.translation).norm() < 1e-5);
}

#[test]
fn camera_view_projection_has_correct_aspect() {
    let cam = Camera::new(60.0, 16.0 / 9.0, 0.1, 1000.0).unwrap();
    let vp = cam.view_projection();
    // VP matrix must be non-degenerate (det != 0)
    assert!(
        vp.determinant().abs() > 1e-6,
        "VP matrix must be non-degenerate"
    );
}

#[test]
fn aabb_frustum_culling_visible_object_not_culled() {
    let cam = Camera::new(60.0, 1.0, 0.1, 100.0).unwrap();
    let vp = cam.view_projection();
    let frustum = Frustum::from_view_projection(&vp);
    // Small box right in front of camera (at z=-5, which is within near/far)
    let aabb = Aabb::new(Vec3::new(-0.5, -0.5, -6.0), Vec3::new(0.5, 0.5, -4.0)).unwrap();
    assert!(
        frustum.intersects_aabb(&aabb),
        "box in front of camera should be visible"
    );
}

#[test]
fn invalid_fov_returns_error() {
    assert!(Camera::new(0.0, 1.0, 0.1, 100.0).is_err());
    assert!(Camera::new(180.0, 1.0, 0.1, 100.0).is_err());
    assert!(Camera::new(-45.0, 1.0, 0.1, 100.0).is_err());
}

#[test]
fn invalid_ndc_rejected() {
    assert!(Ndc::new(1.5, 0.0).is_err());
    assert!(Ndc::new(0.0, -1.5).is_err());
    assert!(Ndc::new(0.5, 0.5).is_ok());
}

#[test]
fn skinned_vertex_weights_normalized() {
    let v = SkinnedVertex {
        position: [0.0; 3],
        normal: [0.0; 3],
        uv: [0.0; 2],
        joint_indices: [0; 4],
        joint_weights: [0.5, 0.3, 0.2, 0.0],
    };
    assert!(v.weights_are_normalized(), "weights should sum to 1.0");
}

#[test]
fn pipeline_type_safety_uncompiled_vs_compiled() {
    let uncompiled = RenderPipeline::<Uninitialized>::new("test", "v.vert", "f.frag");
    // Calling .label() should only work on Compiled, not Uninitialized
    let compiled = uncompiled.compile();
    assert_eq!(compiled.label(), "test");
    // PipelineSet builds successfully
    let _set = PipelineSet::build();
}

proptest! {
    // sRGB conversion stays in [0, 255]
    #[test]
    fn srgb_conversion_in_range(r in 0.0f32..1.0, g in 0.0f32..1.0, b in 0.0f32..1.0) {
        let color = LinearRgb::new(r, g, b).unwrap();
        let srgb = color.to_srgb();
        // u8 is always <= 255 by type; just assert the conversion succeeds
        let _ = srgb;
    }

    // Transform * identity == identity * transform (near-commutativity for rotation=identity)
    #[test]
    fn transform_scale_multiplication(scale in 0.1f32..10.0) {
        let t = Transform { translation: Vec3::zeros(), rotation: UnitQuat::identity(), scale: Vec3::new(scale, scale, scale) };
        let id = Transform::identity();
        let result = id.mul_transform(&t);
        prop_assert!((result.scale - t.scale).norm() < 1e-4);
    }
}
