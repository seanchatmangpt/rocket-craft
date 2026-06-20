# F1 Geometry & Morphology Exploration Report — explorer_geom_2

## 1. Summary of Investigation
This report details the concrete implementation strategy for the `FLAGSHIP_UE4_MECH_PLANT_001` mecha asset pipeline. By analyzing the merged Turtle ontologies, inline SPARQL configurations, and the `mecha_offline.test.ts` suite, we identify structural gaps in destruction states, locomotion animations, and loadout representation. We recommend a unified strategy using OpenUSD VariantSets for destruction states, semantic locomotion properties for heavy animations, and explicit attachment socket individuals for loadouts.

---

## 2. Current State Analysis

### A. Turtle Ontologies
The current mecha asset's semantic model resides in `ontology/all_merged.ttl`, which aggregates:
- **Core Primitives (`core.ttl`)**: Defines basic typestates, anti-Cartesian boundaries, and SIMD/Scalar byte-class vectors.
- **Asset Fabric (`asset_fabric.ttl`)**: Declares the structural taxonomy of mecha parts (e.g., `mud:TorsoCore`, `mud:BladeLeft`) and primitive families (`mud:TaperedBox`, `mud:FeatherPanel`, `mud:BladePrism`, `mud:Cylinder`).
- **Generator Parameters (`generator_parameters.ttl`)**: Contains the discrete spatial coordinate instances (translateX/Y/Z, scaleX/Y/Z, rotateX/Y/Z) and material bindings.

*Observed Defect*: Currently, there are no definitions or parameters mapping destruction states (`broken armor`, `exposed frames`, `VFX sockets`), locomotion animations (`idle`, `walk`, `deploy`), or weapon loadout configurations in the ontology graphs.

### B. SPARQL Queries
In `ggen.toml`, the generation rules for the USD parts (e.g., `SM_Torso`, `SM_Head`, `SM_WingArray_Left`, `SM_WingArray_Right`, `SM_Blade_Left`, `SM_Blade_Right`) extract geometry primitive data using inline SPARQL queries that bind `?CURRENT_PART_ID` to target parts. This prevents foreign component inclusion (USD303) and isolates part definitions (USD302).

### C. Templates
The `part_mesh.usda.tera` template renders geometry primitives based on the family type (`tapered_box`, `feather_panel`, `blade_prism`, `cylinder`) and binds materials (e.g., `M_WhiteArmor`, `M_CyanBlade`, `M_DarkFrame`, `M_GoldVisor`).

### D. Offline Test Suite (`mecha_offline.test.ts`)
The offline test verifies structural layout and metrics:
- Enforces OpenUSD rules (USD301-307) including DefaultPrim, Y-up axis, 0.01 metersPerUnit, and mirrored X-axis translations.
- Checks morphology boundaries (VIS201-208) including silhouette IoU (>=0.25), wing feather count (>=48), torso core volume (> 2x head volume), and PBR material presence.
- Verifies rigging structure (joint mapping, hierarchy, VFX attachment placeholder) and the cook receipt presence.

---

## 3. Concrete Implementation Strategy

### A. Geometry & Morphology (Milestone 1)

#### 1. Swept Feather Panels
To satisfy `CTQ-F1-001` (cinematic silhouette complexity) and `VJ-CRIT-001` (silhouette authority), the wing feather arrays must be parameterized mathematically in `generator_parameters.ttl`.
- **Panel Count**: A minimum of 48 total panels (24 left, 24 right) must be declared.
- **Sweep Parameterization**: Let $i \in [0, N-1]$ be the feather index. The translation and rotation values should scale quadratically to form a swept fan shape:
  $$X_i = X_{root} \mp (a \cdot i + b \cdot i^2)$$
  $$Z_i = Z_{root} + c \cdot i$$
  $$\theta_{y, i} = \theta_{y, root} \pm (d \cdot i + e \cdot i^2)$$
- **Mirroring Symmetry**: The coordinates must pass the symmetry check:
  - Left wing panel translates: $-X, Y, Z$ and rotations: $RX, RY, RZ$.
  - Right wing panel translates: $+X, Y, Z$ and rotations: $RX, -RY, -RZ$.

