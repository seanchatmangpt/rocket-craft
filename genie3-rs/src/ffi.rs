use crate::types::{Vector3, Rotation3D, Transform, Bounds3D};

#[no_mangle]
pub extern "C" fn genie3_vector_new(x: f32, y: f32, z: f32) -> Vector3 {
    Vector3::new(x, y, z)
}

#[no_mangle]
pub extern "C" fn genie3_vector_add(a: Vector3, b: Vector3) -> Vector3 {
    a.add(&b)
}

#[no_mangle]
pub extern "C" fn genie3_vector_distance(a: Vector3, b: Vector3) -> f32 {
    a.distance(&b)
}
