# Infinity Blade IV: The Waking God — Game Design Document

**Version 1.0 | Engine: Unreal Engine 4.24 | Platforms: iOS, Android, Windows**

---

## 1. Vision Statement

Infinity Blade IV is a gesture-driven action RPG set in the far future of the Deathless civilization. You play as Siris, aging knight of the QIP bloodline, tasked with stopping the resurrection of Galath — the Worker of Secrets — whose consciousness has re-emerged inside a child's body with catastrophically unstable Quantum Immortality Potential. Galath's memories have fractured and corrupted over centuries of dormancy. He does not remember being a monster. That is what makes him the most dangerous being alive.

The game is the spiritual and narrative culmination of the trilogy: the bloodline mechanic introduced in IB1 reaches its logical extreme, the lore seeded across three games and the novel pays off, and the gesture combat is refined to its purest form.

---

## 2. Core Pillars

| Pillar | Expression |
|---|---|
| **Elegance** | Every mechanic has one clear decision point. Combat is readable in one second. |
| **Consequence** | Death is not failure — it is progression. Bloodline rewards the cycle. |
| **Mystery** | Galath does not know he is the villain. Neither do you, at first. |
| **Mastery** | Perfect parry timing, combo chains, and perk builds create a skill ceiling worthy of years. |

---

## 3. Narrative Overview

### 3.1 Setting

Seven years after IB3. The God King Raidriar sacrificed himself to seal the Worker of Secrets' essence into a dormant state, destroying himself permanently in the process (no QIP continuity — he chose annihilation). Without its architect, the Deathless civilization has fragmented into rival Pantheon factions. Siris and Isa are guardians of a hidden sanctuary in the ruins of Old Lantimor.

### 3.2 Galath

The child they found in the wreckage was seven years old with white-silver hair and no memory of his true name. Siris named him "Aran." Over seven years, Aran has shown extraordinary gifts: he masters weapon forms in hours, perceives QIP fields that adults cannot sense, and speaks in his sleep in ancient Deathless proto-language. The Pantheon wants him. They call him "the Seed."

At Bloodline 10, the player witnesses Aran's first "episode" — a full activation of his Worker of Secrets subroutines. His eyes go silver. He redesigns an entire fortress's QIP lattice with his bare hands before collapsing. He does not remember it.

At Bloodline 15, Galath's memories begin bleeding through. Aran starts finding his own old notes — schematics he left across the world for himself to find if reborn. He follows them compulsively, leading him to the Infinity Spire.

At Bloodline 20, Corrupted Galath — the final boss — is Aran when he reads the full Protocol: a self-authored recursive QIP overwrite that restores his ancient consciousness and overwrites the child entirely. Siris must fight his adopted son.

### 3.3 The Pantheon

Five surviving Deathless, each with competing agendas:

- **Ausar** — Siris's true Deathless identity, fragments of which surface as visions at rebirth thresholds. Wants to prevent Galath from waking.
- **Thane** — believes Galath can be controlled, wants to harness the Worker of Secrets' power for a new Deathless order.
- **Saydhi** — neutral scholar, provides exposition and equipment; secretly cataloguing Galath's awakening for unknown purposes.
- **Jara** — Titan-class warrior aligned with Thane; primary antagonist mid-game boss.
- **The Architect** — unseen until the final act; was Galath's second-in-command before IB1. Has been orchestrating events for centuries.

### 3.4 Ending Branches

**True Ending (Bloodline 20, all Titan Seals collected):** Siris disrupts the Protocol mid-execution. Galath/Aran exists as a hybrid — neither fully Galath nor fully the child. He is left as the last, uncertain Deathless: mortal enough to age, powerful enough to reshape the world. Siris walks away. Isa stays.

**Sacrifice Ending (Bloodline 15–19):** Siris cannot stop the Protocol. He dies in the fight. Aran/Galath, feeling Siris's death for the first time through their QIP bond, partially suppresses the overwrite. The game ends on ambiguity.

**Bloodline Loop Ending (Negative Bloodline):** The player has died 20+ times without completing the game. Galath wakes fully. The world resets. The opening cutscene plays again — but Siris's face has changed.

---

## 4. Combat Design

### 4.1 Gesture Input

Touch-to-intent mapping on a 50px minimum threshold:

| Gesture | Action |
|---|---|
| Swipe Right | Right Strike |
| Swipe Left | Left Strike |
| Swipe Up | Overhead Strike |
| Short Tap | Dodge Roll |
| Hold + Release | Magic Cast |
| Two-finger tap | Parry Stance |

All gestures feed into `AIB4PlayerController::DetectSwipeDirection()` using axis-dominance: `|deltaX| > |deltaY|` → horizontal; otherwise vertical.

### 4.2 Parry Windows

| Parry Type | Timing Window | Effect |
|---|---|---|
| Miss | Outside 200ms | Full damage received |
| Normal Parry | 0–200ms | Block damage, small stagger |
| Perfect Parry | 0–50ms | No damage, full stagger, time dilation 0.2× for 1.5s |
| Clash | Same-frame strike | Both stagger, no damage |

Perfect parry triggers `UGameplayStatics::SetGlobalTimeDilation(World, 0.2f)`.

### 4.3 Combo Chains

