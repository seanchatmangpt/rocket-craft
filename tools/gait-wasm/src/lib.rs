// Genuine WebAssembly Gait Engine

#[no_mangle]
pub extern "C" fn get_gait_rotation_x(bone_id: i32, sec: f32, is_moving: i32, is_space: i32) -> f32 {
    let mut rot = 0.0;
    if is_moving != 0 {
        if bone_id == 6 { // left_leg
            rot = (sec * 12.0).sin() * 0.7;
        } else if bone_id == 7 { // right_leg
            rot = -(sec * 12.0).sin() * 0.7;
        } else if bone_id == 4 { // left_arm
            rot = -(sec * 12.0).sin() * 0.4;
        } else if bone_id == 5 { // right_arm
            rot = (sec * 12.0).sin() * 0.4;
        }
    }
    if is_space != 0 && (bone_id == 6 || bone_id == 7) {
        rot = -0.5;
    }
    rot
}

#[no_mangle]
pub extern "C" fn get_gait_rotation_z(bone_id: i32, _sec: f32, _is_moving: i32, is_space: i32) -> f32 {
    let mut rot = 0.0;
    if is_space != 0 {
        if bone_id == 4 { // left_arm
            rot = 1.3;
        } else if bone_id == 5 { // right_arm
            rot = -1.3;
        }
    }
    rot
}

#[no_mangle]
pub extern "C" fn get_foundry_rotation(bone_id: i32, axis: i32, sec: f32) -> f32 {
    // bone_id: 1 = spine, 2 = torso, 3 = head, 4 = left_arm, 5 = right_arm, 6 = left_leg, 7 = right_leg
    match bone_id {
        1 => { // spine: [sec * 0.5, sec * 0.7, sec * 0.3]
            match axis {
                0 => sec * 0.5,
                1 => sec * 0.7,
                2 => sec * 0.3,
                _ => 0.0,
            }
        }
        2 => { // torso: [sec * 0.6, sec * 0.4, sec * 0.8]
            match axis {
                0 => sec * 0.6,
                1 => sec * 0.4,
                2 => sec * 0.8,
                _ => 0.0,
            }
        }
        3 => { // head: [sec * 0.3, sec * 0.9, sec * 0.2]
            match axis {
                0 => sec * 0.3,
                1 => sec * 0.9,
                2 => sec * 0.2,
                _ => 0.0,
            }
        }
        4 => { // left_arm: [sec * 0.8, sec * 0.2, sec * 0.5]
            match axis {
                0 => sec * 0.8,
                1 => sec * 0.2,
                2 => sec * 0.5,
                _ => 0.0,
            }
        }
        5 => { // right_arm: [sec * 0.4, sec * 0.6, sec * 0.9]
            match axis {
                0 => sec * 0.4,
                1 => sec * 0.6,
                2 => sec * 0.9,
                _ => 0.0,
            }
        }
        6 => { // left_leg: [sec * 0.9, sec * 0.3, sec * 0.4]
            match axis {
                0 => sec * 0.9,
                1 => sec * 0.3,
                2 => sec * 0.4,
                _ => 0.0,
            }
        }
        7 => { // right_leg: [sec * 0.2, sec * 0.8, sec * 0.6]
            match axis {
                0 => sec * 0.2,
                1 => sec * 0.8,
                2 => sec * 0.6,
                _ => 0.0,
            }
        }
        _ => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gait_rotations() {
        assert_eq!(get_gait_rotation_x(6, 0.0, 1, 0), 0.0);
        assert_eq!(get_gait_rotation_x(6, 0.0, 1, 1), -0.5);
        assert_eq!(get_gait_rotation_z(4, 0.0, 0, 1), 1.3);
        assert_eq!(get_foundry_rotation(3, 1, 1.0), 0.9);
    }
}
