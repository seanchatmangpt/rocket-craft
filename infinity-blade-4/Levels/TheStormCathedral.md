# Level: The Storm Cathedral

**Act:** II | **Bloodline Unlock:** 3 | **Enemy:** StormMageTitan → Jara (Pantheon Boss)

---

## Overview

A cathedral built inside a permanent electrical storm — a decommissioned Deathless weather-control installation that Galath once used to synthesize lightning-QIP weapons. The Pantheon faction loyal to Thane has reactivated it. Jara, the Titan-class warrior, commands the cathedral from the central nave.

This is the first major story boss fight. Jara reveals the existence of the Architect and that Aran has been leaving trail markers for himself.

---

## Layout

```
[LANDING PLATFORM] → [OUTER NAVE] → [SIDE CHAPELS ×2] → [CENTRAL NAVE] → [BELL TOWER]
                           ↓
                   [UNDERCROFT: QIP Reactor]
```

**Landing Platform:** Narrow ledge in open storm. Lightning strikes every 12s in random positions (avoidable with dodge timing, not guaranteed). Sets tone.

**Outer Nave:** Two-lane approach with enemy archers on elevated rails. Teaches upward swipe to deflect projectiles (overhead strike sends magic bolts back).

**Side Chapels:** Optional — each contains one rare equipment chest. Left chapel: sword fragment. Right chapel: shield fragment. Neither forces engagement.

**Central Nave:** Main boss arena, 40m × 25m rectangular. Broken stained-glass floor — falling through gaps at Phase 3 is instant death (adds spatial awareness requirement).

**Bell Tower:** Reached after Jara's defeat. Contains the Architect's first direct message to Aran — a QIP data-crystal that Aran absorbs involuntarily. Triggers first "episode" cutscene.

**Undercroft:** Optional secret area. Contains `LightningOrb` — a Tier 2 magic unlock if found before Bloodline 3 XP gates it naturally.

---

## Enemy Encounters

| Encounter | Enemy | Notes |
|---|---|---|
| 1 | StormMageTitans ×3 | Outer nave; teaches lightning dodge pattern |
| 2 | GuardTitan ×2 | Side chapel gates; optional |
| 3 | Jara (Boss) | Central nave; three-phase Pantheon warrior |

---

## Jara Fight Design

**Phase 1 (100–60% HP):** Two-hand greatblade. High-damage overhead strikes with long telegraphs. Perfect parry reflects blade-shock wave back (deals 30% of swing damage). Parry-bait attacks start here.

**Phase 2 (60–30% HP):** Activates QIP-infused armor (blocks magic entirely). Player must strip armor via 5 consecutive normal parries. This is the `bCanBreakParry` mechanic from IB4TitanAI at Bloodline 3.

**Phase 3 (30–0% HP):** Armor shattered. Jara fights in berserker mode — attack speed ×2, breaks combo window if player is not at max combo depth. Lightning strikes in Phase 3 actively target the player's position (no longer random).

---

## Lore Beats

- **Entrance:** Thane's hologram: "The child carries all of it. Every piece of the Protocol. You cannot protect him from himself."
- **Jara (pre-fight):** "I respected Raidriar. You are a memorial to a mistake he made."
- **Post-Jara:** Aran in Bell Tower, hands on data-crystal: "I know this language. I wrote it." His eyes flash silver for 2 seconds. He doesn't remember it after.
- **Undercroft:** Saydhi's field log: "Subject re-encodes QIP patterns with zero latency. He should not be able to do this without neural interface hardware. He has no hardware."

---

## Design Notes

- Storm audio is reactive: thunder intensifies when combo multiplier reaches 3×
- Lightning strikes in Phase 3 have a 0.8s shadow before impact (fair warning, punishing if ignored)
- Stained glass floor gaps widen each time Jara's Phase transitions — player must adapt spatial movement
- On Bloodline 5+, Jara is absent (already defeated canonically per bloodline memory); replaced by two SilverTitans, but Bell Tower scene still plays
