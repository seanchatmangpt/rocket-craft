# TRANSFER FUNCTION REPORT

## Factor to Failure Mapping

- **Chassis Factor (X_chassis)**:
  - `torso_contains_foreign_parts` -> Fails at `MODULAR_USD` with `USD303` defect.
  - `socket_contains_mesh_payload` -> Fails at `MODULAR_USD` with `USD311` defect.
  - `assembly_reference_inside_part_file` -> Fails at `MODULAR_USD` with `USD308` defect.
  - `duplicate_part_fingerprint` -> Fails at `MODULAR_USD` with `USD301` defect.
  - `missing_owner_part_id` -> Fails at `MODULAR_USD` with `USD304` defect.
  - `invalid_envelope_exceeded` -> Fails at `MODULAR_USD` with `USD307` defect.

- **Surface Factor (X_surface)**:
  - `missing_manifest` -> Fails at `PBR_MANIFEST` with `PBR_MANIFEST_MISSING` defect.
  - `missing_basecolor` -> Fails at `PBR_MANIFEST` with `PBR_BASECOLOR_MISSING` defect.
  - `missing_metallic` -> Fails at `PBR_MANIFEST` with `PBR_METALLIC_MISSING` defect.

- **Rig Factor (X_rig)**:
  - `missing_sockets` -> Fails at `RIG_SOCKET` with `RIG_SOCKETS_MISSING` defect.
  - `invalid_joint_limits` -> Fails at `RIG_SOCKET` with `RIG_JOINT_LIMITS_INVALID` defect.

- **Loadout Factor (X_loadout)**:
  - `invalid_weapon_mounts` -> Fails at `RIG_SOCKET` with `RIG_WEAPON_MOUNTS_INVALID` defect.

- **Destruction Factor (X_destruction)**:
  - `missing_destruction_states` -> Fails at `DESTRUCTION` with `DESTRUCTION_STATES_MISSING` defect.