Combo depth drives damage multipliers:

| Depth | Multiplier |
|---|---|
| 1 | 1.0× |
| 2 | 1.5× |
| 3 | 2.0× |
| 4+ | 3.0× |

Combo resets after 2.0s of inactivity via `FTimerHandle`. The perk `ComboMaster` extends the reset window to 3.5s.

### 4.4 Magic System

| Type | Behavior |
|---|---|
| Fire | AoE burst on impact, applies Burn (5 DPS × 3s) |
| Lightning | High velocity, chains to nearby enemies, applies Stun (1.2s) |
| Ice | Homing, applies Freeze (immobilize 2s) |
| Dark | Arcing travel, ignores 50% armor, applies Dark debuff |
| Light | Healing aura, removes status effects from player |

Magic type unlocks are tied to Bloodline progression: Fire (BL0), Lightning (BL3), Ice (BL6), Dark (BL10), Light (BL15).

---

## 5. Progression Systems

### 5.1 Bloodline Rebirth

On player death, `UIB4NewGamePlus::TriggerRebirth()`:
1. Bloodline counter increments
2. Gold and equipment reset
3. XP, level, and unlocked magic types carry over
4. One Bloodline Perk Point granted
5. Enemy scaling: God King Level = 50 × (Bloodline + 1)
6. `MasteryXPMultiplier` doubles each rebirth
7. `FOnRebirth` broadcast to all listeners

At Bloodline 20: `EnterNegativeBloodline()` — no perk points, accelerating enemy scaling.

### 5.2 Bloodline Perk Tree (15 Perks, 3 Tiers)

**Tier 1 (Bloodline 0):** BloodyResolve, IronHide, SwiftStrikes, MagicSensitivity, Scavenger

**Tier 2 (Bloodline 5):** DeadlyPrecision, FortressStance, ComboMaster, ArcaneChanneling, TreasureHunter

**Tier 3 (Bloodline 10):** AusarLegacy, DeathlessResilience, QIPResonance, WorkerOfSecretsGift, InfinitySeeker

Prerequisites enforced by `UIB4BloodlinePerkTree::SelectPerk()`.

### 5.3 Equipment Rarity

Common → Uncommon → Rare → Epic → Legendary → Infinity

Mastery XP thresholds: 500 / 1000 / 2000 / 5000 / 10000 / 25000

Mastered equipment doubles sell value. Infinity rarity items have unique `SpecialMoveName` attacks.

### 5.4 Level Cap and Stats

Level 1–45. `XPForLevel(n) = round(100 × n^1.5)`.

Stat allocation (Attack / Defense / Magic / Health):
- Max Health = 100 + (HealthStat × 60)
- Magic Bonus = MagicStat × 10

---

## 6. Enemy Design

### 6.1 Titan Roster (15 types)

Enemies scale from LightTitan (Bloodline 0, 100 HP) to CorruptedGalath (Bloodline 20, 50,000 HP). See `Data/enemies.csv`.

Phase thresholds:
- Phase 2 activates at 60% HP
- Phase 3 activates at 30% HP

### 6.2 AI Behavior Tree Keys

| Key | Type | Meaning |
|---|---|---|
| `BB_Target` | Object | Current attack target |
| `BB_TargetDistance` | Float | Distance to target (units) |
| `BB_CombatPhase` | Int | 1/2/3 phase state |
| `BB_CanAttack` | Bool | True when distance ≤ 300 |

### 6.3 Corrupted Galath (Final Boss)

Three-phase fight in the Infinity Spire:

- **Phase 1:** Hard-light shield blocks all damage. Only perfect parries (×3) break it. `bShieldActive = true`.
- **Phase 2:** Dual-blade draw. `ApplyQIPScar()` — three stacks forces player Rebirth without death credit.
- **Phase 3:** Reality fracture. Random `SetGlobalTimeDilation(0.7–1.3×)` every 4s. Spawns 2 Titan reinforcements.

---

## 7. Monetization (Premium)

IB4 follows the original Infinity Blade premium model:

- **Base purchase:** $6.99 USD (iOS/Android), $14.99 (PC)
- **No loot boxes.** All equipment obtainable through play.
- **Optional Gold Pack:** Cosmetic weapon skins only (no stat advantage).
- **No energy systems.** No timers. No pay-to-win.

The gem socket system and bloodline perk tree are the depth hooks — players return for optimal builds, not forced friction.

---

## 8. Audio Design

| Context | Direction |
|---|---|
| Exploration | Sparse ambient tones, ancient Deathless harmonic drones |
| Combat | Percussive, escalating tempo synchronized to combo depth |
| Perfect Parry | Micro-silence then single resonant tone |
| Boss Phase Transition | Full orchestra swell, theme shifts key signature down a minor third |
| Galath Awakening | Child's voice layered beneath ancient Deathless speech synthesis |

---

## 9. Target Performance Metrics

| Metric | Target |
|---|---|
| Frame Rate | 60fps locked (iOS A15+, Android Snapdragon 888+) |
| Load Time | < 3s per arena on device |
| Input Latency | < 16ms touch-to-game-event |
| Parry Window Accuracy | ±2ms from declared window |
| Memory Budget | ≤ 1.5GB RAM on device |
