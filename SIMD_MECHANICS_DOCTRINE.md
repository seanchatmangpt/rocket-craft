# SIMD Mechanics Doctrine

The performance jump is not merely:

```text
Blueprint slow → SIMD fast
```

It is:

```text
local object scripting → global semantic field computation
```

That changes the **class of mechanics** available.

The old UE4 Blueprint / Actor / Tick model is good for:
- a few hundred rich objects
- hand-authored interactions
- local triggers
- scripted events
- designer-authored behavior

The SIMD / ggen / byte-authority model is good for:
- millions of small semantic states
- global scans
- field propagation
- mass classification
- bulk promotion/demotion
- deterministic replay
- server authority at world scale

So the new mechanic frontier is:
**Mechanics where “everything small matters,” but nothing small is allowed to become a heavyweight Actor.**

---

## 1. The Core Unlock: World-Scale Semantic Fields

Previous method:
```text
Actor owns state.
Blueprint checks state.
Event mutates local object.
```

SIMD method:
```text
World owns dense byte fields.
ggen generates transition laws.
SIMD updates thousands/millions of states per frame.
UE4 projects the result.
```

This enables a new class of mechanics:
**The world is not a collection of objects. The world is a living semantic field.**

Instead of asking: "What does this Actor do?"
You ask:
- What is the current state of the whole damage field?
- What is the current heat field?
- What is the current grip field?
- What is the current stress field?
- What regions changed?
- What needs projection?
- What deserves authority?

Blueprint can fake this locally. SIMD can run it globally.

---

## 2. Mechanics Now Possible

### A. World-Scale Damage Fields
Every panel, socket, joint, surface, tile, track segment, mech limb, and facility cell can carry byte-class damage. 
Damage is not an event. Damage is a field.

### B. Heat, Stress, and Fatigue Propagation
Heat propagates through sockets, armor, engine cells, weapon mounts, track surface, facility machinery, and environmental zones.
You do not manage a health bar. You manage a machine under thermal and structural law.

### C. Track Grip as a Living Surface
Every segment of the track has grip_class, heat_class, rubber_class, debris_class, moisture_class, damage_class. 
The server owns the grip field. The client projects it.

### D. Semantic Destruction Without Full Physics Simulation
Not every bolt simulates physically, but every important cell can change state. 
Destruction is not a spectacle. Destruction is receipted state transition.

### E. Dirty-Mask Visuals
Maintain massive dirty masks for damage, oil, mud, burn, corrosion, dust, rubber, bloodless impact marks, heat discoloration, and repair residue. 
The world remembers contact.

### F. Always-On Diagnostic Vision
Every authority byte can be projected into diagnostic overlays at any time. 
thermal vision, stress vision, market-flow vision, socket-health vision, etc.

### G. Massive Affordance Maps
Global affordance maps: cover_class, visibility_class, danger_class, repairability_class, loot_value_class.
The world became queryable at scale.

### H. Semantic Crowd / Swarm Behavior
Crowd cells carry byte states. Individuals project from fields.

### I. Market as a Physical Field
Market conditions become world state. Byte classes for scarcity, demand, risk, liquidity.
The market becomes spatial and playable.

### J. Real-Time Semantic LOD Promotion/Demotion
LOD is continuously recomputed from semantic relevance. The world focuses itself.

---

## 3. Visual Mechanics Enabled by SIMD

SIMD does not replace the GPU. SIMD feeds meaning to the GPU.

- **Semantic Heat Bloom:** heat_class field → shimmer mask → emissive material
- **Stress Fracture Visualization:** stress_class field → crack masks → vibration
- **Dirt, Wear, and Use History:** contact history → grime accumulation
- **Damage-Readable Silhouettes:** Semantic LOD preserves important silhouette changes.
- **Massive Low-Cost Micro-Animation:** Field drives many projections cheaply.

---

## 4. Game Mechanics That Become New Genres

- **Mech Engineer Racing:** The car/mech is a living machine.
- **Factory-as-Dungeon:** Walk the line, find bottlenecks, repair machines, stop line through Agent Jidoka.
- **Semantic Sabotage:** Player attacks semantic dependencies instead of health bars.
- **Replayable Forensics Gameplay:** Gameplay around diagnosis and auditing.
- **Living World Maintenance:** Inspect, diagnose, repair, verify, receipt, replay, certify, return to service.

---

## 5. The Most Important New Mechanic: Authority Compression

You can have 1,000,000 cells each with 4–8 bytes of meaningful authority instead of 10,000 Actors each with huge object overhead.
This enables a world where:
**Everything can matter a little, while only important things matter a lot.**

---

## Final Law

SIMD does not merely accelerate mechanics.
SIMD admits a new mechanic class: **world-scale, byte-authority, semantically projected, replayable state fields.**

Previous method: Actors with scripts.
New method: **Fields with law.**

---

## 6. The Render Bridge: Math of Geometry Sync

The safe answer to "Is there enough time to sync geometry?":
SIMD can update millions of authority states per frame. But you must not push millions of geometry edits into Unreal every frame.

You push small projection deltas:
- material masks
- instance transforms
- LOD class changes
- mesh variant IDs
- dirty chunks
- animation parameters
- DataTable/state-buffer changes

The rule is:
**SIMD owns semantic state. UE4 owns geometry projection. The bridge sends deltas, not the world.**

### A. The Basic 3D Math
Do not stream raw geometry (vertices) every frame unless the mesh count is tiny. 1,000,000 vertices/frame ≈ 1.9–3.8 GB/sec. That destroys the frame budget before draw-call pressure even hits.

### B. The Core Bridge Pattern
The bridge should look like this:
```text
Authority Arrays
    ↓
SIMD Transition Kernels
    ↓
Dirty Bitset
    ↓
Chunk Aggregator
    ↓
Projection Commands
    ↓
UE4 Render Surfaces
```

Where projection commands are things like `SetMaterialMask(chunk_id, mask_id)`, not `UpdateEveryActorEveryFrame()`.

### C. Example: Track Grip Field
Track has 1,000,000 cells (4 MB of authority state). SIMD updates all 1,000,000 cells.
Divide track into chunks (e.g., 64x64 cells). Render bridge asks: "Which chunks changed enough to matter visually?"
Maybe only 12 chunks changed. UE4 receives 12 material mask updates, not 1,000,000 object updates.

### D. The Hard Boundary: Collision
Do not update collision for every semantic state change. Only update collision at threshold crossings (e.g., damage 6 = collision profile change, damage 0-5 = visual only).

### E. The Design Law
**Never sync geometry because state changed. Sync projection only when semantic change becomes visible, interactive, or authoritative.**
