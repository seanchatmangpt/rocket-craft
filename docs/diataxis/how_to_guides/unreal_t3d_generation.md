# How to Compile World Specifications to Unreal Engine T3D Layouts

This guide provides instructions on how to compile a JSON World Specification (`WorldSpec`) into a copy-pasteable Unreal Engine 4 `.t3d` level map file using the Unify command-line tools.

---

## The Compilation Pipeline

When you run the compiler:
1. **Receipt Chaining**: The tool generates and appends a BLAKE3 cryptographic receipt chain over the world specification's properties using `genie_core::receipt_chain::ReceiptChainManager`. This guarantees specification integrity and saves the updated spec back to disk.
2. **Layout Compilation**: The compiler resolves relative positioning (actors/objects are defined relative to their containing `Place` center) into absolute 3D coordinates.
3. **Actor Mesh Generation**:
   - **Places** generate standard floor boxes using `Cube.Cube` static meshes.
   - **Actors** generate biped-sized cylinders using `Cylinder.Cylinder` meshes.
   - **Objects** generate spherical props using `Sphere.Sphere` meshes.
4. **T3D Text Output**: The level layout is written to disk in the clipboard format understood by the Unreal Editor.

---

## Step-by-Step Instructions

### Step 1: Author a Compliant World Specification
Create a file named `my_world_spec.json`. A minimally compliant world specification must define an `engine_version`, at least one `Place` (which acts as the floor / spatial parent), and optionally `actors` and `objects` placed inside it.

Write the following content to `my_world_spec.json`:

```json
{
  "engine_version": "UE4.27-ES3",
  "places": [
    {
      "id": "FactoryFloor",
      "name": "Assembly Line Alpha",
      "bounds": {
        "center": { "x": 0.0, "y": 0.0, "z": 100.0 },
        "half_extents": { "x": 500.0, "y": 500.0, "z": 10.0 }
      },
      "properties": {}
    }
  ],
  "actors": [
    {
      "id": "SupervisorRobot",
      "name": "Unit_01_Supervisor",
      "role": "RoboticSupervisor",
      "place_id": "FactoryFloor",
      "placement": {
        "position": { "x": -200.0, "y": 50.0, "z": 50.0 },
        "rotation": { "x": 0.0, "y": 90.0, "z": 0.0 }
      },
      "properties": {}
    }
  ],
  "objects": [
    {
      "id": "AssemblerMachine",
      "name": "CNC_Mill_Alpha",
      "class": "MillMachine",
      "place_id": "FactoryFloor",
      "placement": {
        "position": { "x": 150.0, "y": -100.0, "z": 20.0 },
        "rotation": { "x": 0.0, "y": 180.0, "z": 0.0 }
      },
      "properties": {},
      "tags": ["machinery", "heavy"]
    }
  ],
  "relationships": [],
  "rules": [],
  "history": [],
  "processes": [],
  "receipts": []
}
```

### Step 2: Compile the Specification to T3D
Run the `world-generate` command, specifying the path to your spec JSON and the path where you want the resulting `.t3d` level file to be saved:

```bash
cargo run -p unify -- world-generate --spec ./my_world_spec.json --output ./my_level_layout.t3d
```

You can also compile layout files as part of the unified `genie manufacture` workflow:
```bash
cargo run -p unify -- genie manufacture --intent "Build a factory floor with a supervisor robot and CNC mill" --out-spec ./my_world_spec.json --out-t3d ./my_level_layout.t3d
```

### Step 3: Verify the Output T3D File
Open the generated `my_level_layout.t3d` file. The format starts with map/level headers and lists the actors with their coordinates and UE asset references:

```text
Begin Map
   Begin Level
      Begin Actor Class=StaticMeshActor Name=Place_FactoryFloor Archetype=StaticMeshActor'/Script/Engine.Default__StaticMeshActor'
        Begin Object Class=StaticMeshComponent Name=StaticMeshComponent0 ObjName=StaticMeshComponent0 Archetype=StaticMeshComponent'/Script/Engine.Default__StaticMeshActor:StaticMeshComponent0'
        End Object
        Begin Object Name="StaticMeshComponent0"
           StaticMesh=StaticMesh'/Engine/BasicShapes/Cube.Cube'
           RelativeLocation=(X=0.000000,Y=0.000000,Z=40.000000)
           RelativeRotation=(Pitch=0.000000,Yaw=0.000000,Roll=0.000000)
           RelativeScale3D=(X=10.000000,Y=10.000000,Z=1.000000)
        End Object
        StaticMeshComponent=StaticMeshComponent0
        RootComponent=StaticMeshComponent0
        ActorLabel="Floor_Assembly Line Alpha"
     End Actor
...
   End Level
End Map
```

*Notice how `RelativeLocation.Z` for the floor floor is offset downward by `half_extents.z + 50.0` so that actors stand perfectly flush on top of the surface.*

---

## Importing into Unreal Engine 4 Editor

Because the `.t3d` file conforms to Unreal Engine's native clipboard format, importing is a direct copy-paste operation:

1. Open your project in the **Unreal Engine 4 Editor** (version 4.24.x or 4.27.x).
2. Open the **Level Viewport** of the level you want to populate.
3. Open the compiled `my_level_layout.t3d` file in any text editor and **select all text (Ctrl+A)** and **copy it (Ctrl+C)**.
4. Click inside the level viewport to focus it.
5. Press **Ctrl+V** (Paste).
6. The compiled actor list will instantiate immediately at their exact 3D coordinates.

### Class Resolution Warning
If the editor outputs warnings that a Blueprint class (e.g. `/Game/BP_RoboticSupervisor.BP_RoboticSupervisor_C`) cannot be resolved:
1. Ensure your project's Content folder has the corresponding Blueprint asset (`BP_RoboticSupervisor`).
2. If the class exists at a different path, edit the `role` (for actors) or `class` (for objects) in the `my_world_spec.json` to specify the absolute class path (e.g., `"/Game/Industrial/Blueprints/BP_Supervisor.BP_Supervisor_C"`).
