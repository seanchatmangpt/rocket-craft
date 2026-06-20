# SPR: TPS/DfLSS 100% PROCESS CAPABILITY

## Definition of 100%
100% is not infinite aesthetic quality. 100% means:
The process is capable of producing assets that meet the declared AAA UE4 specification, repeatably, without manual intervention, under replay.

`AAA_UE4_100 = CTQ_complete ∧ measurement_valid ∧ generator_capable ∧ verifier_hard_gated ∧ UE4_consumes ∧ replay_passes ∧ defect_escape_rate_bounded`

## TPS Value Stream (The Cells)
Design law → candidate enumeration → geometry cell → hard-surface detail cell → material/texture cell → rig/socket cell → collision/LOD cell → UE4 import/cook cell → visual/gameplay/IP verifier cell → receipt/replay cell.
No cell passes bad work downstream.

## DfLSS DMADV Mandate
- **Define**: What must be true for an asset to be called a AAA UE4 mech?
- **Measure**: Build measurement before generation. The current failure happened because the measurement system was weak. Produce `measurement_system_analysis` (MSA) for morphology, material, UE4 import, rig, and IP distance.
- **Analyze**: Identify parameters creating AAA outcomes vs scrap.
- **Design**: Design source law + candidate generation + verifier gates + repair actions.
- **Verify**: Delete outputs, regenerate, import into UE4, cook, screenshot, compare, receipt, repeat.

## The CTQ Tree
- **CTQ-1 (UE4-ready asset)**: UE4 imports it, cooks it, loads it. Materials, textures, skeleton, collision, LODs, sockets bind.
- **CTQ-2 (Geometry)**: Modular part identity, valid topology, no duplicates, beveled armor, panel density, silhouette class.
- **CTQ-3 (Material/Texture)**: PBR channels (BaseColor, Normal, Roughness, Metallic, AO, Emissive, Decal/Wear masks).
- **CTQ-4 (Rig/Sockets)**: Skeleton, joint limits, IK targets, weapon/support sockets, damage zones.
- **CTQ-5 (IP Distance)**: Outside expressive clusters, outside trade-dress confusion.
- **CTQ-6 (Replay)**: Delete outputs → regenerate identical receipts. Same seed → same asset.
- **CTQ-7 (Throughput)**: High N candidates, stable M admitted, defect escape rate trending down.

## The Control Plan & Andon Triggers
Stop the line if: part files duplicate, visual metric admits known bad fixture, UE4 import fails, PBR stack incomplete, rig sockets missing, IP report missing, receipt chain incomplete, replay differs.

## The Crown Verifier Command
The swarm must implement the following automation:
`just verify-aaa-ue4-mech-pack-001`
Execution:
1. delete generated outputs
2. regenerate from source law
3. emit USD/FBX/MaterialX/textures/rig/collision/LOD
4. run headless renders
5. run part-aware morphology
6. run PBR validation
7. run UE4 command-line import
8. run UE4 cook
9. run UE4 screenshot
10. run IP-distance
11. emit OCEL
12. emit receipts
13. replay hashes
14. compare diagnostics
15. exit nonzero on any failure