#### 2. Angular Armor Shell Hierarchy
To satisfy `CTQ-F1-002`, `CTQ-F1-003`, and `VJ-CRIT-002` (hard-surface detail density):
- **Core Frame / Chassis**: Model the internal skeletal structure using `cylinder` primitives bound to `M_DarkFrame` representing heavy structural steel rods.
- **Armor Shells**: Overlay the chassis with angular, multi-faceted `tapered_box` panels bound to `M_WhiteArmor` to form the protective casing. Enforce a high primitive density (minimum 1000 primitives total across all parts) to represent mechanical complexity suitable for cinematic close-ups.

#### 3. Torso / Head Distinction
- **Volume Ratio**: The torso primitives must sum to a volume at least 2 times larger than the head primitives.
- **Characteristic Features**:
  - The torso core aggregates shoulders, legs, arms, and wing mounts.
  - The head unit contains a central visor of type `tapered_box` bound to `M_GoldVisor` (emissive) and mirrored V-fin antennas (`mud:v_fin_left`, `mud:v_fin_right`) of type `blade_prism` angled outward at the forehead.

#### 4. Cyan Blade Rods
- Placed symmetrically on the hands/arms (opposite signs on the X-axis).
- Modeled as `blade_prism` bound to `M_CyanBlade` with metallic (`0.9`) and emissive (`0.8`) surfaced MaterialX properties.

---

### B. Destruction & Locomotion Animation States (Milestone 5)

#### 1. Destruction States (`CTQ-F1-008`, `VJ-CRIT-005`)
We propose representing destruction states using OpenUSD **VariantSets** defined in the part templates. This allows a single asset file to switch between intact and battle-damaged representations dynamically in the engine.
- **Intact Variant**: Renders the complete, undamaged white armor plates.
- **Damaged Variant**:
  - **Exposed Frame**: Drops rendering for damaged white armor plates, exposing the underlying dark frame cylinders.
  - **Broken Armor**: Renders fractured/shattered versions of the armor panels (subdivided tapered boxes with jagged vertices).
  - **VFX Sockets**: Spawns dedicated attachment points (`socket_sparks_01`, `socket_smoke_01`) at the boundary of the broken plates to emit particle effects.

#### 2. Locomotion Animations (`CTQ-F1-007`, `VJ-CRIT-006`)
Heavy locomotion sequences must be declared in the ontology and projected to UE4:
- `idle`: Stable breathing loop, shifting center of mass (sine wave translation offsets on Torso joint).
- `walk`: Heavy, high-inertia bipedal stride (simulated foot-planting, body roll, and weight transfer).
- `deploy`: Visor startup sequence (glow transition on emissive channel), unfolding wing arrays, and weapon rails activation.
- The digital twin walkthrough transitions (`mud:RouteNode`) must map active locomotion cycles to states (e.g., Entrance plays `idle`, walking path plays `walk`).

---

### C. Weapon Loadouts (Milestone 6)
To support dynamic equipment swapping without naming or structural collisions:
- Define socket individuals (e.g., `mud:socket_hand_left`, `mud:socket_wing_mount_right`) in the ontology.
- Weapons (e.g., `mud:blade_left`, `mud:shield_generator`) must declare their attachment target sockets.
- The root assembly `ASSET_ReferenceFabric_001.usda` queries the active loadout configuration and references the weapon USDs inside the corresponding socket transforms.

---

## 4. Proposed Schema & SPARQL Query Extensions

### A. Turtle Ontology Extensions
The following triples must be appended to the Turtle ontology (e.g., `ontology/all_merged.ttl`) to represent VFX sockets, destruction states, animations, and loadouts:

