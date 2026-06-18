#!/usr/bin/env python3
"""
blender_convert.py — Headless Blender 3D model → UE4 FBX converter.

Invoked by the Rust BlenderConverter as:
    blender --background --python blender_convert.py -- \
        --input /path/to/model.obj \
        --output /tmp/work/model.fbx \
        --format obj

The LAST LINE of stdout must be a single-line JSON:
    {"ok": true, "output_path": "/abs/path/out.fbx"}
    {"ok": false, "error": "human-readable reason"}

All human-readable logging goes to STDERR.
"""

import sys
import os
import json
import argparse


def result_ok(output_path: str) -> None:
    """Print the success sentinel as the final stdout line."""
    print(json.dumps({"ok": True, "output_path": output_path}), flush=True)


def result_err(error: str) -> None:
    """Print the error sentinel as the final stdout line."""
    print(json.dumps({"ok": False, "error": error}), flush=True)


def parse_args():
    """Parse arguments that appear AFTER the '--' separator Blender inserts."""
    try:
        sep = sys.argv.index("--")
        script_args = sys.argv[sep + 1:]
    except ValueError:
        script_args = sys.argv[1:]

    parser = argparse.ArgumentParser(description="Convert 3D model to UE4 FBX")
    parser.add_argument("--input",  required=True, help="Input 3D model file path")
    parser.add_argument("--output", required=True, help="Output .fbx file path")
    parser.add_argument("--format", required=True,
                        choices=["obj", "fbx", "stl", "dae", "gltf", "glb"],
                        help="Source file format")
    return parser.parse_args(script_args)


def clear_scene(bpy):
    """Remove all default objects from the scene."""
    bpy.ops.object.select_all(action='SELECT')
    bpy.ops.object.delete()
    # Also clear unused data blocks
    for collection in [bpy.data.meshes, bpy.data.materials,
                        bpy.data.lights, bpy.data.cameras]:
        for item in list(collection):
            collection.remove(item)


def import_model(bpy, fmt: str, path: str) -> None:
    """Import the model using the appropriate Blender importer."""
    print(f"[blender_convert] Importing as {fmt}: {path}", file=sys.stderr)

    if fmt == "obj":
        result = bpy.ops.import_scene.obj(filepath=path)
    elif fmt == "fbx":
        result = bpy.ops.import_scene.fbx(filepath=path)
    elif fmt == "stl":
        result = bpy.ops.import_mesh.stl(filepath=path)
    elif fmt == "dae":
        result = bpy.ops.wm.collada_import(filepath=path)
    elif fmt in ("gltf", "glb"):
        result = bpy.ops.import_scene.gltf(filepath=path)
    else:
        raise ValueError(f"Unsupported format: {fmt}")

    if 'FINISHED' not in result:
        raise RuntimeError(f"Import operator returned: {result}")


def export_ue4_fbx(bpy, output_path: str) -> None:
    """Export the scene as UE4-compatible FBX."""
    print(f"[blender_convert] Exporting FBX to: {output_path}", file=sys.stderr)

    os.makedirs(os.path.dirname(output_path) or ".", exist_ok=True)

    result = bpy.ops.export_scene.fbx(
        filepath=output_path,
        use_selection=False,
        # UE4 requires FBX 7.4
        version='FBX_7_4_00',
        # Scale: 1 Blender unit = 1 cm in UE4
        apply_scale_options='FBX_SCALE_ALL',
        # Axis: Blender Z-up → UE4 Z-up with correct orientation
        axis_forward='-Z',
        axis_up='Y',
        # Mesh settings
        use_mesh_modifiers=True,
        mesh_smooth_type='FACE',
        use_tspace=True,
        # Armature settings — critical for UE4
        add_leaf_bones=False,          # UE4 adds virtual leaf bones; double bones = import errors
        use_armature_deform_only=False,
        # Animation
        bake_anim=True,
        bake_anim_use_all_bones=True,
        bake_anim_use_nla_strips=False,
        bake_anim_use_all_actions=False,
        bake_anim_force_startend_keying=True,
    )

    if 'FINISHED' not in result:
        raise RuntimeError(f"FBX export operator returned: {result}")


def main():
    # bpy is only available inside Blender — guard against accidental direct invocation
    try:
        import bpy
    except ImportError:
        result_err(
            "Not running inside Blender — invoke as: "
            "blender --background --python blender_convert.py -- <args>"
        )
        sys.exit(1)

    args = parse_args()

    # Validate input exists
    if not os.path.exists(args.input):
        result_err(f"Input file not found: {args.input}")
        sys.exit(1)

    try:
        clear_scene(bpy)
    except Exception as e:
        print(f"[blender_convert] Warning: could not fully clear scene: {e}",
              file=sys.stderr)

    try:
        import_model(bpy, args.format, args.input)
    except Exception as e:
        result_err(f"Import failed ({args.format}): {e}")
        sys.exit(1)

    try:
        export_ue4_fbx(bpy, args.output)
    except Exception as e:
        result_err(f"FBX export failed: {e}")
        sys.exit(1)

    if not os.path.exists(args.output):
        result_err(
            f"Export appeared to succeed but output file missing: {args.output}"
        )
        sys.exit(1)

    size_kb = os.path.getsize(args.output) / 1024
    print(
        f"[blender_convert] completed. Output: {args.output} ({size_kb:.1f} KB)",
        file=sys.stderr
    )

    # This MUST be the last stdout line
    result_ok(args.output)
    sys.exit(0)


main()
