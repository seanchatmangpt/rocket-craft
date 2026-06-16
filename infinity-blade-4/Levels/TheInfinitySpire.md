# Level: The Infinity Spire

**Act:** III | **Bloodline Unlock:** 10 | **Enemy:** GoldTitan → CorruptedGalath (Final Boss)**

---

## Overview

A kilometer-tall structure of living Deathless alloy — partially grown, partially built — that Galath designed as his true home, his research fortress, and his rebirth anchor point. It has been dormant for three centuries but is now waking, responding to Aran's approach. The Spire recognizes him.

This is the endgame arena set. Three floors, escalating, leading to the apex chamber where Corrupted Galath must be fought.

---

## Layout

```
[BASE GATE] → [LOWER SPIRE: The Living Walls]
                     ↓
              [MID-SPIRE: The Memory Galleries]
                     ↓
              [UPPER SPIRE: The Protocol Engine]
                     ↓
              [APEX: The Glass Throne] ← Final boss arena
```

**Base Gate:** The Architect makes first physical appearance here. Does not fight. Watches. Speaks one sentence: "He has been waiting." Then steps aside.

**Lower Spire (Living Walls):** Three GoldTitan encounters on moving platform lifts. Platforms shift position based on QIP field pulsations — spatial navigation while fighting. Miss-timed dodge can put you on the wrong platform.

**Mid-Spire (Memory Galleries):** Linear hall. No combat. Holograms of Galath's memories play on loop — his original form, his early Deathless colleagues, a younger Raidriar who was his friend before everything. Aran walks silently. On Bloodline 15+, he says: "I remember the light in that hall."

**Upper Spire (Protocol Engine):** Two QuantumTitans simultaneously. First true 2v1 encounter. The Protocol Engine counts down in the background — after 90 seconds it emits an AoE pulse (3 in Phase 1, 6 in Phase 2) that damages the player. Encourages aggressive combat pace.

**Apex (Glass Throne):** Circular chamber, transparent floor revealing the entire world far below. The Corrupted Galath fight occurs here.

---

## Corrupted Galath Fight (IB4GodKingAI)

Full three-phase encounter per `IB4GodKingAI.h/.cpp`:

### Phase 1: The Shield
- Hard-light shield absorbs ALL damage including magic
- Only perfect parries register — 3 perfect parries break the shield
- Galath's voice: the child's voice, confused, overlaid with ancient resonance
- `bShieldActive = true`; `PerfectParriesReceived` counter tracked

### Phase 2: Dual Blades
- Shield breaks; Galath draws two blades (one sword, one infinity weapon)
- `ApplyQIPScar()` each hit — 3 stacks triggers forced Rebirth
  - **Critical design:** the Rebirth is non-lethal here (QIP-scar mechanism) — Siris "dies" but returns with no bloodline credit, enemy HP preserved
  - This is the only encounter in the game that can force Rebirth without death
- Galath's dialogue: "I don't want to do this. I don't know how to stop."
- DamageMultiplier 1.25× (Phase 2 modifier from IB4TitanAI base)

### Phase 3: Reality Fracture
- Random `SetGlobalTimeDilation(0.7–1.3×)` every 4 seconds
  - 0.7× = slow motion; player attacks cost more time but so does Galath
  - 1.3× = fast motion; combo windows tighter, input precision required
- Spawns 2 QuantumTitan reinforcements via NavMesh random spawn
- Galath's attack pattern: no longer tactical, pure kinetic release
- Galath's voice: no longer overlaid — only the ancient resonance, Galath-as-was

### Post-Fight Triggers

**True Ending (Bloodline 20 + all Titan Seals):** Protocol disruption cutscene. Aran/Galath exists as hybrid. Siris sheathes his weapon. Silence.

**Sacrifice Ending (Bloodline 15–19):** Siris takes the killing blow. Falls. Aran/Galath feels the QIP bond break. Partial overwrite rollback. Ambiguous end card.

**Bloodline Loop Ending (BL 0–14 or Negative):** Galath fully wakes. The Spire seals. Credits roll over Siris waking up in Lantimor at the beginning of the game.

---

## Enemy Encounters

| Encounter | Area | Notes |
|---|---|---|
| GoldTitan ×3 | Lower Spire | Platform lifts — spatial nav |
| QuantumTitan ×2 | Upper Spire | First 2v1; Protocol Engine pressure |
| CorruptedGalath | Apex | Final boss; three phases |

---

## Lore Beats

- **Base Gate / Architect:** "He built this place to die in. He was more afraid than any of us." First and only sympathetic Architect line.
- **Memory Galleries:** Galath's memory of inventing the QIP immortality process — a young man, terrified of his own death, building the cage that would trap his whole civilization.
- **Apex entrance:** Aran turns to Siris: "I left you a way out. I built it into the foundation. I wanted you to survive." He doesn't know he's quoting a memory.
- **Phase 2 Galath:** "I remember you. Ausar. You were there when I finished it. You told me it was wrong." Siris: "And I was right."

---

## Design Notes

- Glass Throne floor shatters progressively through Phase 3 — no fall-through, but visual fracturing matches time dilation randomness (world feeling unstable)
- Time dilation shifts are announced with a brief prismatic flash (visual cue, not just mechanical)
- The Protocol Engine countdown in Upper Spire is designed to end precisely at the Corrupted Galath transition — narrative and mechanical pressure converge
- On Bloodline 20 run, Memory Galleries show Siris's face among Galath's memories — he was always part of the plan