```turtle
# ---------------------------------------------------------
# VFX & Weapon Sockets Schema
# ---------------------------------------------------------
mud:SocketType rdf:type owl:Class ;
    rdfs:label "Socket Type" .

mud:VFXSocket rdf:type owl:Class ;
    rdfs:subClassOf core:Entity ;
    rdfs:label "VFX Socket" .

mud:socket_sparks rdf:type mud:SocketType ; rdfs:label "sparks" .
mud:socket_smoke rdf:type mud:SocketType ; rdfs:label "smoke" .
mud:socket_debris rdf:type mud:SocketType ; rdfs:label "debris" .

mud:socket_sparks_torso rdf:type mud:VFXSocket ;
    mud:belongsToPart mud:torso_core ;
    mud:socketType mud:socket_sparks ;
    mud:translateX "-5.0000"^^xsd:float ;
    mud:translateY "3.0000"^^xsd:float ;
    mud:translateZ "115.0000"^^xsd:float .

# ---------------------------------------------------------
# Destruction State Metamodel
# ---------------------------------------------------------
mud:DestructionState rdf:type owl:Class ;
    rdfs:label "Destruction State" .

mud:intact_state rdf:type mud:DestructionState ; rdfs:label "intact" .
mud:damaged_state rdf:type mud:DestructionState ; rdfs:label "damaged" .

mud:destructionState rdf:type owl:ObjectProperty ;
    rdfs:domain mud:GeometryPrimitive ;
    rdfs:range mud:DestructionState ;
    rdfs:label "Destruction State Binding" .

# Map existing primitives to intact, and new exposed frame/broken plates to damaged
mud:prim_0001 mud:destructionState mud:intact_state .

# Add exposed mechanical frame primitive
mud:prim_exposed_frame_01 rdf:type mud:GeometryPrimitive ;
    mud:belongsToPart mud:torso_core ;
    mud:primitiveFamily "cylinder" ;
    mud:destructionState mud:damaged_state ;
    mud:translateX "0.0000"^^xsd:float ;
    mud:translateY "1.0000"^^xsd:float ;
    mud:translateZ "115.0000"^^xsd:float ;
    mud:scaleX "2.0000"^^xsd:float ;
    mud:scaleY "2.0000"^^xsd:float ;
    mud:scaleZ "10.0000"^^xsd:float ;
    mud:rotateX "15.0000"^^xsd:float ;
    mud:rotateY "0.0000"^^xsd:float ;
    mud:rotateZ "0.0000"^^xsd:float ;
    mud:materialBinding mud:M_DarkFrame .

# ---------------------------------------------------------
# Locomotion Animation Metamodel
# ---------------------------------------------------------
mud:LocomotionAnimation rdf:type owl:Class ;
    rdfs:subClassOf core:Entity ;
    rdfs:label "Locomotion Animation" .

mud:anim_idle rdf:type mud:LocomotionAnimation ;
    mud:animName "idle" ;
    mud:duration "2.0"^^xsd:float ;
    mud:rootMotion false .

mud:anim_walk rdf:type mud:LocomotionAnimation ;
    mud:animName "walk" ;
    mud:duration "1.6"^^xsd:float ;
    mud:rootMotion true .

mud:anim_deploy rdf:type mud:LocomotionAnimation ;
    mud:animName "deploy" ;
    mud:duration "4.0"^^xsd:float ;
    mud:rootMotion false .

# ---------------------------------------------------------
# Weapon Loadouts Metamodel
# ---------------------------------------------------------
mud:WeaponLoadout rdf:type owl:Class ;
    rdfs:subClassOf core:Entity ;
    rdfs:label "Weapon Loadout" .

mud:LoadoutVariant rdf:type owl:Class ;
    rdfs:label "Loadout Variant" .

mud:loadout_default rdf:type mud:WeaponLoadout ;
    mud:activeLoadout mud:loadout_dual_blades .

mud:loadout_dual_blades rdf:type mud:LoadoutVariant ;
    mud:leftHandMount mud:blade_left ;
    mud:rightHandMount mud:blade_right .
```

### B. Enhanced SPARQL Query (`usd_prims.rq`)
To support the dynamic extraction of intact vs. damaged geometry primitives and VFX sockets, the SPARQL query must extract the destruction state and socket properties:

