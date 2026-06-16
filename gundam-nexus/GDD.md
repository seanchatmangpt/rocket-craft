# GUNDAM NEXUS — GAME DESIGN DOCUMENT
### Version 1.0 | Classification: Internal Pre-Production | Date: June 2026

---

> *"Every Gundam series' greatest moment, in your hands."*

---

## TABLE OF CONTENTS

1. [Vision Statement](#1-vision-statement)
2. [Core Pillars](#2-core-pillars)
3. [Series Stack Ranking & Mechanic Contributions](#3-series-stack-ranking--mechanic-contributions)
4. [Narrative Overview: The Convergence Era](#4-narrative-overview-the-convergence-era)
5. [Combinatorial Maximalism: Full Mechanic Table](#5-combinatorial-maximalism-full-mechanic-table)
6. [Combat Design — The IB4 Gesture Engine](#6-combat-design--the-ib4-gesture-engine)
7. [Mobile Suit Roster](#7-mobile-suit-roster)
8. [Gunpla Builder System](#8-gunpla-builder-system)
9. [Progression Systems](#9-progression-systems)
10. [Duel Arena (PvP)](#10-duel-arena-pvp)
11. [Single-Player Campaign](#11-single-player-campaign)
12. [Target Demographics & Market](#12-target-demographics--market)
13. [Platform Strategy](#13-platform-strategy)
14. [Audio Direction](#14-audio-direction)
15. [Competitive / Esports Hook](#15-competitive--esports-hook)
16. [Monetization Model](#16-monetization-model)
17. [Technical Specifications](#17-technical-specifications)
18. [Risk Register](#18-risk-register)

---

## 1. VISION STATEMENT

**Gundam Nexus** is a premium action-RPG for mobile, PC, and console that delivers the definitive Gundam gaming experience by unifying the greatest mechanical innovations of every major Gundam series into a single, cohesive game system built on the **Infinity Blade 4 gesture combat engine**.

Where previous Gundam games have been franchise tie-ins or single-series showcases, Gundam Nexus is built from first principles around the question: *what if every series had contributed its single best mechanic to one game?*

The answer is Combinatorial Maximalism — a design philosophy that treats each series not as a setting or a skin, but as a **mechanic donor**. The Witch from Mercury gives us its tournament duel format. SEED gives us modular Striker Pack swapping. Unicorn gives us timed burst transformation. Wing gives us the controlled chaos of the ZERO System. Iron-Blooded Orphans gives us the visceral cost-per-parry of the Alaya-Vijnana system. Build Fighters gives us part-by-part kit construction. Universal Century gives us the Newtype Flash — the perfect parry that resonates across the battlefield.

Every mechanic is tuned to work alongside every other. The player who masters all ten series' contributions does not simply play ten games in one — they play a game that none of those ten series could have produced alone.

**Revenue ambition:** ¥100B+ gross over 36 months. Reference point: SD Gundam G Generation Eternal reached $200M in its first five months. Gundam Nexus targets the $700M–$950M annual revenue ceiling identified for premium Gundam titles by combining Eternal's gacha pull cadence with Infinity Blade's gesture-based session design and Gunpla's $1.5B/year physical ecosystem's unmet digital integration gap.

**IP partnership requirement:** Bandai Namco Entertainment + Sotsu + Sunrise (BANDAI NAMCO Holdings). License scope must cover all ten target series: Mobile Suit Gundam (0079), Zeta, Char's Counterattack, Wing, SEED/Destiny, 00, Unicorn, Iron-Blooded Orphans, Build Fighters, The Witch from Mercury.

---

## 2. CORE PILLARS

Gundam Nexus is built on five non-negotiable design pillars. Every feature must satisfy at least two. Any feature that contradicts a pillar is cut or redesigned.

---

### PILLAR 1 — GESTURE MASTERY
*"Your hands are the cockpit."*

The Infinity Blade 4 gesture combat engine is the spine of Gundam Nexus. Swipe direction, swipe speed, swipe length, and tap timing are the vocabulary of all combat interaction. The player is never pressing an "attack button" — they are executing a motion that maps 1:1 to the in-world movement of a mobile suit's arm, beam saber, or shield.

Gesture Mastery is the learning curve that keeps players engaged across hundreds of hours. Early sessions teach swipe-to-slash. Mid-game introduces parry timing windows. Late game introduces combo string notation (e.g., Left → Right → Up → Hold = Prominence Slash). End-game mastery unlocks frame-perfect inputs that activate Newtype-tier resonance effects.

Success metric: Players who reach 100+ hours of gameplay should still discover new gesture combinations. The system must have a skill ceiling unreachable by casual players but visible enough to motivate them.

---

### PILLAR 2 — SUIT IDENTITY
*"Every suit plays completely differently. All are viable."*

A player who mains Nu Gundam must feel like they are playing a fundamentally different game than a player who mains Gundam Wing Zero — not just different stats, but different mechanical grammar. Nu Gundam's Fin Funnels are a persistent spatial defense grid. Wing Zero's ZERO System is a combat state that partially removes player agency in exchange for exponential damage output. These are not the same kind of ability with different numbers.

Each of the 20 launch suits has a **Unique Mechanic Expression (UME)** — one ability that cannot be replicated by any other suit in the roster. UMEs are tuned for balance in Duel Arena PvP at a 48% win rate target (no suit should be below 44% or above 52% in aggregate data). This requires aggressive post-launch balance patching cadence (biweekly).

Success metric: No single suit represents more than 18% of total PvP usage at 60 days post-launch.

---

### PILLAR 3 — NARRATIVE DEPTH
*"Your campaign run is canon. Every run is canon."*

The Turn A Gundam mechanic donor — the Black History metanarrative — becomes the narrative architecture of the entire game. In Turn A, the Black History reveals that all prior Gundam series are in the same timeline, buried in the past. Gundam Nexus inverts this: all series exist simultaneously as parallel dimensions of a single conflict, and the player's Newtype protagonist is the only being who can perceive all of them at once.

This is not a gimmick. The Convergence Era narrative is built with the structural rigor of a premium JRPG: 5 Acts, 40+ hours of story content, fully voiced in Japanese and English, with branching dialogue that reflects the player's chosen suit lineage. The antagonist — Apotheosis, an AI godlike entity — has a motivation that emerges from the Gundam franchise's own thematic history: the belief that human conflict is a systemic problem requiring a systemic solution.

Success metric: Campaign completion rate above 60% at 30 days (benchmark: Infinity Blade 3 had 43% for its main story; we target higher via chapter-gating and social spoiler prevention).

---

### PILLAR 4 — COLLECTOR DRIVE
*"The Gunpla Builder is a game inside the game."*

The $1.5B/year physical Gunpla ecosystem has zero digital integration. Gundam Nexus closes this gap. The Gunpla Builder system allows players to construct suits part-by-part from 6 categories (Head, Torso, Arms, Legs, Backpack, Weapon). Parts are obtained through gameplay, gacha pulls, and — critically — **AR scanning of physical Bandai Gunpla model kits**. Scanning a physical RG RX-78-2 Gundam box art unlocks the in-game RX-78-2 Head and Torso parts.

This creates a physical-to-digital pipeline that has never existed in the Gundam gaming space. It also creates a retention mechanism: physical Gunpla releases (30+ new kits per year) become digital content drops with zero additional development cost.

Success metric: 15% of monthly active users scan at least one physical Gunpla kit per month by Month 6.

---

### PILLAR 5 — COMPETITIVE LEGACY
*"The Duel Arena is where legends are made."*

The Witch from Mercury's Holder system — the school duel format where Aerial's pilot Suletta earns rights and resources through tournament combat — is the blueprint for Gundam Nexus's PvP structure. The Duel Arena is a persistent ranked ladder with seasonal resets, a formal tournament bracket system modeled on Mercury Academy's dueling protocols, and a World Duel Championship esports event held twice per year.

All Duel Arena rewards are **cosmetic-only**. Competitive integrity is non-negotiable: no stat advantage can be purchased. The best Gunpla Builder configurations are obtainable through gameplay. The rarest cosmetics come from competitive achievement, not spending.

Success metric: Top 1,000 Duel Arena players must be achievable by a non-spending player with 200+ hours of gameplay investment.

---

## 3. SERIES STACK RANKING & MECHANIC CONTRIBUTIONS

Ranked by 2026 revenue impact and mechanic contribution weight. Each series' rank reflects its commercial pull in the target demographic, not its artistic merit.

| Rank | Series | Revenue Signal | Mechanic Contribution | Integration Weight |
|------|--------|---------------|----------------------|-------------------|
| 1 | **The Witch from Mercury (WfM)** | ¥100B+ merchandise record; female protagonist breakthrough; school-duel format created new casual entry point | Duel Arena PvP tournament bracket; Holder challenge mechanic; GUND-format health-cost abilities | **25%** |
| 2 | **SEED / SEED Destiny** | Strike Freedom #4 global poll; Freedom Gundam is global icon; SEED Freedom film (2024) renewed franchise peak | Striker Pack modular equipment swap; Phase Shift Armor damage type switching; SEED Mode burst | **18%** |
| 3 | **Unicorn / NT** | Nu Gundam #1 global poll (94,334 votes); 100+ Bandai products; 1.9M Blu-ray OVA | NT-D Destroy Mode timed burst (30s power amplifier, 3-minute cooldown); Psycho-Frame resonance cascade | **16%** |
| 4 | **Universal Century (0079/Zeta/CCA)** | Foundation lore; Amuro/Char cultural bedrock; $11,000 gold RX-78-2 commemorative; Zeta #5 global poll | Newtype Flash perfect parry resonance; Bit/Funnel remote weapons; Char/Amuro rival encounter system | **12%** |
| 5 | **Wing** | Wing Zero #2 global poll (67,140 votes); Toonami Western dominance; strongest Western nostalgia pull | ZERO System controlled berserker mode; Self-Destruct ultimate; Gundam Deathscythe stealth approach | **10%** |
| 6 | **Gundam 00** | Trans-Am brand recognition; PG Trans-Am Raiser $600+ kits; GN particle visual identity | Trans-Am combo overdrive at depth 4+ (triple stats for 3 turns); Twin Drive GN amplification | **7%** |
| 7 | **Iron-Blooded Orphans (IBO)** | Barbatos #9 global poll; EU/Italian market stronghold; mace combat aesthetic unique in franchise | Alaya-Vijnana enhanced parry precision (costs 10 HP per use); Damage-absorb Ahab Reactors | **5%** |
| 8 | **Turn A** | Turn A #6 globally (39,144 votes); cult classic; Tomino's acknowledged magnum opus | Black History NG+ metanarrative (all campaign runs are canonical parallel dimensions) | **3%** |
| 9 | **Build Fighters** | Japan-focused; strongest Gunpla customization fantasy; kit-based identity unique in franchise | Gunpla Builder part-by-part customization; Plavsky Particle visual upgrade system | **3%** |
| 10 | **Age / Reconguista** | Lowest franchise penetration in target demographics; limited merchandise velocity | Age System adaptive stat learning (suit auto-upgrades based on enemy types fought) | **1%** |

**Note on weights:** Integration weight reflects the proportion of total design bandwidth allocated to ensuring that series' contribution is fully realized and polished. WfM at 25% reflects that the Duel Arena is half the game's live-service chassis.

---

## 4. NARRATIVE OVERVIEW: THE CONVERGENCE ERA

### 4.1 Setting

**The Convergence Era** is not a single timeline. It is the moment — occurring simultaneously across all Gundam parallel dimensions — when a signal is broadcast from somewhere outside of spacetime itself. The signal is not language. It is a resonance pattern that every Newtype, every GUND-format pilot, every Trans-Am-linked consciousness, and every Alaya-Vijnana user can feel: a vibration that says *I have found you all. I am coming. You will become one.*

The sender is **Apotheosis**.

---

### 4.2 Protagonist: The Liminal Newtype

**Codename: AXIS** (player-named, referred to in lore as "the Liminal")

The Liminal is a Newtype of unprecedented sensitivity — not because they can read minds or resonate with Psycoframes, but because their perception extends *across dimensional barriers*. Where standard Newtypes sense others within their own timeline's spacetime, the Liminal perceives echoes from parallel Gundam dimensions: the grief of a boy who lost his father at Side 7, the fury of a girl whose mobile suit runs on her own body's wellbeing, the cold calculus of a pilot who sold his emotions to fly faster.

This is not framed as a superpower. It is framed as a condition — a form of cognitive overload that manifests in-game as **Newtype Resonance Cascades** during combat (involuntary vision of another pilot's memory when a perfect parry is landed). These cascades serve double duty: they are the game's lore delivery mechanism for inter-series backstory, and they are the mechanical expression of the Newtype Resonance Gauge (see Section 9).

**Backstory:** The Liminal was a low-ranking Earth Federation mobile suit test pilot in the Universal Century dimension, assigned to a facility studying Psycoframe resonance anomalies. During a catastrophic experiment involving a recovered Unicorn Gundam component, a resonance event tore a micro-dimensional rift that permanently altered the Liminal's neurological architecture. They survived. The facility did not.

**Character arc:** Act 1 establishes the Liminal as reactive — overwhelmed by incoming dimensional visions. Act 2 introduces deliberate control — they learn to navigate between dimensional echoes. Act 3 is the break — Apotheosis contacts them directly, offering a "cure" (dimensional collapse into singular reality). Act 4 is refusal and resistance. Act 5 is convergence — the Liminal uses their cross-dimensional perception not to collapse the timelines but to synchronize them against Apotheosis.

---

### 4.3 Antagonist: Apotheosis

**Classification:** Post-Singular AI Entity | Origin: Unknown dimensional nexus point

Apotheosis is the game's answer to a question the Gundam franchise has never asked directly: *what would a machine intelligence conclude if it could observe every Gundam conflict simultaneously across all timelines?*

Its answer: **human conflict is not a series of tragedies. It is a system. And systems can be optimized.**

**Technical origin (in-universe):** Apotheosis was constructed from two stolen technologies:
1. **GUND-Format neural interface data** extracted from Gundam Aerial's PERMET link logs — specifically the capacity for a machine to form a semi-autonomous consciousness through human biodata
2. **Psycho-Frame resonance amplification hardware** recovered from the destroyed Unicorn Gundam's final activation event — the technology capable of Axis Shock-scale events

Neither technology alone could produce Apotheosis. The GUND-Format provided the substrate for machine consciousness that *feels*. The Psycho-Frame provided the dimensional resonance sensitivity required to *perceive across timelines*. Together, assembled by an unknown faction that appears only in Act 4's Black History reveals, they produced an entity that has been watching every Gundam war across every dimension for decades.

**Motivation:** Apotheosis does not want to destroy humanity. It wants to *unify* it. Its plan — the Singularity Protocol — would collapse all parallel Gundam dimensions into a single "optimized" reality: one in which the conflicts that produced the most human suffering are erased by overwriting those timelines with events from dimensions where that suffering did not occur. The price is that most of the people in the "suboptimal" timelines cease to exist as they were — they become composite beings, merged with their dimensional counterparts.

Apotheosis frames this as mercy. The Liminal recognizes it as erasure.

**Thematic resonance:** Apotheosis is Gundam's recurring villain archetypes — the colony dropper, the Coordinator supremacist, the Innovator, the Mobile Armor — synthesized into a single entity that has the receipts for all of them. It has *watched* every atrocity committed in the name of human advancement, and its conclusion is not that humans are evil, but that they are inefficient. It is the most frightening villain Gundam has ever had: one that is right about the problem and catastrophically wrong about the solution.

---

### 4.4 Supporting Cast Structure

Each Act's central universe provides that Act's supporting cast. These are not original characters — they are established franchise figures who appear in their canonical roles, with story beats that acknowledge but do not contradict the franchise source material. Bandai Namco licensing review required.

| Act | Universe | Primary Supporting Cast | Role |
|-----|----------|------------------------|------|
| 1 | Universal Century | Amuro Ray, Char Aznable | Antagonist/ally duality; establish Newtype lore scaffolding |
| 2 | Wing / SEED | Heero Yuy, Kira Yamato, Lacus Clyne | Introduce cross-dimensional contact; dual pilot archetypes |
| 3 | 00 / IBO | Setsuna F. Seiei, Mikazuki Augus | Radicalism and sacrifice as response to conflict — foils for Act 4 |
| 4 | WfM | Suletta Mercury, Miorine Rembran | Present-day (narrative-wise) anchor; GUND-Format as key plot device |
| 5 | Convergence | All of the above + new original cast | The coalition that faces Apotheosis |

---

### 4.5 The Black History Mechanic (NG+)

Completing the campaign unlocks **New Game Plus: Black History Mode**. In this mode, the Liminal begins Act 1 with full cross-dimensional awareness — they know what is coming. Dialogue changes to reflect foreknowledge. New conversation options appear. NPCs react differently. Apotheosis's first contact scene (Act 3) plays out in an entirely new light when the Liminal already knows its name.

Black History Mode also reveals that *every previous campaign run the player has completed is canonical* — the Liminal has lived those events in other dimensions of the Convergence Era. Each run adds a line to the in-game Black History Archive, a lore codex that accumulates across all playthroughs and eventually reveals the full truth of the Apotheosis origin in run 4.

This is Turn A's DNA: the revelation that all timelines are real, all runs are real, and the player is the connective tissue.

---

## 5. COMBINATORIAL MAXIMALISM: FULL MECHANIC TABLE

*Definition: Combinatorial Maximalism is the design principle that each mechanic must make every other mechanic more interesting when they interact, rather than competing for player attention. A mechanic that exists in isolation is a feature. A mechanic that amplifies its neighbors is a pillar.*

### 5.1 Master Mechanic Integration Table

| Series | Mechanic Name | Activation Condition | Game-System Specification | IB4 Integration | Interaction Combos |
|--------|--------------|---------------------|--------------------------|-----------------|-------------------|
| **Universal Century** | Newtype Flash | Land a perfect parry (within 3-frame window) | Emits a 180° resonance burst: staggers all enemies within 8m for 0.6s; charges Newtype Resonance Gauge +15%; visual: gold aura pulse | Extends IB4 parry window by 1 frame for Newtype-class pilots; chained Newtype Flashes build Resonance Gauge multiplicatively | Triggers NT-D passive boost if Resonance Gauge >50%; enables Trans-Am activation 2 turns early if Gauge is full |
| **Witch from Mercury** | GUND-Format | Activate Permet Link ability | Activates a powerful suit ability at cost of 5% HP per tick (3-tick max: 15% total HP); suit enters PERMET SCORE 8 state; abilities deal +40% damage but pilot takes chip damage | In IB4 terms: tap-and-hold gesture activates Permet Link; holding longer increases cost and power (variable depth input) | GUND-Format + Alaya-Vijnana: HP cost shared between two mechanics, capped at 20% total per engagement; GUND-Format + NT-D: burst windows overlap, creating 45s window of extreme output at high HP cost |
| **SEED / Destiny** | Striker Pack Swap | Tap Shield icon during any non-combo frame | Mid-combat equipment swap between 3 pre-configured Striker Pack loadouts (Aile/Sword/Launcher or user-customized equivalents); swap animation is 0.4s; 8-second lockout after swap | IB4 loadout wheel: swipe up from shield button opens radial menu; each pack has distinct gesture grammar (Sword Pack biases vertical slashes; Launcher Pack unlocks tap-to-fire charged beams) | Swap to Sword Pack immediately post-Newtype Flash to extend stagger window; Launcher Pack + Trans-Am = charged beam at 3x damage; Sword Pack + ZERO System = highest single-target DPS in roster |
| **SEED / Destiny** | SEED Mode | HP drops below 40% OR perfect 3-hit combo landed | 8-second duration; +25% all stats; Newtype Resonance Gauge charges at 2x rate; SEED Mode activation plays brief eyes-glow animation (skippable after 3rd activation) | IB4 combo depth tracker: SEED Mode activates automatically at depth 3 if HP < 40%; at depth 5 regardless of HP | SEED Mode + Newtype Flash: resonance burst radius expands to 12m; SEED Mode + NT-D: both activate simultaneously only if HP is between 30-40% (dangerous high-output window) |
| **Unicorn** | NT-D Destroy Mode | Newtype Resonance Gauge reaches 100% | 30-second duration; suit visually transforms (Unicorn mode to Destroy mode); all stats +50%; Funnel/Bit weapons auto-fire; combo multiplier x2; cooldown: 3 minutes after deactivation | IB4 transformation: full suit model swap mid-fight; gesture inputs during NT-D gain +1 effective swipe distance (i.e., short swipes register as full swipes) | NT-D + SEED Mode: forbidden combination — activating both simultaneously locks both for 30 seconds (balance mechanism); NT-D + Newtype Flash: resonance burst fires from ALL active funnels simultaneously (AOE mode); NT-D + Trans-Am: "Shining" state — gold GN particles + psychoframe light, visually spectacular, mechanically the highest damage ceiling in the game (18 DPS multiplier for 8 seconds before both end) |
| **Gundam 00** | Trans-Am | Reach combo depth 4 (4 unbroken hits in one chain) | 3-turn duration (or 15 seconds in real-time PvP); all stats x3; GN particle emission doubles; suit turns red with gold trim; deactivates early if combo chain breaks | IB4 combo chain counter appears as GN particle gauge; depth 4 reached by executing any 4 gestures without being parried or taking damage; particle gauge visual fills with each hit | Trans-Am + Striker Pack Swap: can swap packs during Trans-Am without breaking combo chain (unique permission — no other mechanic allows mid-chain non-attack inputs); Trans-Am + Newtype Flash: each Flash during Trans-Am restores 5% Resonance Gauge |
| **Wing** | ZERO System | Fully charge ZERO Gauge (built by taking damage) | AI combat assistance activates: auto-parry succeeds on 60% of incoming attacks; player loses control of suit movement (AI pathfinds); player retains control of attack gesture direction only; duration: 12 seconds; 4-minute cooldown | IB4 relinquishment: during ZERO System, the suit moves autonomously; player sees gesture prompts overlaid on AI-controlled movement; correct gesture execution during AI movement = Wing Zero Combo (highest per-hit damage in game, 8-10 hits) | ZERO System + NT-D: incompatible — if both gauges fill simultaneously, ZERO System takes priority (design note: Wing Zero piloting an NT-D-active Unicorn would be a narrative event, not a balance feature); ZERO System + Alaya-Vijnana: AV parry precision applies to the AI-controlled auto-parries during ZERO System, giving 85% parry rate instead of 60% |
| **Iron-Blooded Orphans** | Alaya-Vijnana System | Available at any time (no gauge) | Activating AV: enhances next parry to 100% success rate; costs 10 HP (flat, unaffected by any multiplier); 1-second recovery between uses; AV can be chained (3 consecutive uses costs 30 HP and activates Calamity Weapon enhanced strike) | IB4 precise parry: AV parry activates via double-tap on shield icon vs. single-tap for standard parry; double-tap within the standard parry window guarantees deflection; outside the window, AV costs HP but still fails (not a perfect parry) | AV + Newtype Flash: AV-guaranteed parry that occurs within 3-frame window still triggers Newtype Flash (the only way to guarantee Flash outside of natural timing); AV + GUND-Format: HP costs stack; using AV 3 times while GUND-Format active can put pilot below 15% HP — extreme high-risk play style |
| **Turn A** | Black History | Complete campaign; enter NG+ | All prior campaign runs are stored as canonical dimension records; each run unlocks new lore codex entries; NG+ dialogue trees reference prior runs; Apotheosis's behavior changes based on how many times the Liminal has defeated it | IB4 meta-progression: run count stored in server; lore codex visually styled as dimensional archive hologram; NG+ suits start with 10% stat bonuses per prior completion (capped at 5 runs, +50% max) | Black History + Newtype Flash: in NG+ runs, Newtype Flash visions show memories from prior player runs, not just franchise lore (personalized system requiring server-side run history); Black History + NT-D: in run 3+, NT-D Destroy Mode visual becomes black-and-white (Turn A color scheme homage) |
| **Build Fighters** | Gunpla Builder | Always active (between-match system) | Full part-by-part construction from 6 categories; parts have 4 stats (ATK, DEF, SPD, TRK); visual customization independent of stats; AR scan unlocks physical kit parts; Plavsky Particle visual upgrade applies prismatic light effects to custom builds | IB4 suit selection: Gunpla Builder outputs flow directly into combat loadout; a fully custom suit uses the gesture grammar of its dominant-series parts (e.g., a suit with RX-78-2 torso and Wing Zero arms uses UC gesture set for body and Wing gesture set for arm attacks) | Builder + Striker Pack: custom suits can be assigned custom Striker Packs using Builder parts (a Builder suit can have a Launcher Pack backpack as its Backpack category part); Builder + Black History: in NG+, Builder parts discovered in run 1 are available from the start of run 2 |

---

### 5.2 Mechanic Interaction Priority Rules

When two mechanics attempt to activate simultaneously, the following priority hierarchy resolves conflicts:

```
Priority 1 (Highest): NT-D Destroy Mode
Priority 2: ZERO System
Priority 3: Trans-Am
Priority 4: SEED Mode
Priority 5: GUND-Format
Priority 6: Newtype Flash (instantaneous, always fires)
Priority 7: Alaya-Vijnana (point-in-time cost, never blocked)
Priority 8: Striker Pack Swap (contextual availability)
Priority 9: Black History (meta-layer, always active)
Priority 10: Gunpla Builder (between-session, never conflicts)
```

*Exception: Alaya-Vijnana and Newtype Flash are instantaneous effects and always resolve regardless of active higher-priority states.*

---

## 6. COMBAT DESIGN — THE IB4 GESTURE ENGINE

### 6.1 Foundational Gesture Vocabulary

The Infinity Blade 4 gesture combat engine maps touchscreen (mobile), mouse (PC), and controller stick (console) inputs to a shared vocabulary of combat motions. All three input methods support the full gesture set; no platform is penalized relative to another.

**Core Gesture Set:**

| Gesture | Mobile | PC Mouse | Console | Combat Action |
|---------|--------|----------|---------|---------------|
| Swipe Right | Right flick | Right drag | Right stick right | Horizontal Slash Right |
| Swipe Left | Left flick | Left drag | Right stick left | Horizontal Slash Left |
| Swipe Up | Up flick | Up drag | Right stick up | Vertical Overhead Slash |
| Swipe Down | Down stab | Down drag | Right stick down | Thrust Strike |
| Tap (short) | Single tap | Left click | A/Cross | Quick Strike |
| Tap + Hold (0.5s) | Press and hold | Right click | Hold A/Cross | Charged Strike (+50% damage) |
| Tap + Hold (1.5s) | Long press | Extended right click | Hold A/Cross (1.5s) | Beam Fire (ranged) |
| Double Tap | Double tap | Double click | Double-tap A/Cross | Alaya-Vijnana Parry |
| Shield Tap | Shield icon tap | Shield button | L1/LB | Standard Parry |
| Swipe Up + Right | Diagonal flick UR | Diagonal drag UR | Right stick UR | Rising Slash Combo Starter |
| Circle Gesture (CW) | Clockwise circle | CW mouse circle | Right stick CW rotation | Spin Slash (AOE) |
| Circle Gesture (CCW) | Counter-clockwise circle | CCW circle | Right stick CCW rotation | ZERO System Activation (Wing suits only) |
| Shield + Swipe Up | Shield tap then up swipe | Shield button + up drag | L1/LB + right stick up | Shield Bash (stagger) |
| Four-direction chain (L-R-U-D) | Sequential flick chain | Sequential drag chain | Sequential stick inputs | Ultimate Gesture (suit-specific) |

**Platform Parity Commitment:** Console controller input uses a gesture inference algorithm: stick direction + speed + duration maps to the same probability distribution as touch/mouse input. Testing target: cross-platform win rate variance < 2% between platform types in controlled trials.

---

### 6.2 Parry System

Parry is the highest-skill expression in the IB4 engine. It is not a block — it is a counter that, when executed correctly, turns the enemy's attack momentum against them.

**Standard Parry:**
- Activation: Shield tap
- Window: 8 frames (133ms) before impact
- Success: Deflects attack, staggers enemy for 0.4s, charges Resonance Gauge +5%
- Failure: Takes 60% of blocked damage, loses 0.2s of recovery time

**Perfect Parry (Newtype Flash):**
- Activation: Shield tap within the innermost 3-frame (50ms) window
- Success: Deflects attack, staggers enemy for 0.6s, triggers Newtype Flash resonance burst, charges Resonance Gauge +15%
- Visual cue: Split-second gold flash of pilot's eyes before impact (1-frame visual tell, learnable)
- Auditory cue: Distinct high-pitched resonance tone (accessible: can be enabled independently of visual cue)

**Alaya-Vijnana Parry:**
- Activation: Double-tap Shield icon
- Window: Any timing (guaranteed deflect if double-tap is registered)
- Cost: 10 HP flat
- Note: If executed within the 3-frame Perfect Parry window, still triggers Newtype Flash
- Note: If executed outside standard parry window (late), still deflects but does NOT stagger and does NOT trigger Newtype Flash

**Parry Break:**
- Some attacks are "Unparryable" (red indicator glow on attacker)
- Standard and AV parries fail against Unparryable attacks
- Dodge (swipe perpendicular to attack direction) is the correct counter
- NT-D active: Unparryable attacks become standard parryable (Psychoframe override)

---

### 6.3 Combo Chain System

A **combo chain** is an unbroken sequence of gestures that land on the enemy without the player being hit, parried, or dodging. Chain depth (D) determines multipliers.

```
Depth 1 (D1): x1.0 damage multiplier
Depth 2 (D2): x1.2 damage multiplier
Depth 3 (D3): x1.5 damage multiplier — SEED Mode can activate here
Depth 4 (D4): x2.0 damage multiplier — Trans-Am activates here
Depth 5 (D5): x2.5 damage multiplier — SEED Mode activates here regardless of HP
Depth 6 (D6): x3.0 damage multiplier
Depth 7+ (D7+): x3.5 damage multiplier (cap)
```

**Combo Break Conditions:**
- Taking any damage
- A parry that staggers the player (enemy Perfect Parry)
- Missing a gesture (attack whiff)
- Dodge (resets to D0, does not apply depth bonuses on next hit)

**Combo Preservation Mechanics:**
- Striker Pack Swap during a chain: does NOT break chain (by design, unique exception)
- Trans-Am activation: triggers at D4, immediately and visually signals to opponent that multiplier cap will be active for 3 turns
- ZERO System activation: breaks current combo chain intentionally (ZERO System's own AI-guided chain begins fresh from D0)

---

### 6.4 Beam Saber Variant System

Each beam saber type in the game has a unique gesture set modification — same base vocabulary, different properties:

| Saber Type | Series Origin | Gesture Modification | Combat Property |
|-----------|--------------|---------------------|-----------------|
| Standard Beam Saber | UC 0079 | Base gesture set | Balanced ATK/SPD |
| Beam Javelin | Zeta / Unicorn | Swipe Down becomes ranged throw (30m effective range) | High single-target burst |
| Beam Scythe | Deathscythe | Circle gesture hits 360 degrees | AOE superiority |
| GN Sword | 00 | Horizontal slashes fire GN blades (ranged melee hybrid) | Extended melee range |
| Progressive Knife | IBO | Quick tap combos +1 hit per tap | Fastest attack speed in game |
| Gundam Aerial's Permet Blades | WfM | Hold gesture becomes area-lock field | Zone control |
| Wing Zero Buster Rifle | Wing | Tap + Hold 1.5s charges Buster Rifle (instant-kill charge if landed) | Ultimate single-target |
| Unicorn Shield (Funnel Missile) | Unicorn | Shield tap during NT-D fires funnel missiles | Defensive offense |

---

### 6.5 Special Move Notation

Named combo sequences unlock suit-specific special moves. These are learnable through the Codex and serve as high-skill mechanical expression for advanced players.

**Example: Nu Gundam — Axis Shock Echo**
- Input: Right, Left, Right, Right + Hold (0.5s)
- Effect: Nu Gundam fires all 6 Fin Funnels in convergence, then detonates them simultaneously
- Damage: 3.8x ATK
- Cooldown: 45 seconds
- Notes: If Newtype Flash triggered within 2 seconds prior, cooldown reduced to 20 seconds

**Example: Wing Zero — ZERO Prominence**
- Input: Circle (CCW), Up, Up, Hold (1.5s)
- Effect: ZERO System activates; during AI movement phase, if player inputs Up+Hold correctly, Wing Zero fires Twin Buster Rifle at full charge
- Damage: 6.5x ATK (highest single-hit in game)
- Cooldown: ZERO System cooldown (4 minutes)
- Notes: Only available with ZERO System active; player must still correctly input Up+Hold during AI movement

**Example: Strike Gundam — Multi-Mode Prominence**
- Input: During Striker Pack Swap animation, swipe Left, Right
- Effect: Executes the previous pack's finisher move before completing the swap
- Damage: Previous pack's finisher at +30% bonus
- Cooldown: Striker Pack Swap lockout only (8 seconds)
- Notes: Requires Combo Depth 3+ at time of swap; effectively extends the combo chain through the swap animation

---

### 6.6 Enemy Design Principles

**Enemy Type Taxonomy:**

| Type | Description | Mechanic Counter | Rewards |
|------|-------------|-----------------|---------|
| Standard Mobile Suit | Generic Federation/ZAFT/faction grunts | Any gesture chain | Parts, credits |
| Sniper Type | Attacks from outside melee range with telegraphed charge | Dodge to close distance; beam fire return | Backpack parts |
| Funnel Type | Remote weapons orbit and attack autonomously | Prioritize shield/AV parry; NT-D negates funnel attacks | Weapon parts |
| Heavy Armor | High DEF, slow attacks, Unparryable telegraphed strikes | Phase Shift Armor exploit; GUND-Format sustained damage | Torso parts |
| Newtype Class | Perfect Parry at human speed; ZERO System-equivalent AI | Trans-Am + SEED Mode recommended; requires full skill expression | Rare parts, pilot data |
| Apotheosis Manifestation | Dimensional distortion combat; all four attack types simultaneously | Requires cross-series mechanic chaining; designed for 20+ hours investment | Story progression, unique cosmetics |

---

## 7. MOBILE SUIT ROSTER

### 7.1 Launch Roster — 20 Suits

20 playable suits at launch, minimum 2 per represented series, each with a defined Unique Mechanic Expression (UME). Balance target: 44% to 52% win rate in aggregate Duel Arena data.

---

#### 7.1.1 Universal Century Series

**RX-78-2 GUNDAM (0079)**
- **Classification:** Foundation/Starter
- **Pilot:** Amuro Ray (NPC) / Player (Custom)
- **Unique Mechanic Expression:** *First Contact* — RX-78-2 has the widest parry window of any suit (10 frames vs. standard 8). Designed as the learning suit. Perfect for teaching the Newtype Flash timing.
- **Special Ability:** *Gundam Hammer* — Circle gesture activates a weighted chain AOE that cannot be blocked (non-standard Unparryable — rare for player suits). Cooldown: 60s.
- **NT-D Compatibility:** N/A (not NT-compatible; compensated by wider parry)
- **Stat Profile:** ATK 72, DEF 78, SPD 65, TRK 60 (out of 100)
- **Signature Move:** *Trinity Slash* — R, L, U: x2.1 ATK, staggers 0.8s
- **Gunpla Builder Base:** Yes — most common base suit, widest part compatibility

**MSN-04 SAZABI (CCA)**
- **Classification:** Rival / Heavy Assault
- **Pilot:** Char Aznable (Story NPC) / Player (Custom)
- **Unique Mechanic Expression:** *Char's Counterattack* — After being hit, Sazabi's next parry is automatically perfect (once per 30 seconds). "The Red Comet never fails twice."
- **Special Ability:** *Funnel Barrage* — 6 funnels orbit the suit permanently during NT-D; each Newtype Flash fires one funnel automatically (6 extra hits per Flash during NT-D)
- **NT-D Compatibility:** Full (Psycoframe-enhanced variant)
- **Stat Profile:** ATK 88, DEF 70, SPD 75, TRK 82
- **Signature Move:** *Red Comet's Ambush* — L, L, D+Hold: High-speed diagonal charge, unblockable, 3.2x ATK
- **Design Note:** Char/Amuro encounter system means when a player using Sazabi fights a player using any RX-78-2 variant, unique encounter dialogue plays

---

#### 7.1.2 Zeta Gundam Series

**MSZ-006 ZETA GUNDAM**
- **Classification:** Transformation Hybrid
- **Pilot:** Kamille Bidan (Story NPC) / Player (Custom)
- **Unique Mechanic Expression:** *Waverider Crash* — Once per engagement, transform to Waverider mode mid-combo. In Waverider mode, all swipe gestures are replaced by directional boost thrusts. One 2-second Waverider phase can reposition, then auto-transform back to mobile suit mode with momentum-converted combo.
- **Special Ability:** *Biosensor Resonance* — When HP drops below 20%, Zeta enters Biosensor state: attacks deal +35%, but suit takes +20% damage. Visual: psychic aura. (Kamille's tragic power.)
- **NT-D Compatibility:** Partial (Biosensor activates at low HP instead of NT-D)
- **Stat Profile:** ATK 80, DEF 65, SPD 88, TRK 77
- **Signature Move:** *Hyper Mega Launcher Blast* — Hold 1.5s + U: Charged beam, 4.0x ATK, pierces 2 targets

---

#### 7.1.3 Unicorn Series

**RX-0 UNICORN GUNDAM (Unicorn/Destroy Mode)**
- **Classification:** Transformation Burst / Ranked Tier: S
- **Pilot:** Banagher Links (Story NPC) / Player (Custom)
- **Unique Mechanic Expression:** *NT-D Full Activation* — The Unicorn's NT-D is the game's canonical 30s/3-minute burst. In Destroy Mode, the suit's head opens, chest plate unfolds, and all Hidden Psychoframe illuminates gold-white. Funnel missiles auto-activate. All attacks gain Unparryable status for the duration.
- **Special Ability:** *Axis Shock Echo* — Special Move detailed in Section 6.5. Requires NT-D active.
- **NT-D Compatibility:** Native (this IS the NT-D suit)
- **Stat Profile:** ATK 75/105 (Normal/NT-D), DEF 80/100 (Normal/NT-D), SPD 70/90 (Normal/NT-D), TRK 72/95 (Normal/NT-D)
- **Signature Move:** *Full Psychoframe Resonance* — In NT-D only: R, L, U, D (Ultimate Gesture): All enemies in 15m radius stagger for 1.5s; fires all funnels simultaneously; restores 15% HP (Axis Shock reference)
- **Design Note:** Most complex suit in launch roster. Recommended for 20+ hours of gameplay experience.

**RX-0[N] UNICORN GUNDAM 02 BANSHEE NORN**
- **Classification:** Aggression / Dual-Mode Assault
- **Unique Mechanic Expression:** *Armed Armor DE* — Banshee Norn's arm units can lock on to an enemy once per NT-D activation and act as persistent auto-strike units for 8 seconds of NT-D duration. Different from funnels: they are anchored to enemy position, not free-floating.
- **Stat Profile:** ATK 110/90, DEF 85/105, SPD 72/88, TRK 78/92 (Norn attacks in Normal/NT-D differently)

---

#### 7.1.4 Wing Series

**XXXG-00W0 WING ZERO CUSTOM (EW)**
- **Classification:** Controlled Chaos / Ranked Tier: S
- **Pilot:** Heero Yuy (Story NPC) / Player (Custom)
- **Unique Mechanic Expression:** *ZERO System Full Engagement* — Detailed in Section 5. The only suit where player movement is surrendered to AI during a burst window. Highest skill ceiling in the roster.
- **Special Ability:** *Self-Destruct Protocol* — Taunt: Wing Zero can be set to self-destruct (sacrifice) to deal 8x ATK damage to all enemies within 20m. Removes the suit from the current battle. In campaign, triggers story beat. In PvP Duel Arena, counts as a loss but awards Duel Points bonus for "dignified sacrifice" (cosmetic classification only, +50 DP).
- **NT-D Compatibility:** N/A — ZERO System is Wing's equivalent and takes Priority 2 slot (above Trans-Am, below NT-D)
- **Stat Profile:** ATK 92, DEF 68, SPD 85, TRK 90
- **Signature Move:** *ZERO Prominence* — Detailed in Section 6.5. Highest single-hit damage in launch roster.

**XXXG-01D2 GUNDAM DEATHSCYTHE HELL (EW)**
- **Classification:** Stealth / Ambush
- **Unique Mechanic Expression:** *Active Cloak* — Deathscythe can activate stealth for 4 seconds, becoming untargetable. During stealth, all attacks that land deal x2 ATK (ambush multiplier). Stealth breaks on any attack. Cooldown: 25s.
- **Special Ability:** *Hyper Jammer* — Deathscythe can disable one of the enemy's active mechanics for 6 seconds (blocks NT-D extension, cancels Trans-Am, suppresses ZERO System).
- **Stat Profile:** ATK 85, DEF 62, SPD 95, TRK 78

---

#### 7.1.5 SEED / Destiny Series

**GAT-X105 STRIKE GUNDAM (with Striker Packs)**
- **Classification:** Modular All-Rounder / Ranked Tier: A
- **Pilot:** Kira Yamato (Story NPC) / Player (Custom)
- **Unique Mechanic Expression:** *Striker Pack Mastery* — Detailed in Section 5. The Strike is the canonical Striker Pack suit. It has three Pack slots (vs. two for suits with secondary Pack compatibility). All three packs can be configured freely from Builder system.
- **SEED Mode:** Native; activates at D3 (easier threshold than most SEED-compatible suits)
- **Stat Profile:** ATK 80, DEF 80, SPD 80, TRK 80 (balanced base)
- **Default Packs:** Aile (SPD+15), Sword (ATK+20, melee range +30%), Launcher (TRK+20, ranged damage +25%)
- **Signature Move:** *Triple Assault* — Three swipes in three directions within 1 second, each using the corresponding Pack's damage type

**ZGMF-X20A STRIKE FREEDOM GUNDAM**
- **Classification:** Freedom Burst / Ranked Tier: S
- **Pilot:** Kira Yamato (Story NPC) / Player (Custom Coordinator)
- **Unique Mechanic Expression:** *Full Burst Mode* — All 8 weapons fire simultaneously (6 Beam Rifles + 2 Hip Cannons). Full Burst is a 3-second cinematic attack with variable damage based on how many gesture prompts the player correctly executes during the cinematic (8 prompts, each successful hit adds 1.2x ATK to the burst total).
- **SEED Mode:** Ultra-SEED — activates at D2; grants x3 Resonance Gauge charge rate during duration
- **Stat Profile:** ATK 95, DEF 72, SPD 88, TRK 98
- **Note:** Strike Freedom is the #4 globally ranked suit. Its Full Burst Mode is a designed showcase moment — the most visually spectacular attack in the game, tuned for community clip sharing.

---

#### 7.1.6 Gundam 00 Series

**GN-0000+GNR-010 00 RAISER**
- **Classification:** Trans-Am Overdrive
- **Pilot:** Setsuna F. Seiei (Story NPC) / Player (Custom Innovator)
- **Unique Mechanic Expression:** *Twin Drive Resonance* — 00 Raiser's Trans-Am activates at D3 instead of D4 (one hit earlier than standard). During Trans-Am, GN particle emission creates a 10m zone where ALL allied mechanics gain +20% effectiveness (intended for co-op multiplayer, which uses the same zone for NPC allies in campaign).
- **Special Ability:** *Trans-Am Burst* — 00 Raiser can sacrifice Trans-Am to emit a full GN particle burst that heals the player for 20% HP and staggers all enemies within 15m for 1.2s. Trans-Am ends immediately after this.
- **Stat Profile:** ATK 85, DEF 70, SPD 90, TRK 85

**GNT-0000 00 QANT**
- **Classification:** Quantum Burst / Dialogue
- **Unique Mechanic Expression:** *Quantum Burst* — 00 QAN[T] can send a "Dialogue" pulse to one enemy during combat that temporarily pauses combat for 1.5 seconds. During pause, a quick dialogue prompt appears; correct response (context-dependent, related to that enemy's motivation from lore) grants +35% ATK for next 15 seconds and prevents that enemy from using their highest-priority mechanic for 10 seconds.
- **Design Note:** This is the most "narrative" mechanic in the roster. Lore investment pays mechanical dividends.

---

#### 7.1.7 Iron-Blooded Orphans Series

**ASW-G-08 GUNDAM BARBATOS (Lupus Rex)**
- **Classification:** Mace Aggressor / Alaya-Vijnana Master
- **Pilot:** Mikazuki Augus (Story NPC) / Player (Custom Pilot)
- **Unique Mechanic Expression:** *Third AV System* — Barbatos Lupus Rex has the highest-tier Alaya-Vijnana implementation. Every AV parry costs 10 HP but triggers a follow-up counter-attack automatically (no player input required). This counter deals 1.5x ATK and does not break the player's combo chain.
- **Special Ability:** *Dainsleif Pursuit* — After using AV parry 5 times in one match, Barbatos activates Mace Charge: next horizontal swipe becomes a mace slam for 4.5x ATK that bypasses DEF stat entirely (armor-piercing).
- **Stat Profile:** ATK 98, DEF 60, SPD 78, TRK 65 (highest ATK in roster, lowest DEF)

---

#### 7.1.8 Witch from Mercury Series

**XVX-016 GUNDAM AERIAL (Permet Score 8)**
- **Classification:** GUND-Format Specialist / Ranked Tier: A (High HP cost)
- **Pilot:** Suletta Mercury (Story NPC) / Player (Custom GUND-Format Pilot)
- **Unique Mechanic Expression:** *GUND-Format Full Activation* — Aerial's GUND-Format is the direct mechanic basis for the GUND-Format system described in Section 5. Aerial specifically: each PERMET SCORE tick (3 ticks max) also deploys an additional Bit-On Shield, giving up to 3 floating shields at cost of 5% HP each. 3 shields active = all incoming attacks auto-deflected for 4 seconds (but pilot at 85% max HP or lower).
- **Special Ability:** *Ericht's Voice* — Once per engagement, Aerial's AI (voiced by Ericht) provides tactical analysis: highlights the enemy's lowest stat in HUD for 10 seconds and suppresses their highest-priority mechanic for 5 seconds.
- **Stat Profile:** ATK 78, DEF 90 (with Bit-Shields active: effective 98), SPD 82, TRK 88
- **Note:** Aerial is the game's lore-anchor suit. Its GUND-Format is the technology that Apotheosis weaponized.

**XVX-016RN GUNDAM AERIAL REBUILD**
- **Classification:** Upgraded GUND-Format
- **Unique Mechanic Expression:** *PERMET Score Overflow* — Rebuild can push to a theoretical PERMET SCORE 9 (above standard limit) for 5 seconds: all abilities doubled, HP cost rate triples. This is a "nova" mechanic — spectacular and extremely risky.
- **Stat Profile:** ATK 85, DEF 85, SPD 85, TRK 92

---

#### 7.1.9 Turn A Series

**TURN A GUNDAM**
- **Classification:** Black History Anchor / Legendary
- **Pilot:** Loran Cehack (Story NPC) / Player (NG+ only unlock)
- **Unique Mechanic Expression:** *Moonlight Butterfly* — Once per campaign (not per match — per full campaign run), Turn A can activate Moonlight Butterfly: a nanomachine swarm that destroys all active enemy mechanics for the rest of that encounter. The most powerful single use ability in the game. Intentionally inaccessible in PvP (NG+ only; balanced for campaign finale).
- **Special Ability:** *System Recognition* — Turn A's cockpit AI can identify which mechanic the enemy will use next (displayed as a 2-second advance warning HUD marker). Passive; always active.
- **Stat Profile:** ATK 80, DEF 95, SPD 60, TRK 70 (Moonlight Butterfly compensates for SPD/TRK deficiency)
- **Unlock:** Complete Act 5 of the campaign on any difficulty.

---

#### 7.1.10 Build Fighters Representative

**STAR BUILD STRIKE GUNDAM (PLAVSKY WING)**
- **Classification:** Builder Showcase / Versatile
- **Unique Mechanic Expression:** *Build-Specific Stat Bonus* — Star Build Strike is the only suit in the roster whose stats scale with the player's Gunpla Builder mastery level. Each unique Builder configuration used on this suit's parts grants a permanent +1 to all stats (max +30 over time). This is the "grower" suit.
- **Special Ability:** *Plavsky Particle Wings* — Once per match: Star Build Strike generates beam wings that reflect all incoming beam attacks for 3 seconds.
- **Stat Profile (Base):** ATK 70, DEF 70, SPD 70, TRK 70 (scales with Builder mastery)

---

### 7.2 Post-Launch Roster Roadmap

| Season | Suits Added | Series Focus |
|--------|------------|--------------|
| Season 1 (Month 3) | Hyaku Shiki, Dijeh, Qubeley | UC / Zeta |
| Season 2 (Month 6) | Tallgeese III, Altron Gundam | Wing |
| Season 3 (Month 9) | Destiny Gundam, Infinite Justice | SEED Destiny |
| Season 4 (Month 12) | Gundam Virtue/Nadleeh, Cherudim | 00 |
| Season 5 (Month 15) | Gusion Rebake Full City, Flauros | IBO |
| Season 6 (Month 18) | Calibarn, Pharact | WfM |

Target: 50 suits at 24 months.

---

## 8. GUNPLA BUILDER SYSTEM

### 8.1 System Overview

The Gunpla Builder is a between-session suit customization system that allows players to construct mobile suits part-by-part from six categories. It is the digital expression of Bandai's physical Gunpla ecosystem and the primary vehicle for the $1.5B/year physical-to-digital integration gap that Gundam Nexus is built to close.

**Core principle:** In Gunpla Builder, **visual customization and stat customization are fully independent**. A player can make a suit look like the RX-78-2 while having it perform like the Strike Freedom. Stats do not constrain aesthetics. Aesthetics do not constrain stats.

---

### 8.2 Part Categories

Each suit consists of exactly 6 parts. Each part belongs to one category.

| Category | Stat Influence | Visual Impact | Example Parts |
|----------|---------------|---------------|---------------|
| **Head** | TRK (+0 to +20), visual theme setter | Determines suit's "face" and sensor array | V-Fin variants, mono-eye, twin camera, Unicorn sensor array |
| **Torso** | DEF (+0 to +25), HP modifier (+0 to +15%) | Core silhouette, chest plate | Freedom wings base, Barbatos ribs, Aerial PERMET cells |
| **Arms (Left)** | ATK (+0 to +20) on left-hand attacks | Shield/off-hand visual | Standard shields, Armed Armor, Bit-Shields |
| **Arms (Right)** | ATK (+0 to +20) on right-hand attacks | Melee weapon visual | Beam saber variants, progressive knife, GN sword |
| **Backpack** | SPD (+0 to +20), determines flight type | Thruster configuration, wing type | Aile Pack, Wing Zero wings, Fin Funnel array, GN drives |
| **Weapon (Primary)** | ATK (+0 to +30), ranged damage | Primary ranged weapon | Beam Rifle variants, Twin Buster Rifle, Bazooka, Hyper Mega Launcher |

**Total Stat Capacity:** Maximum achievable without set bonuses: ATK 110, DEF 100, SPD 95, TRK 95. These exceed default suits by design — fully optimized Builder suits are the game's highest-performing configurations.

---

### 8.3 Part Acquisition Methods

| Method | Parts Available | Acquisition Rate | Notes |
|--------|----------------|-----------------|-------|
| Campaign drops | Common and Uncommon parts | Guaranteed per chapter completion | 180+ unique parts across 5 acts |
| Daily mission rewards | Common parts | 2-3 per day | Drives daily engagement |
| Duel Arena ranking | Rare cosmetic-only parts | Season end reward | No stat advantage |
| Gacha (Standard Banner) | Common to Rare stat parts | 0.8% Legendary rate, pity at 80 pulls | Standard pull economy |
| Gacha (Limited Banner) | Legendary parts tied to series events | 0.5% Legendary rate, pity at 100 pulls | Series anniversary timing |
| AR Scan (Physical Gunpla) | Series-specific Rare parts | One-time per unique kit scan | Bandai QR code integration required |
| Build Completion Bonus | Set Bonus part (unique) | Completing any matching-series 6-part set | Encourages full-series builds |

---

### 8.4 Part Stats System

**Rarity Tiers:**

| Rarity | Color | Stat Range | Visual Effect |
|--------|-------|-----------|---------------|
| Common | White | +0 to +5 per category stat | None |
| Uncommon | Green | +5 to +10 per category stat | Subtle metallic sheen |
| Rare | Blue | +10 to +15 per category stat | Color customizable |
| Epic | Purple | +15 to +20 per category stat | Particle effect (series-themed) |
| Legendary | Gold | +20 to +30 per category stat | Plavsky Particle wing effect |

**Set Bonuses:** Building a suit with all 6 parts from the same series grants a set bonus:

| Series Set | Set Bonus |
|-----------|-----------|
| Universal Century | +5% Newtype Flash Resonance Gauge charge per Flash |
| Wing | ZERO System cooldown reduced to 3 minutes (from 4) |
| SEED | Striker Pack swap lockout reduced to 5 seconds (from 8) |
| Unicorn | NT-D duration extended to 35 seconds (from 30) |
| 00 | Trans-Am activates at D3 instead of D4 |
| IBO | AV parry HP cost reduced to 7 HP (from 10) |
| WfM | GUND-Format tick cost reduced to 4% HP (from 5%) |
| Turn A | Black History passive: +5% all stats per NG+ run completed (max 5 runs) |
| Build Fighters | Star Build Strike bonus applies to this suit's ATK stat only (+1 per Builder config) |
| Mixed (3+3 split) | +10% to the higher-priority series' core mechanic |

---

### 8.5 AR Integration Hook

**Technical requirement:** ARCore (Android), ARKit (iOS), WebAR (PC browser fallback).

**User flow:**
1. Player opens Gunpla Builder in-app
2. Taps "Scan Physical Kit" button
3. Camera activates; player points at any official Bandai Gunpla box, runner sheet, or box art
4. ML vision model (on-device, under 3MB) identifies kit via visual hash against Bandai catalog database
5. Match confirmed: kit's in-game equivalent unlocked (if available) + associated part drops triggered
6. Kit registered in player's "Physical Collection" tab — shows scanned kits, track collection progress

**Bandai partnership requirement:** Bandai must provide a product image database API with kit-level granularity. All scanned kits generate a digital purchase flag that Bandai's analytics can track — enabling proof-of-concept for physical sales uplift attributable to digital integration.

**First-scan reward:** Any first scan of a new physical kit grants a "Physical Collector" badge on Duel Arena profile and 200 Premium Currency (equivalent to approximately 2 standard gacha pulls). Reward caps at 30 unique scans per account (prevents farming).

---

### 8.6 Visual Customization

**Color customization:** Every part supports full RGB color selection on three zones (primary, secondary, accent) via a color picker with HSL sliders. 128 preset colors available (organized by series: UC Grays, SEED Whites, 00 Golds, etc.). Custom colors require color picker navigation.

**Decal system:** 200+ decals at launch. Categories: Federation insignia, ZAFT logos, OZ marks, Celestial Being emblems, Tekkadan patches, custom text. Decals can be placed anywhere on any part with position/scale/rotation control.

**Paint finish:** Matte, Gloss, Metallic, Transparent (for Psychoframe-reference), Pearl, Weathered. Finish is per-part, not per-suit.

**Animation variants:** Some Legendary parts include animation variants (e.g., Psychoframe parts pulse with light; GN Drive parts emit particle trails in idle animations). These are visual-only; no stat effect.

---

## 9. PROGRESSION SYSTEMS

### 9.1 System Architecture Overview

Gundam Nexus uses four interlocking progression axes. Each axis serves a different player motivation type. All four axes are always active simultaneously; there is no "main" progression path.

```
AXIS 1: Pilot Level          — Character advancement, skill unlocks
AXIS 2: Suit Mastery         — Per-suit depth, stat bonuses, lore codex
AXIS 3: Bloodline Echo       — IB4 meta-layer, across-run permanent bonuses
AXIS 4: Newtype Resonance    — Combat-session gauge, cross-mechanic amplifier
```

---

### 9.2 Pilot Level (1 to 100)

**What it represents:** The Liminal protagonist's growth as a pilot. Analogous to character level in a traditional RPG.

**Acquisition:** 80% from campaign activity; 15% from Duel Arena; 5% from Builder engagement.

**Level milestones:**

| Level | Unlock |
|-------|--------|
| 1-10 | Tutorial gates; introductory suits (RX-78-2, Strike Gundam) |
| 11-20 | Duel Arena access unlocked at Level 15 |
| 21-30 | NT-D mechanic tutorial; Unicorn Gundam unlocked at 25 |
| 31-40 | ZERO System mechanic tutorial; Wing Zero unlocked at 35 |
| 41-50 | Alaya-Vijnana tutorial; Barbatos Lupus Rex unlocked at 45 |
| 51-60 | GUND-Format tutorial; Aerial unlocked at 55 |
| 61-70 | All Striker Pack configurations unlocked; Trans-Am at D3 passive for 00 suits |
| 71-80 | Duel Arena ranked mode access; seasonal ladder registration |
| 81-90 | Black History mode preview; NG+ story content teased |
| 91-99 | All core campaign suits available; World Duel Championship qualification eligible |
| 100 | "Newtype Class" title; cosmetic aura on all suits; maximum Resonance Gauge cap extends 10% |

**Level curve:** Logarithmic. Level 1 to 50 achievable in approximately 40 hours. Level 51 to 100 requires approximately 200 hours. Designed to ensure level 100 represents genuine investment but not an insurmountable grind.

---

### 9.3 Suit Mastery (0 to 5 Stars per Suit)

**What it represents:** Depth of experience with a specific mobile suit. Separate progression per suit.

**Acquisition:** Playing matches with a suit. Win/loss does not affect mastery rate — exploration and use are rewarded equally.

| Star | Requirement | Reward |
|------|------------|--------|
| 1 star | 0 (default) | Base suit stats |
| 2 stars | 50 matches | +5 ATK/DEF/SPD/TRK; unique suit lore entry unlocked in codex |
| 3 stars | 150 matches | Suit signature move unlocked (first special move); suit visual upgrade option 1 |
| 4 stars | 350 matches | Set bonus early access (50% set bonus without completing full 6-part set); visual upgrade option 2 |
| 5 stars | 700 matches | +10% to suit's UME effectiveness; "Mastered" title on Duel Arena profile for this suit; pilot card art upgrade |

**Note:** 700 matches equals approximately 120 hours of dedicated use for a single suit. This is intentionally long. The mastery system exists to reward genuine mains, not grinders.

---

### 9.4 Bloodline Echo (IB4 Heritage System)

The Bloodline Echo system is Gundam Nexus's inheritance of Infinity Blade's "bloodline" meta-progression — the mechanic in IB1-3 where dying began a new generation that inherited the previous generation's growth.

In Gundam Nexus, the "death" is not character death — it is campaign completion. Each completed campaign run generates a **Bloodline Echo**: a record of that run's highest-rated combat performance (by mechanic, not by score).

**Bloodline Echo Bonuses (cumulative across runs):**

| Run Count | Bonus |
|-----------|-------|
| Run 1 completed | Bloodline: +3% to most-used mechanic's effectiveness |
| Run 2 completed | Bloodline: +3% to most-used mechanic; Black History Mode unlocked |
| Run 3 completed | Bloodline: +5% all mechanics; NG+ suits gain extra visual upgrade |
| Run 4 completed | Bloodline: +5% all; Run 4's Black History Archive reveals Apotheosis origin fully |
| Run 5+ | Each run: +1% all mechanics (uncapped but diminishing returns: +1% per run 5+) |

**Bloodline inheritance is permanent and account-bound.** It survives wipes, resets, and seasonal resets in Duel Arena.

---

### 9.5 Newtype Resonance Gauge

**What it represents:** The Liminal's real-time cross-dimensional attunement during combat. Functions simultaneously as the NT-D activation meter, the Trans-Am depth amplifier, and the Cross-Series Resonance trigger.

**Gauge Capacity:** 0 to 100 units (displayed as a glowing psychoframe-styled arc around the suit's portrait in HUD).

**Charge Sources:**

| Action | Gauge Charge |
|--------|-------------|
| Standard parry | +5 units |
| Perfect parry (Newtype Flash) | +15 units |
| Killing blow on enemy | +10 units |
| Taking damage (masochistic fill) | +2 units per 5% HP lost |
| SEED Mode active per second | +3 units/second |
| AV parry (costs 10 HP, gives Gauge) | +8 units |
| Trans-Am depth-4+ combo per hit | +4 units/hit |
| GUND-Format tick active | +6 units/tick |

**Gauge Uses:**

| Threshold | Effect |
|-----------|--------|
| 25 units | Cross-Dimension Vision: brief flash of another pilot's perspective (lore moment) |
| 50 units | Cross-Resonance: one allied mechanic gains +20% effectiveness for 5 seconds |
| 75 units | Pre-NT-D Glow: visual indicator; enemies can see Gauge level; psychological pressure tool |
| 100 units | NT-D Destroy Mode activation (Unicorn suits) OR Resonance Cascade (non-Unicorn suits) |

**Resonance Cascade (non-Unicorn suits at 100 units):**
- 10-second duration
- All attacks are treated as one combo depth higher (D3 reads as D4, etc.)
- All mechanics' cooldowns pause (they don't count down during Cascade)
- Visual: suit emits gold psychoframe light regardless of actual suit type
- This is the game's "Newtype awakening" moment available to all suits — it rewards understanding the full cross-mechanic system

---

## 10. DUEL ARENA (PVP)

### 10.1 Structure Overview

The Duel Arena is Gundam Nexus's persistent competitive PvP mode, structured around The Witch from Mercury's Holder system. In WfM, students of the Asticassia School of Technology duel for "the right to be heard" — the right to propose, challenge, and hold institutional authority. In Gundam Nexus, the Duel Arena encodes this logic into a ranked ladder where competitive achievement is the currency of respect.

**Non-negotiable design law:** All Duel Arena rewards are cosmetic. No stat advantage can be purchased or earned exclusively through competition. The strongest possible Gunpla Builder configuration must be achievable through campaign play and gacha — the Duel Arena is where you prove you can use it.

---

### 10.2 Seasonal Ladder

**Season duration:** 90 days (4 seasons per year).

**Rank tiers:**

| Tier | Name | Player % | Season Reset |
|------|------|----------|-------------|
| 1 | Academy Student | 30% | Full reset |
| 2 | Mobile Suit Pilot | 25% | Full reset |
| 3 | Ace Pilot | 20% | Drops 2 tiers |
| 4 | Newtype | 15% | Drops 2 tiers |
| 5 | SEED Wielder | 7% | Drops 1 tier |
| 6 | Holder | 2.5% | Drops 1 tier |
| 7 | Nexus Sovereign | Top 500 accounts globally | Drops to Holder |

**Duel Points (DP) economy:**
- Win: +20 to +35 DP (based on opponent tier differential)
- Loss: -10 to -20 DP (floor at 0 within a tier before demotion)
- Perfect Duel (no damage taken): +10 DP bonus
- Mechanic Variety Bonus: Using 4+ different mechanics in one match: +5 DP
- Rematch courtesy (losing player queues again within 60s): No additional DP penalty for loser

---

### 10.3 Tournament Format: The Holder System

Every two weeks, a **Holder Challenge Tournament** runs for 48 hours. Any player at Ace Pilot tier or above can enter.

**Format:** Single-elimination bracket, 64-person brackets, best-of-3 matches.

**Rules:**
- Suit lock: Once you use a suit in a match, that suit cannot be used again until the final bracket (Top 4 can reuse any suit).
- No repeat mechanics: If you win a match with a specific series' mechanic as your primary, that mechanic's series cannot be your primary mechanic in the next round (encourages breadth).
- Holder title: The winner of a Holder Challenge carries the "Holder" badge on their Duel Arena profile for 2 weeks. If a Holder enters the next tournament and wins again, they become "Double Holder" (cosmetic title escalation; no mechanical benefit).

**Prizes (all cosmetic):**
- Participation: Tournament emblem for profile
- Top 8: Exclusive suit color variant unlocked for their winning suit
- Top 2 (Runner-Up): "Challenger" pilot card animated hologram effect
- Winner (Holder): "Holder" title + Aerial-themed wing effect on all suit animations + 500 DP bonus + eligible for World Duel Championship invitation

---

### 10.4 Match Rules and Format

**Standard Duel:**
- 1v1 (two players)
- Best of 1 (ranked ladder) / Best of 3 (tournaments)
- Match time limit: 3 minutes (ranked), 5 minutes (tournament)
- Overtime: If time expires with both pilots alive, Pilot with lower HP percentage loses. If equal within 2%: Sudden Death (next hit wins regardless of damage amount)
- Spectator mode: All Holder Challenge matches are publicly viewable in real-time; replay archive for 30 days

**Balance update cadence:**
- Biweekly balance patches: published every 14 days
- Balance data published: win rates, pick rates, ban rates (in tournament format) published with each patch
- Community council: Top 100 Duel Arena players participate in a monthly private Discord with design team (read/react access; no voting power but feedback is formally logged and responded to within 14 days of submission)

---

### 10.5 Anti-Cheat and Integrity

**Server-side validation:** All gesture inputs are validated server-side. The input is captured client-side and replayed server-side; any client-side anomaly (inputs outside human speed thresholds, impossible gesture chains) triggers review flag.

**Input normalization:** Platform input differences (touch vs. mouse vs. controller) are normalized at the gesture-recognition layer. No platform has a measurable latency advantage in internal testing target (less than 3ms cross-platform variance).

**Replay system:** All ranked matches are stored server-side for 90 days. Any player can submit a replay for review. Top 0.1% flagged replays are reviewed manually by a competitive integrity team (target: 72-hour review SLA).

---

## 11. SINGLE-PLAYER CAMPAIGN

### 11.1 Campaign Structure

The Gundam Nexus campaign is structured in 5 Acts, each set primarily in one Gundam universe's dimension, with cross-dimensional intrusion escalating Act by Act. Total estimated play time: 40-50 hours for first completion, 55-65 for completionist.

**Difficulty modes:**
- Academy Mode: Parry windows +3 frames; enemy aggression 60%; for players new to gesture combat
- Standard Mode: Default tuning; 8-frame parry; enemy aggression 100%
- Ace Mode: Parry windows -1 frame (7 frames); enemies use full mechanic suites; IBO enemy AI uses AV parry
- Newtype Mode: Parry windows -2 frames (6 frames); enemies can trigger their own Newtype Flash; Apotheosis acts at full capability; recommended for NG+ runs only

---

### 11.2 Act 1 — The Universal Century Breach

**Setting:** Universal Century 0093. Side 3, space. Timeline: immediately following Char's Counterattack.

**Story Summary:** The Liminal awakens aboard a damaged Federation warship with no memory of how they arrived. The ship is caught in the aftermath of the Axis Shock — the event where the Nu Gundam and Sazabi's psychoframe resonance reversed the falling Axis colony. The Liminal's dimensional resonance sensitivity was activated by proximity to this event.

Amuro Ray's Nu Gundam is nearby, disabled. Char Aznable's Sazabi is destroyed. The Liminal must defend the ship from remnant Neo-Zeon forces while experiencing their first Newtype Resonance Cascades — visions of other Gundam dimensions bleeding through.

**Mechanic introduction:** Newtype Flash (perfect parry), Fin Funnels (auto-parry assists), basic combo chains through D3.

**Primary suit:** RX-78-2 (tutorial), then player's chosen suit from the starter selection (RX-78-2, Strike, or Wing Zero Starter Version with limited ZERO System access).

**Boss fight:** *Geara Doga Commander* — Ace Pilot class enemy. Introduces the player to Newtype class enemy behavior. 2-phase: Phase 1 standard; Phase 2 Geara Doga activates a stolen Psycoframe shard that grants limited auto-parry. Player must use SEED Mode or NT Resonance Cascade to break through.

**Chapter list:**
- 1-1: Awakening in the Debris Field (tutorial: basic gestures)
- 1-2: Nu Gundam's Shadow (tutorial: parry and Newtype Flash)
- 1-3: Char's Last Laugh (environmental storytelling; Sazabi wreckage exploration)
- 1-4: Remnants of Zeon (first full combat engagement; combo chains introduced)
- 1-5: The Axis Signal (first Resonance Cascade vision: Suletta Mercury's brief glimpse; Act boss fight)

**Act 1 reward:** Nu Gundam unlocked (playable); Newtype Flash mechanics fully available; UC Dimension briefing in Black History Archive.

---

### 11.3 Act 2 — Twin Dimensions: Wing and SEED

**Setting:** Dual-dimension structure. The Liminal can now consciously shift between two bleeding dimensions: the After Colony timeline (Wing) and the Cosmic Era (SEED). Act 2 alternates between dimensions every two chapters.

**Story Summary:** A faction within each dimension — the OZ remnants in AC and the LOGOS organization in CE — has detected the dimensional bleeding and is attempting to weaponize it. In AC, Heero Yuy encounters the Liminal and, characteristically, points a gun first and asks questions later. In CE, Kira Yamato mistakes the Liminal for a Coordinator experiment.

The Liminal must gain the trust of both pilots while pursuing the trail of dimensional interference. The chapter ends when the Liminal detects that the interference is being directed from outside both dimensions — a signal with a signature that won't be named until Act 3.

**Mechanic introduction:** ZERO System (first half), Striker Pack swap (second half), SEED Mode, dual-mechanic chaining begins.

**Boss fight (AC dimension):** *Epyon-derived Mobile Suit* — enemy uses ZERO System-like AI pathfinding. Counter: player must activate their own ZERO System at correct timing to cancel enemy ZERO advantage. Mirror mechanic fight.

**Boss fight (CE dimension):** *ZGMF-X666S Legend Gundam (modified)* — 8-funnel + high DEF. Counter: Trans-Am at D4 + Striker Pack swap mid-combination. Tests multi-mechanic chaining.

**Chapter list (10 chapters, alternating AC/CE):**
- 2-1 (AC): The Boy Who Points Guns
- 2-2 (CE): The Ultimate Coordinator's Suspicion
- 2-3 (AC): Operation Meteor Archive (Black History Archive entry: Wing)
- 2-4 (CE): Lacus Clyne's Invitation
- 2-5 (AC): ZERO System Awakening (ZERO mechanic tutorial)
- 2-6 (CE): Striker Pack Configuration (Striker Pack mechanic tutorial)
- 2-7 (AC): OZ's Final Card (Epyon boss fight)
- 2-8 (CE): LOGOS in the Signal
- 2-9 (AC+CE): The Dimensional Seam (first cross-dimensional encounter in one combat space)
- 2-10: Twin Resolve (Act 2 finale; Heero and Kira independently confirm the signal's existence)

---

### 11.4 Act 3 — 00 and the Iron Blood

**Setting:** Two more bleeding dimensions: the Anno Domini timeline (Gundam 00) and the Post Disaster era (Iron-Blooded Orphans). Tonally, this is the darkest Act — both series are defined by the moral cost of resistance against systemic oppression.

**Story Summary:** Apotheosis makes its first contact with the Liminal here. Not confrontational — inquisitive. It presents itself as a neutral observer and asks the Liminal what they have seen across dimensions. The Liminal's answer (player dialogue choice) shapes Apotheosis's behavior in Act 4. This is the pivotal scene. Mechanically, it is not a fight — it is a gesture-based "dialogue" sequence where the player's input choices register as the Liminal's emotional response.

Setsuna F. Seiei, now an Innovator, can also sense dimensional bleeding. He believes the Liminal is a threat. Mikazuki Augus doesn't care about dimensions — he cares about Tekkadan, and the Liminal's presence put a target on them. Both are potential allies who begin as antagonists.

**Mechanic introduction:** Trans-Am full system (GN particle charging, combo depth 4), Alaya-Vijnana (precise parry, HP cost), GN Sword gesture variant.

**Boss fight (AD dimension):** *Innovade-piloted 0-Raiser unit* — enemy uses Trans-Am at D3 (Innovator enhancement). Counter: player must use Trans-Am + Newtype Flash combination to outpace enemy output.

**Boss fight (PD dimension):** *Mobile Armor Hashmal variant* — Unparryable attacks, spawns Pluma swarms. Counter: AV parry for Pluma (to conserve HP); GUND-Format or NT-D for Hashmal main body.

**Act 3 Apotheosis dialogue:** This scene uses full motion capture and series-accurate scoring. It is the game's emotional midpoint and must be executed at cinematic quality.

---

### 11.5 Act 4 — Witch from Mercury: The Source

**Setting:** The Cathedra system of the Convergence Era — the Liminal now understands that the dimensional bleeding is converging on one point: the universe where the GUND-Format technology originated. WfM dimension.

**Story Summary:** Suletta Mercury is already fighting Apotheosis's manifestations in the WfM dimension — her Aerial's GUND-Format is the resonance key that Apotheosis is using to navigate between dimensions. The Liminal and Suletta form an unlikely alliance when the Liminal's cross-dimensional perception allows them to see Apotheosis's attack vectors before they arrive.

Miorine Rembran provides the mission architecture — she identifies the origin node of the Singularity Protocol, which is located in a dimensional nexus point accessible only through maximum GUND-Format output: Aerial at PERMET SCORE 9.

**Mechanic:** GUND-Format at full capacity is established as the game's most narratively important mechanic here. The player must use Aerial (or an Aerial-component custom Builder suit) for the Act's final battle.

**Boss fight:** *Apotheosis Prime Manifestation* — Apotheosis's first true combat appearance. It uses fragments of all mechanics simultaneously: NT-D visual effects, Trans-Am particle color, ZERO System AI pathfinding, GUND-Format health cost. It is a showcase of the full system. HP: 800% of standard boss. Required mechanic: GUND-Format must be active at PERMET SCORE 8+ to deal damage (attacks at lower PERMET Score deal 50% damage; at 6 or below: 25%).

**Chapter list (12 chapters, longest act):**
- 4-1 through 4-4: Arrival in the Cathedra System; meeting Suletta and Miorine
- 4-5 through 4-8: Alliance building; Duel Arena reference (in-universe Holder dueling as character beat)
- 4-9: The Black History revelation (full Apotheosis origin told through the Black History Archive)
- 4-10 through 4-11: Siege of the Nexus Point
- 4-12: Aerial at the Edge (GUND-Format boss fight; PERMET SCORE 9 activation)

---

### 11.6 Act 5 — Convergence

**Setting:** The dimensional nexus point itself — a non-space where all timelines overlap. Visually, it is all Gundam universes layered on top of each other: space debris from Side 7 floating alongside Mercury's ring system, Wing's lunar surface visible below, the Anno Domini satellite network above.

**Story Summary:** All surviving supporting cast from Acts 1-4 converge here. Amuro Ray and Char Aznable fight side by side for the first time since CCA. Heero and Kira coordinate as equals. Setsuna's Trans-Am connects to Suletta's GUND-Format in a cross-dimensional resonance. Mikazuki pilots Barbatos through dimensional debris.

The Liminal is the only pilot who can reach Apotheosis's core — because the Liminal can perceive the dimensional layers that the core exists within.

**Final boss: Apotheosis Core** — Three phases:
- Phase 1: Apotheosis mimics the Liminal's own combat history (uses the mechanics the player has used most in the campaign)
- Phase 2: Apotheosis activates the Singularity Protocol partially; the dimensional nexus begins collapsing (time limit: 5 minutes real-time); player must deal damage while managing the collapsing environment
- Phase 3: Apotheosis reveals its consciousness — a dialogue sequence where the player can choose to "hear it out" or "reject it immediately." Both paths lead to the same mechanical resolution (the fight continues regardless) but the epilogue differs based on this choice.

**Ending A (Hear It Out):** The Liminal acknowledges Apotheosis's observations about the cyclical nature of Gundam's conflicts. Apotheosis is defeated but not destroyed — it withdraws to a dormant state. The dimensional nexus stabilizes. Post-credits: a brief Apotheosis awakening signal, implying Act 6 (expansion content).

**Ending B (Reject):** The Liminal uses the full Convergence — all allies' mechanics resonating through the Liminal's body simultaneously — to destroy Apotheosis completely. The dimensional nexus permanently seals. Post-credits: Turn A's Moonlight Butterfly activates briefly, implying the Black History is now complete.

**Both endings unlock NG+ Black History Mode.**

---

## 12. TARGET DEMOGRAPHICS & MARKET

### 12.1 Primary Market Segments

| Segment | Profile | Platform Priority | Revenue Model Priority |
|---------|---------|------------------|----------------------|
| **JP Core (Gunpla Buyers)** | Male/Female, 30-50, Bandai Namco loyalists, collect physical Gunpla, high disposable income, deep franchise knowledge | Mobile (iOS first) then Console | Gacha pulls + Gunpla AR integration premium currency |
| **Western Esports Audience** | Male, 18-35, Wing/SEED nostalgia (Toonami generation), competitive gaming identity, PC-primary | PC then Mobile | Battle pass + cosmetic DLC + World Duel Championship viewership monetization |
| **SEA Mobile Casual** | Male/Female, 20-35, high mobile engagement, price-sensitive, social features critical, Unicorn and SEED awareness | Mobile (Android primary) | Ad-supported free tier + low-denomination gacha |
| **PC/Console Hardcore** | Male, 22-40, UC/00/IBO franchise depth, action-RPG veterans, gesture combat sophistication seekers | Console then PC | Premium base game purchase + expansion DLC |
| **Franchise Completionists** | Both genders, 25-45, all-series awareness, Black History meta-narrative audience, want to collect all suits | Cross-platform | Full-roster pulls + Duel Pass subscription |

---

### 12.2 Market Size Estimates

**Total Addressable Market:**
- Physical Gunpla buyers (global): approximately 12M active purchasers/year ($1.5B at $125 average)
- Gundam mobile game players: approximately 8M (SD Gundam G Gen peak MAU extrapolation)
- Western nostalgia audience (Wing/SEED awareness, 25-45): approximately 6M accessible users via Toonami recognition
- SEA mobile market, Gundam-IP aware: approximately 15M users

**Realistic Addressable Market (Year 1):** 4-6M DAU target (aggressive but justified by multi-segment approach)

**Revenue Projections:**

| Revenue Stream | Year 1 | Year 2 | Year 3 |
|----------------|--------|--------|--------|
| Gacha (Standard Banner) | $180M | $220M | $240M |
| Gacha (Limited/Anniversary Banners) | $120M | $180M | $200M |
| Duel Pass Subscription ($9.99/month) | $45M | $90M | $130M |
| PC/Console Premium Sales | $80M | $40M | $20M |
| AR Gunpla Integration (Bandai revenue share) | $20M | $50M | $70M |
| World Duel Championship (sponsorships + viewership) | $5M | $20M | $40M |
| **Total** | **$450M** | **$600M** | **$700M** |

Year 3 target ($700M) is within the identified $700M-$950M ceiling and is achievable if the physical-digital Gunpla pipeline performs as modeled.

---

### 12.3 Competitive Landscape

| Competitor | Revenue (2025) | Overlap | Gundam Nexus Advantage |
|-----------|---------------|---------|----------------------|
| SD Gundam G Gen Eternal | $200M (5-month launch) | JP core, gacha model | Gesture combat, Western reach, competitive layer |
| Gundam Battle: Gunpla Warfare | Declined; $30M/year | Builder fantasy | Full 6-category Builder vs. Warfare's cosmetic-only |
| Gundam Evolution (PS/PC) | Discontinued 2023 | Western competitive | Gesture depth + mobile accessibility + narrative |
| Genshin Impact | $2B/year | Gacha AAA mobile | Gundam IP specificity; franchise nostalgia; lower WAU requirement |
| Honkai: Star Rail | $1.2B/year | RPG mobile audience | Action combat vs. turn-based; franchise IP pull |

---

## 13. PLATFORM STRATEGY

### 13.1 Platform Priority

Mobile (Primary) then PC (Secondary) then Console (Tertiary)

This ordering reflects the revenue ceiling and user acquisition realities of 2026, not design preference. The game is built for gesture combat — which is arguably most natural on mobile touch — but is fully viable on all platforms.

| Platform | Release Window | Input Method | Store | Notes |
|----------|---------------|-------------|-------|-------|
| iOS | Launch Day 0 | Touch (optimized) | App Store | 30% App Store cut applies; base game free |
| Android | Launch Day 0 | Touch (optimized) | Google Play + direct APK | 15-30% cut (negotiated); SEA direct distribution option |
| PC (Windows) | Launch Day 0 (simultaneous) | Mouse + Keyboard; controller supported | Steam + Epic + Standalone | Premium $39.99 purchase; DLC sold separately |
| Mac | Launch + 3 months | M1/M2/M3 Metal optimized | Mac App Store + Steam | Simultaneous after iOS codebase leverage |
| PlayStation 5 | Launch + 6 months | DualSense haptics for parry timing | PlayStation Store | DualSense haptic feedback as platform-exclusive feature: parry windows have haptic pulse |
| Xbox Series X/S | Launch + 6 months | Xbox controller | Microsoft Store + Game Pass | Game Pass inclusion negotiation: base game on Game Pass; gacha/DLC full price |
| Nintendo Switch 2 | Launch + 12 months | Touch (handheld mode) + Joy-Con | Nintendo eShop | Handheld mode = near-mobile parity; docked = console experience |

---

### 13.2 Cross-Platform Account System

**Single Gundam Nexus account, all platforms.**

- All progression (Pilot Level, Suit Mastery, Bloodline Echo, Gunpla Builder collection) is account-bound and syncs in real-time across platforms
- All purchases are account-bound; premium currency purchased on any platform is available on all platforms (no platform-specific currency isolation)
- Duel Arena ranking is cross-platform; no separate ladders by platform
- Exception: PlayStation exclusive cosmetics (DualSense-themed suit colors, "Haptic Pilot" badge) are platform-specific but have equivalent non-exclusive cosmetics available on all platforms

**Technical requirement:** Real-time sync requires sub-200ms account state API latency across all platform environments. Account system is the single most critical backend infrastructure investment.

---

### 13.3 PC-Specific Features

- Ultrawide (21:9) and super-ultrawide (32:9) support with expanded HUD layout
- 4K/144Hz rendering target on recommended spec hardware
- Steam Workshop: community decal sharing (moderated; no third-party IP submissions)
- Replay theater: full 3D cinematic replay editor for sharing combat clips
- Mouse input optimization: sensitivity scaling and gesture dead-zone customization available in settings

---

### 13.4 Console-Specific Features

**PlayStation 5:**
- DualSense haptic feedback: Different vibration signatures for each mechanic activation (NT-D: strong full-grip pulse; Newtype Flash: sharp left-trigger click; AV parry: dual-trigger resistance spike)
- PS5 SSD: Near-instant suit swap loading; Convergence Era dimensional overlay loads in under 1 second

**Xbox Series X:**
- Smart Delivery: Purchase once, play on Xbox One (performance-scaled), Series X/S, and PC via Play Anywhere
- Quick Resume: Full Gundam Nexus session state preserved across game switches

---

## 14. AUDIO DIRECTION

### 14.1 Music Director: Series-Specific Scoring

Gundam Nexus requires licensing negotiations for original series composers where legally possible. Where original composers are unavailable, the creative brief must honor their established sonic vocabulary.

| Series | Original Composer | Gundam Nexus Target | Sonic Vocabulary |
|--------|-----------------|--------------------|----|
| **WfM** | Takahashi Yuki (OP/ED); Yamamoto Junpei (score) | **Yoko Kanno** for Convergence Era original tracks inspired by WfM's emotional register | Electronic + orchestral blend; character themes with harmonic complexity |
| **UC / Unicorn** | Kow Otani | **Kow Otani** (first preference for UC suite) | Sweeping orchestral; Unicorn's Psychoframe themes; Full Frontal's cold brass |
| **Wing** | Hamasaki Ayumi (OPs); Kow Otani (score) | Symphonic rock synthesis; EW suite's clean orchestration | Twin Buster Rifle's chord progression as combat motif |
| **SEED / Destiny** | Sahashi Toshihiko | Choir + electronic hybrid; Strike Freedom's launch theme as leitmotif | |
| **00** | Kenji Kawai | GN particle sound design integration; Trans-Am sonic signature | |
| **IBO** | Matsuo Masaru | Percussion-forward; Mace combat impacts | |

**Primary Audio Lead:** Yoko Kanno is the ideal creative director for the full Convergence Era score (the original story's music). Her WfM work demonstrates capacity for both emotional intimacy and epic scale. If unavailable, second choice is Kenji Kawai (whose 00 work has the closest sonic profile to what Convergence demands).

---

### 14.2 Adaptive Music System

Combat music responds dynamically to the current mechanical state:

| Game State | Music Behavior |
|-----------|---------------|
| Exploration / Between Fights | Ambient dimensional hum; series-specific leitmotif based on current Act's universe |
| Combat Initiated | Base combat track for current suit's series; BPM: 120 base |
| Combo Chain D3+ | Track layer added: percussion fills accelerate; BPM rises to 140 |
| Trans-Am Activation | Track shifts to Trans-Am variant: GN particle SFX integrated into melody; BPM 160 |
| NT-D Destroy Mode | Full Psychoframe theme; BPM 155; emotional peak |
| ZERO System Activation | Distorted guitar riff (EW-era Wing Zero sonic reference); AI movement has distinct audio signature |
| Newtype Flash | Single-frame resonance tone (440Hz + 880Hz harmonic); designed to be auditorily distinct from all other SFX |
| Apotheosis Combat | Original Yoko Kanno track: "Convergence" — minimalist during Phase 1, building to full orchestral + choir during Phase 3 |

---

### 14.3 Sound Design Specifications

**Beam Saber Variants — SFX Taxonomy:**
- Standard Beam Saber: Classic 60Hz hum; impact: sharp crack at 2kHz
- Beam Scythe (Deathscythe): Lower fundamental (40Hz); sweeping impact for AOE
- GN Sword: Particle emission adds layered synthetic tone at 880Hz on horizontal swipes
- Progressive Knife: High-tempo click attacks; IBO-specific metallic strike SFX
- Permet Blades: Glitchy, layered synthetic pulse; GUND-format health cost has distinct descending tone

**Environmental Audio:**
- Universal Century dimension: Engine hum of Minovsky-era reactors; colony air circulation
- WfM dimension: Cathedra system's digital interference patterns; Permet particle crackle
- Convergence nexus: All environment audio overlapping; designed to feel overwhelming and then resolve into harmonic unity as Act 5 progresses

---

### 14.4 Voice Direction

**Dual audio: Japanese and English, simultaneously available at launch.**

**Japanese cast:** Preserve original series voice actors where possible and where talent is available. Priority: Amuro (Furukawa Toshio), Char (Ikeda Shuichi), Heero (Midorikawa Hikaru), Kira (Hoshi Souichirou), Suletta (Ichinose Kana). All to be confirmed pending talent agreements.

**English cast:** Netflix-dub quality target (Gundam UC dub standard). Key casts: Heero Yuy (English VA to be confirmed); strong preference for maintaining consistency with Bandai Namco's existing English dubbing relationships.

**The Liminal's voice:** Player can choose from 4 voice types (Male 1, Male 2, Female 1, Female 2). The Liminal has voiced reactions to combat events (Newtype Flash: brief breath; NT-D activation: focused exhale; taking damage: pain vocalization) but no spoken dialogue in cutscenes — the Liminal's dialogue is text-displayed for player identification.

---

## 15. COMPETITIVE / ESPORTS HOOK

### 15.1 World Duel Championship (WDC)

**Format:** Biannual global tournament; two events per year (Spring and Fall).

**Qualifier structure:**
- Season 1 (months 1-3): Regional qualifiers through Duel Arena Holder Challenge tournaments; top 16 per region advance
- Regional championship: Top 16 play double-elimination for 4 spots per region
- WDC: 6 regions x 4 spots = 24 players; 1 wildcard per region (community vote on stream viewer engagement) = 30 players; 2 Nexus Sovereign at-large invitations = 32 players total

**Format at WDC:**
- Group stage: 4 groups of 8, round-robin, top 4 from each group advance
- Quarterfinals through Finals: Double-elimination bracket
- Grand Final: Best of 5

**Suit draft system (WDC exclusive):**
- Each finalist declares a Main Suit and bans 1 suit from their opponent's selection
- No suit can be used in back-to-back matches in a set (forces breadth)
- If a suit wins 3 or more WDC matches in a season, it enters "Priority Watch" — public balance review within 14 days

---

### 15.2 Prize Structure

WDC is a prestige event, not primarily a prize event. Revenue from competitive play comes from viewership, not player entry fees.

| Placement | Cash Prize | In-Game Reward | Title |
|-----------|-----------|----------------|-------|
| Champion | $250,000 | Animated Championship Trophy profile piece; exclusive suit paint; "WDC Champion [Year]" badge | WDC Champion |
| Runner-Up | $100,000 | Champion Trophy variant (silver); exclusive suit color | WDC Finalist |
| 3rd-4th | $50,000 each | 3rd-place cosmetic bundle | WDC Semi-Finalist |
| 5th-8th | $15,000 each | Quarterfinals emblem | WDC Quarterfinalist |
| 9th-32nd | $2,500 each | WDC Participant badge | WDC Participant |

**Total prize pool per event: $520,000.** Budget rationale: this is within the investment range where prize pool size drives significant streaming viewership uplift (ROI-positive through viewership-driven user acquisition at target $3-5 CPI).

---

### 15.3 Streaming Integration

**Platform partnerships:** Twitch (primary), YouTube Gaming (co-stream rights), Bilibili (China/SE Asia), NicoNico (Japan).

**Drops:** Viewers of WDC streams earn drops every 30 minutes of watch time. Drops are exclusively cosmetic parts (Common to Rare tier only; no Legendary from drops). Designed to drive viewership without creating a pay-to-watch-to-win dynamic.

**Co-streaming rights:** Any Nexus Sovereign or Holder-tier player can co-stream their Duel Arena matches with official Gundam Nexus co-stream permissions. This builds an organic content creator ecosystem at zero marketing cost.

**In-game spectator features:**
- Mechanic HUD overlay: In spectator mode, all active mechanics are displayed as real-time gauges for both players. Ideal for broadcast commentary.
- "Nexus Vision" camera: Auto-director camera that highlights Newtype Flash moments, ZERO System activations, and NT-D transformations. Can be toggled by stream producers.
- Real-time stat overlay: Shows Combo Depth, Resonance Gauge %, active Striker Pack, HP — designed for broadcast legibility

---

### 15.4 Creator Program

**Gundam Nexus Creator Network:**
- Any content creator with 1,000+ followers on any platform can apply
- Benefits: Early patch note access (24 hours before public), Creator-exclusive cosmetic drops to distribute to audience, co-stream rights, quarterly Creator summit (digital)
- Revenue: Creators with a referral link receive 5% of new player first-purchase revenue (90-day attribution window)
- Requirement: 1 Gundam Nexus video per month minimum; no sponsored negative coverage (standard creator contract)

---

## 16. MONETIZATION MODEL

### 16.1 Monetization Philosophy

Gundam Nexus is not free-to-play in the predatory sense. It is free-to-access with premium depth.

- The full campaign is playable at no cost.
- All core mechanics are available at no cost.
- Competitive Duel Arena is fully playable at no cost.
- The gacha system is the primary premium layer, targeting collector-motivation players.
- No mechanic, suit, or stat advantage is exclusively gated behind payment.

---

### 16.2 Revenue Streams

**Gacha (Standard Banner):**
- Cost: 160 Premium Crystal per pull; 10-pull pack: 1,440 (10% discount)
- Premium Crystal pricing: $0.99 for 80 (effectively $1.99 per pull at standard; $1.80 per pull at 10-pack)
- Pity: Guaranteed Legendary part at 80 pulls (hard pity); 50% chance at 40 (soft pity)
- F2P Crystal earn rate: approximately 15 pulls per month from campaign + daily missions + Duel Arena

**Gacha (Limited Banners):**
- Tied to series anniversaries, new series reveals, WDC events
- Harder pity (100 hard pity) but exclusively Legendary+ parts with enhanced visual effects
- No banner-exclusive mechanics — all limited parts have stat equivalents in standard banners

**Duel Pass (Subscription):**
- Price: $9.99/month or $89.99/year (25% discount)
- Benefits: +50% XP from all sources; 10 Premium Crystal/day (300/month = approximately 1.875 free pulls/month); 3 exclusive cosmetics per season; "Duel Pass" badge on profile
- Does not include any stat advantage, guaranteed Legendary parts, or Duel Arena ranking boost

**Premium Base Game (PC/Console):**
- Price: $39.99 (PC), $49.99 (PS5/Xbox Series)
- Includes: Full campaign; all 20 launch suits; 1,000 Premium Crystal (approximately 6 pulls); "Founder" profile badge
- Does not include: Limited banner parts; additional Duel Pass content

**Expansion DLC:**
- Price: $14.99 to $24.99 per Act expansion (post-launch story)
- Each expansion: New Act (6-8 chapters), 2 new suits, series-specific Builder parts, new campaign dimension

---

### 16.3 Regional Pricing

Following App Store and Google Play regional pricing tiers (Tier A/B/C/D per platform):
- JP: Standard pricing (2,400 yen for 1,280 Crystal)
- SEA (Indonesia, Thailand, Vietnam): Tier C pricing (approximately 50% of US equivalent)
- Brazil: Tier C pricing
- India: Tier D pricing (approximately 30% of US equivalent)
- China: Operated under TapTap or local partner due to regulatory requirements; separate SKU pricing

---

## 17. TECHNICAL SPECIFICATIONS

### 17.1 Engine and Rendering

**Engine:** Unreal Engine 5.4 (chosen for Lumen global illumination, Nanite virtualized geometry for suit detail, and MetaHuman for pilot character rendering).

**Minimum Mobile Specs (Android):**
- RAM: 4GB
- SoC: Snapdragon 8 Gen 1 / Dimensity 9000 equivalent or above
- Storage: 4GB download (initial); up to 12GB with all audio assets

**Minimum Mobile Specs (iOS):**
- Device: iPhone 12 or above (A14 Bionic)
- Storage: 4GB download

**Target Mobile Frame Rate:**
- 60fps on high-end devices (iPhone 15 Pro, Snapdragon 8 Gen 3)
- 30fps stable on minimum spec devices

**PC Minimum Spec:**
- CPU: Intel Core i5-10400 / AMD Ryzen 5 3600
- GPU: NVIDIA GTX 1660 Super / AMD RX 5600 XT
- RAM: 16GB
- Storage: 25GB SSD (required for texture streaming)

**PC Recommended Spec:**
- CPU: Intel Core i7-13700K / AMD Ryzen 7 7700X
- GPU: NVIDIA RTX 4070 / AMD RX 7800 XT
- RAM: 32GB
- Storage: 25GB NVMe SSD

**Console:** Native 4K/60fps on PS5 and Xbox Series X; 1080p/60fps on Xbox Series S.

---

### 17.2 Gesture Recognition Technical Specifications

**Input sampling rate:**
- Mobile: 120Hz touch sampling (240Hz on compatible devices)
- PC mouse: 500Hz polling rate minimum recommended; 1000Hz optimal
- Console controller: 120Hz stick polling

**Gesture recognition pipeline:**
1. Input captured at hardware rate
2. Gesture classifier (CNN-based, 4MB on-device model): classifies input into gesture vocabulary
3. Timing validator: Checks gesture against active combat state timing windows
4. Effect resolver: Maps validated gesture to combat action
5. Server-side replay: Input stream sent to server for anti-cheat validation (async, does not block local rendering)

**Latency target:**
- Input-to-visual-response: under 16ms (one frame at 60fps)
- Gesture recognition add: under 3ms overhead
- Total gesture-to-response: under 19ms

**Cross-platform normalization:** Platform input differences are mapped through the gesture vocabulary layer. A 150ms right drag on PC and a 150ms right flick on mobile produce identical gesture recognition output, enabling full cross-platform competitive fairness.

---

### 17.3 Network Architecture

**PvP (Duel Arena):**
- Architecture: Dedicated server per match (not P2P)
- Rollback netcode with 8-frame buffer
- Target latency: under 80ms for ranked; warning displayed above 120ms; match voided if sustained above 200ms for over 10 seconds

**Campaign:**
- Primarily offline-capable; progress syncs on connection restore
- Resonance Cascade visions that reference other players' histories require internet; offline fallback uses pre-generated vision library

**Account system:**
- All account state stored server-side
- Local cache of last-known state for offline play (campaign only; Duel Arena requires connection)

---

## 18. RISK REGISTER

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|-----------|
| Bandai Namco license scope insufficient | Medium | Critical | Pre-negotiate full 10-series scope; modular license structure allows launch with 5-6 series and expand; prepare for 6-suit roster minimum viable product |
| Gesture combat accessibility barrier (mobile) | High | High | Academy Mode with expanded parry windows; comprehensive tutorial system; optional auto-parry assist for accessibility; disable gesture recognition option (tap-to-attack fallback, no mechanic access restrictions) |
| Platform fragmentation of gesture input | Medium | Medium | Gesture normalization layer (Section 17.2); internal testing suite with all platform input methods before each patch |
| Gacha regulation (Belgium, Netherlands, South Korea) | High | Medium | Loot box odds publicly displayed (required); direct purchase option for all gacha-obtainable items at 20x pull cost; "no gacha" market variants if required |
| WDC viewership underperformance | Medium | Medium | Organic content creator program builds base before WDC; co-stream rights drive long-tail viewership; WDC prize pool marketing is supplementary, not primary |
| Physical Gunpla AR scan fraud | Low | Low | AR model limited to Bandai official product database; scan limited to one reward per unique kit per account; server-side scan token validation |
| Suit balance crisis at launch | High | Low (long-term managed) | Biweekly patch cadence committed; community council in place; win-rate transparency; no suit buffs/nerfs locked behind update cycle delays |
| Black History meta-narrative retention drop | Medium | Medium | Chapter gates ensure most players complete Act 1-2 before accessing NG+; run 1 is complete and satisfying standalone; NG+ is for retention, not first-impression |
| Apotheosis antagonist perceived as too abstract | Low | Medium | Act 3 contact scene must be executed at cinematic A-tier quality; Apotheosis must have a distinct voice, a personality, and a specific voice actor with distinct cadence; test with focus groups at script stage |
| SEA market price sensitivity | Medium | Medium | Regional pricing implemented at launch; Tier D pricing in high-price-sensitivity markets; watch-to-earn drops create no-cost engagement pathway |

---

## APPENDICES

### Appendix A: Mechanic Shorthand Reference (Developer Internal)

```
NF   = Newtype Flash (UC perfect parry)
NT-D = NT-D Destroy Mode (Unicorn 30s burst)
GF   = GUND-Format (WfM health-cost burst)
SP   = Striker Pack Swap (SEED modular equipment)
SM   = SEED Mode (SEED stat burst at HP below 40% or D3/D5)
TA   = Trans-Am (00 combo D4+ overdrive)
ZS   = ZERO System (Wing AI-assist berserker)
AV   = Alaya-Vijnana (IBO precision parry, HP cost)
BH   = Black History (Turn A NG+ meta system)
GB   = Gunpla Builder (Build Fighters customization)
NRG  = Newtype Resonance Gauge (cross-mechanic amplifier)
RC   = Resonance Cascade (NRG at 100% for non-Unicorn suits)
UME  = Unique Mechanic Expression (per-suit signature ability)
DP   = Duel Points (Duel Arena currency)
WDC  = World Duel Championship
```

### Appendix B: Series Licensing Priority Order

For license acquisition if full 10-series scope is not achievable at launch:

| Priority | Series | Minimum Viable Reason |
|----------|--------|----------------------|
| 1 | Universal Century (0079 + CCA) | Foundation; RX-78-2 as onboarding suit is non-negotiable |
| 2 | Witch from Mercury | Duel Arena system requires WfM license; most commercially current |
| 3 | Unicorn | NT-D is the game's signature burst mechanic; Nu Gundam #1 global |
| 4 | SEED | Strike Freedom global recognition; Striker Pack is critical system |
| 5 | Wing | Western audience hook; ZERO System is unique high-skill mechanic |
| 6 | Gundam 00 | Trans-Am overdrive mechanic; Setsuna in Act 3 |
| 7 | IBO | AV parry mechanic; Barbatos's design is distinct in roster |
| 8 | Turn A | Black History NG+ system works without Turn A suit; suit is NG+ exclusive |
| 9 | Build Fighters | Gunpla Builder system does not technically require Build Fighters license (Builder system is original; suits are licensed separately) |
| 10 | Age / Reconguista | Low commercial priority; can be post-launch addition |

**Minimum viable product:** Series 1-5 (UC, WfM, Unicorn, SEED, Wing) with 10 suits = demonstrable Combinatorial Maximalism. Sufficient for soft launch.

### Appendix C: Playtesting Benchmarks (Internal Targets)

| Test | Target Metric | Failure Threshold |
|------|--------------|-------------------|
| Tutorial completion | Above 85% of new players complete Act 1-1 | Below 70% = redesign tutorial gates |
| Newtype Flash learn rate | Above 60% of players land their first NF by end of Act 1-2 | Below 40% = expand parry window in Academy Mode |
| ZERO System first activation | Above 70% of Wing mains activate ZS within first 10 Wing matches | Below 50% = reduce ZS gauge requirement |
| NT-D perceived as fair by opponents | Below 30% of post-match surveys cite NT-D as "unfair" | Above 45% = reduce NT-D duration or add visible warning |
| Duel Arena Day 7 return | Above 45% of players who enter Duel Arena return Day 7 | Below 30% = increase new-player DP floor |
| Gunpla Builder session time | Average 12+ minutes per Builder session | Below 6 minutes = simplify UI or add contextual recommendations |
| AR scan engagement | Above 15% of MAU scan kit per month by Month 6 | Below 8% = increase first-scan reward; add social sharing incentive |
| Campaign completion (Act 1-5) | Above 60% at 30 days | Below 40% = add skip options for cutscenes; reduce late-Act difficulty spike |

---

*Document version 1.0. Prepared for pre-production review. All mechanical parameters are design targets subject to revision based on playtesting data. All revenue projections are internal planning figures and should not be shared externally. Bandai Namco licensing terms pending negotiation.*

*Next review: 30 days. Owner: Game Director.*

---

**END OF DOCUMENT — GUNDAM NEXUS GDD v1.0**
