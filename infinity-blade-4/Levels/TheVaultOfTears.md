# Level: The Vault of Tears

**Act:** II | **Bloodline Unlock:** 6 | **Enemy:** FrostTitan → ArchivistTitan

---

## Overview

A subterranean Deathless archive frozen by a cryo-preservation field accident 300 years before IB1. The Vault contains the most complete record of Galath's original research — including the Protocol's intermediate stages. Saydhi has been here for decades, studying. She is not an ally exactly, but she is the only Deathless who will talk to Siris without trying to kill him first.

The Vault is also where Aran loses his first memory — absorbed into a QIP data-pillar — and gains a new one that is not his.

---

## Layout

```
[GLACIER ENTRANCE] → [CRYO-HALL A] → [ARCHIVE ATRIUM] → [SAYDHI'S STUDY]
                                            ↓
                              [DEEP VAULT: The Frozen War]
                                            ↓
                              [CORE CHAMBER: Memory Engine]
```

**Glacier Entrance:** Sloped approach with FrostTitan ambush. Ice floor reduces dodge distance by 30% (physics material). Requires overhead strikes to break ice armor on FrostTitans.

**Cryo-Hall A:** Narrow corridor of preserved bodies in cryo-pods. Intact Deathless soldiers. One of them, pod labeled "A-7", has Aran's face. He is seven years old in the pod. This is the game's first major horror beat.

**Archive Atrium:** Large circular chamber, central data-pillar. During the ArchivistTitan fight, the pillar periodically pulses AoE (dodge-roll timing).

**Saydhi's Study:** No combat. Exposition hub. Saydhi provides: Galath's motivations (not evil, he was afraid of death like all Deathless), the existence of the Memory Engine, and the location of the Infinity Spire.

**Deep Vault:** Optional, Bloodline 6+ only. Contains `DarkOrb` magic unlock and the Fallen Titan — a Legendary equipment source.

**Core Chamber (Memory Engine):** Aran touches the engine. Cutscene: Galath's last waking memory — standing in this vault, recording the Protocol, knowing that the next person to access the Memory Engine would be himself reborn. He left a message: "Don't read the Protocol." Aran has already read it.

---

## Enemy Encounters

| Encounter | Enemy | Notes |
|---|---|---|
| 1 | FrostTitan ×2 | Glacier entrance; ice armor mechanic |
| 2 | GuardTitan ×3 | Cryo-Hall A gate; can lure one away using environment |
| 3 | ArchivistTitan | Archive Atrium; ranged + AoE + pillar-phase mechanic |
| 4 | FallenTitan | Deep Vault (optional); Legendary drop |

---

## ArchivistTitan Fight Design

**Phase 1 (100–60% HP):** Ranged ice-lance volleys in a fan pattern (3 projectiles per volley). Overhead swipe deflects single-target. Cannot be approached while projecting.

**Phase 2 (60–30% HP):** Enters melee range. Uses IB4TitanAI's `bCanBreakParry` — consecutive parries eventually get broken by armor. Player must mix dodge and parry.

**Phase 3 (30–0% HP):** Activates archive data-pillar surges in cardinal directions (AoE lines on floor). Player must read patterns and maintain aggression. Death by pillar surge is instant.

---

## Lore Beats

- **Pod A-7:** Siris: "That's impossible." Saydhi: "The original prototype body. He made it for himself before the war. He was very thorough." Aran does not react. He is looking at his own hands.
- **Saydhi Conversation:** "The Protocol is not a weapon. It is a restoration subroutine. He wasn't building a god. He was trying to save himself from oblivion. He was afraid." — first moment of sympathy for Galath.
- **Memory Engine:** On Bloodline 10+, the message changes: "If you have read the Protocol, find the Infinity Spire. Come home."

---

## Design Notes

- Ice floor is thematic, not cruel — dodge distance reduction is communicated by ice cracks under player feet (visual)
- Cryo-Hall A pod revelation is a walk-past moment, not a cutscene — pacing respect for players on Bloodline 3+
- Saydhi's study has a readable terminal with 12 archive entries (optional lore depth)
- Memory Engine cutscene is skippable on Bloodline 2+, but changes content on Bloodline 10+