```sparql
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX mud: <https://rocket-craft.com/ontology/mud#>

SELECT ?prim ?part ?type ?translateX ?translateY ?translateZ ?scaleX ?scaleY ?scaleZ ?rotateX ?rotateY ?rotateZ ?materialBinding ?destructionStateLocal ?socketTypeLocal ?partLocalName ?materialLocalName ?primLocalName
WHERE {
  {
    ?prim rdf:type mud:GeometryPrimitive ;
          mud:belongsToPart ?part ;
          mud:primitiveFamily ?type ;
          mud:translateX ?translateX ;
          mud:translateY ?translateY ;
          mud:translateZ ?translateZ ;
          mud:scaleX ?scaleX ;
          mud:scaleY ?scaleY ;
          mud:scaleZ ?scaleZ ;
          mud:rotateX ?rotateX ;
          mud:rotateY ?rotateY ;
          mud:rotateZ ?rotateZ ;
          mud:materialBinding ?materialBinding .
    OPTIONAL { 
      ?prim mud:destructionState ?destState .
      BIND(STRAFTER(STR(?destState), "#") AS ?destructionStateLocal)
    }
  }
  UNION
  {
    ?prim rdf:type mud:VFXSocket ;
          mud:belongsToPart ?part ;
          mud:socketType ?socketType ;
          mud:translateX ?translateX ;
          mud:translateY ?translateY ;
          mud:translateZ ?translateZ .
    BIND("socket" AS ?type)
    BIND("M_DarkFrame" AS ?materialBinding) # Socket proxy material representation
    BIND("0.1"^^xsd:float AS ?scaleX)
    BIND("0.1"^^xsd:float AS ?scaleY)
    BIND("0.1"^^xsd:float AS ?scaleZ)
    BIND("0.0"^^xsd:float AS ?rotateX)
    BIND("0.0"^^xsd:float AS ?rotateY)
    BIND("0.0"^^xsd:float AS ?rotateZ)
    BIND(STRAFTER(STR(?socketType), "#") AS ?socketTypeLocal)
  }
  
  BIND(STRAFTER(STR(?part), "#") AS ?partLocalName)
  BIND(STRAFTER(STR(?materialBinding), "#") AS ?materialLocalName)
  BIND(STRAFTER(STR(?prim), "#") AS ?primLocalName)
}
ORDER BY ?prim
```

---

## 5. Template (Tera) Enhancements

To implement destruction states in the OpenUSD meshes, `part_mesh.usda.tera` must be modified to output variant structures. 

```jinja
#usda 1.0
(
    defaultPrim = "SM_Torso"
    upAxis = "Y"
    metersPerUnit = 0.01
)

{% macro render_mesh_primitive(row) %}
        def Mesh "{{ row.primLocalName }}"
        {
            {% if row.type == "tapered_box" %}
            int[] faceVertexCounts = [4, 4, 4, 4, 4, 4]
            int[] faceVertexIndices = [0, 3, 2, 1, 4, 5, 6, 7, 0, 1, 5, 4, 1, 2, 6, 5, 2, 3, 7, 6, 3, 0, 4, 7]
            point3f[] points = [(-0.5, -0.5, -0.5), (0.5, -0.5, -0.5), (0.5, 0.5, -0.5), (-0.5, 0.5, -0.5), (-0.35, -0.35, 0.5), (0.35, -0.35, 0.5), (0.35, 0.35, 0.5), (-0.35, 0.35, 0.5)]
            {% elif row.type == "feather_panel" %}
            int[] faceVertexCounts = [4, 4, 4, 4, 4, 4]
            int[] faceVertexIndices = [0, 3, 2, 1, 4, 5, 6, 7, 0, 1, 5, 4, 1, 2, 6, 5, 2, 3, 7, 6, 3, 0, 4, 7]
            point3f[] points = [(-0.5, -0.05, -0.5), (0.5, -0.05, -0.5), (0.5, 0.05, -0.5), (-0.5, 0.05, -0.5), (-0.1, -0.01, 0.5), (0.1, -0.01, 0.5), (0.1, 0.01, 0.5), (-0.1, 0.01, 0.5)]
            {% elif row.type == "blade_prism" %}
            int[] faceVertexCounts = [3, 3, 4, 4, 4]
            int[] faceVertexIndices = [0, 2, 1, 3, 4, 5, 0, 1, 4, 3, 1, 2, 5, 4, 2, 0, 3, 5]
            point3f[] points = [(-0.5, -0.5, -0.5), (0.5, -0.5, -0.5), (0.0, 0.5, -0.5), (-0.5, -0.5, 0.5), (0.5, -0.5, 0.5), (0.0, 0.5, 0.5)]
            {% else %} {# cylinder #}
            int[] faceVertexCounts = [8, 8, 4, 4, 4, 4, 4, 4, 4, 4]
            int[] faceVertexIndices = [7, 6, 5, 4, 3, 2, 1, 0, 8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 9, 8, 1, 2, 10, 9, 2, 3, 11, 10, 3, 4, 12, 11, 4, 5, 13, 12, 5, 6, 14, 13, 6, 7, 15, 14, 7, 0, 8, 15]
            point3f[] points = [(1.0, 0.0, -0.5), (0.707, 0.707, -0.5), (0.0, 1.0, -0.5), (-0.707, 0.707, -0.5), (-1.0, 0.0, -0.5), (-0.707, -0.707, -0.5), (0.0, -1.0, -0.5), (0.707, -0.707, -0.5), (1.0, 0.0, 0.5), (0.707, 0.707, 0.5), (0.0, 1.0, 0.5), (-0.707, 0.707, 0.5), (-1.0, 0.0, 0.5), (-0.707, -0.707, 0.5), (0.0, -1.0, 0.5), (0.707, -0.707, 0.5)]
            {% endif %}
            
            double3 xformOp:translate = ({{ row.translateX }}, {{ row.translateY }}, {{ row.translateZ }})
            double3 xformOp:scale = ({{ row.scaleX }}, {{ row.scaleY }}, {{ row.scaleZ }})
            double3 xformOp:rotateXYZ = ({{ row.rotateX }}, {{ row.rotateY }}, {{ row.rotateZ }})
            uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:rotateXYZ", "xformOp:scale"]
            
            rel material:binding = </ASSET_ReferenceFabric_001/Materials/{{ row.materialLocalName }}>
        }
{% endmacro %}

{% macro render_socket_primitive(row) %}
        def Xform "socket_{{ row.socketTypeLocal }}_{{ row.primLocalName }}"
        {
            double3 xformOp:translate = ({{ row.translateX }}, {{ row.translateY }}, {{ row.translateZ }})
            uniform token[] xformOpOrder = ["xformOp:translate"]
        }
{% endmacro %}

def Xform "SM_Torso" (
    variants = {
        string destruction = "intact"
    }
    prepend variantSets = "destruction"
)
{
    # Render static VFX and weapon attachment sockets outside of variant sets
    {% for row in results %}
        {% if row.type == "socket" %}
            {{ self::render_socket_primitive(row=row) }}
        {% endif %}
    {% endfor %}

    variantSet "destruction" = {
        "intact" {
            # Standard armor shells and frames
            {% for row in results %}
                {% if row.type != "socket" and (row.destructionStateLocal == "intact" or not row.destructionStateLocal) %}
                    {{ self::render_mesh_primitive(row=row) }}
                {% endif %}
            {% endfor %}
        }
        "damaged" {
            # Fractured plates, exposed chassis structure, and spark sockets
            {% for row in results %}
                {% if row.type != "socket" and (row.destructionStateLocal == "damaged" or not row.destructionStateLocal) %}
                    {{ self::render_mesh_primitive(row=row) }}
                {% endif %}
            {% endfor %}
        }
    }
}
```

---

## 6. Verification Plan

To verify this implementation strategy independently without modifying the core codebase:
1. **Regenerate Assets**: Execute `/Users/sac/.local/bin/ggen sync` to verify that the extended SPARQL queries and templates compile without syntax errors.
2. **Execute Offline Test Suite**: Run `npx vitest run mecha_offline.test.ts` inside `pwa-staff/` to confirm that all Tiers 1-3 constraints (including morphology, PBR channels, and rigging hierarchies) are validated.
3. **Execute E2E Presentation Pipeline**: Run `./verify_mecha_pipeline.sh` to compile the WASM artifact, stage files, start the local server on port 8080, execute Playwright input actuations, record movement delta (ensuring threshold is met), validate the BLAKE3 receipt chain, and verify the AI Vision Judge report has a `disposition: PASS_FLAGSHIP` with zero critical defects.
