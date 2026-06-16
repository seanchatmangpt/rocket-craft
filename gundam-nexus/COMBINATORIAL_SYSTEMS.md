# Gundam Nexus — Combinatorial Systems Design
## Technical Game-Design Document: Series-Specific Mechanics Blueprint

**Document Type:** Technical Game Design  
**Scope:** All series-derived combat systems, their interactions, and how they layer together  
**Philosophy:** Combinatorial Maximalism — each Gundam series contributes one irreducible mechanical identity; no two systems are synonymous; all systems interact meaningfully without canceling each other out  
**Series Covered:** Iron-Blooded Orphans, The Witch from Mercury, SEED, Unicorn, 00, Wing, Iron-Blooded Orphans (Alaya-Vijnana), Turn A, Build Fighters, Universal Century

---

## Design Philosophy: Combinatorial Maximalism

Gundam as a franchise spans 45+ years and 20+ distinct series. Each series has a visual identity, a thematic identity, and — crucially for Gundam Nexus — a mechanical identity. The Witch from Mercury's central dramatic engine is the duel. Iron-Blooded Orphans is built around the violence of close-quarters melee and the physical toll of the Alaya-Vijnana system. Turn A is fundamentally about cycles, repetition, and the cost of accumulated history.

Gundam Nexus does not flatten these distinctions. Every major series is given one mechanical system that cannot be replicated by any other series. These systems layer — a pilot using Barbatos in a Witch from Mercury Duel Arena, while activating the Alaya-Vijnana Interface and building toward a Newtype Flash, is experiencing three different series' mechanical languages simultaneously. This is the combinatorial payoff.

The design challenge is ensuring no system dominates or renders others obsolete. Each system is designed with a distinct risk/reward profile:
- Some systems are always active (passive bonuses)
- Some require setup (charge meters, consecutive parries)
- Some are one-shot per match (ultimate abilities)
- Some impose costs (HP tax, stat penalties after activation)

Players who master one system are not automatically competent with another. Mastery of all ten systems simultaneously is the skill ceiling of Gundam Nexus. Casual players can thrive with one or two. This design range supports both mobile-casual and esports-competitive play.

---

## System 1: IB4 Gesture Combat Foundation

**Series Basis:** Iron-Blooded Orphans (melee-first combat doctrine)  
**System Type:** Core combat engine (applies to all suits unless superseded by a suit-specific override)  
**Layer:** Foundation — all other systems build on top of this

### Core Gesture Mapping

The gesture input system was developed from internal playtests demonstrating that directional swipes create more tactile investment than button-press confirmations on touchscreen. The mapping is fixed and cannot be remapped by players (consistent competitive standard):

| Gesture | Input Direction | Attack Type | Notes |
|---------|----------------|-------------|-------|
| High Strike | Swipe Up (↑) | Overhead beam saber slash | Effective vs. blocking opponents; countered by upward shield |
| Horizontal Slash | Swipe Left (←) | Wide sweeping cut | Best range; hits multiple parts of suit; slight startup lag |
| Thrust | Swipe Right (→) | Forward piercing stab | Fastest; lowest damage; breaks guard on extended blocks |
| Ranged | Hold + Release (any direction) | Beam rifle shot | Hold duration determines charge; tap = quick shot, 2s hold = charged shot |
| Dodge | Swipe Down (↓) | Evasion burst | No invincibility frames; purely distance-based |

### Beam Saber Variants

Every suit has a distinct beam saber animation but uses the identical gesture map. The animation affects feel and personality, not mechanical properties. Examples:

- **RX-78-2 Gundam:** Wide horizontal sweeps, Amuro-era deliberate style, saber held at midpoint
- **Barbatos:** Mace-forward brawler style; saber held at base (IBO's street-fighting aesthetic)
- **Wing Zero:** Precise surgical cuts; after-image trails reflect ZERO System-assisted accuracy
- **Tallgeese:** Heavy overhead slams; saber held two-handed, reflecting Zechs' power-over-speed philosophy
- **Destiny Gundam:** Dual-wield animation; left and right gesture triggers separate sabers simultaneously (unique exception to one-saber standard; balanced by smaller per-hit damage)
- **Susanoo:** Spinning mortal sword animation; charges through horizontal slash before resolving (0.3s delay between gesture and hit; higher damage payoff)

The animation system allows the art team to express characterization through combat feel without requiring mechanical exceptions for every suit.

### Parry Windows

Parry is executed by swiping opposite to the incoming attack direction. The timing windows are:

| Parry Type | Input Window | Effect | Unlock Condition |
|-----------|-------------|--------|-----------------|
| Normal Parry | 200ms | Blocks damage, slight knockback on attacker, no meter gain | Default |
| Perfect Parry ("Newtype Flash") | 50ms (within the 200ms window) | Blocks damage, +10% Newtype gauge, Impact Seal visual, opponent staggered 0.5s | Always available; requires precision |

The 200ms window was tuned for touchscreen latency compensation. On 60fps hardware, 200ms = 12 frames. On 30fps hardware, 200ms = 6 frames. The game detects device refresh rate at launch and applies a 1.5x window multiplier on sub-60fps devices (effective 300ms / 75ms on older hardware).

The 50ms perfect parry window was tuned against the average human reaction time baseline of 200-250ms. Landing a perfect parry requires anticipation (reading the opponent's gesture pattern) rather than pure reaction. This prevents perfect parry spam and rewards pattern recognition over raw reflexes.

### Combo Chain Depth System

Combo depth tracks consecutive hits without the opponent successfully parrying or dodging.

| Combo Depth | Multiplier | Visual Indicator |
|------------|-----------|-----------------|
| 1 hit | 1.0× base damage | No indicator |
| 2 hits | 1.5× base damage | Yellow flash on hit counter |
| 3 hits | 2.0× base damage | Orange aura on suit |
| 4+ hits | 3.0× base damage | Red aura + combo counter displayed prominently |

**Combo Reset Conditions:**
- Opponent successfully parries (any parry type resets to depth 0)
- Opponent successfully dodges
- 2 turns of no hits land (idle reset)
- Own HP drops below 25% (panic reset — defensive incentive)

Alaya-Vijnana modification: With AVJ active, the idle reset extends to 3 turns instead of 2. This reflects the IBO philosophy of sustained close-quarters pressure.

### Foundation Interaction with Other Systems

The IB4 Foundation is the substrate. All other systems express through it:
- Trans-Am multiplies combo damage (see System 5)
- NT-D extends perfect parry window (see System 4)
- ZERO System affects attack direction selection (see System 6)
- Build Fighters parts modify base ATK/DEF/SPD stats that the multipliers act on (see System 9)

No other system replaces the gesture map. Even when AI-assisted systems (ZERO, Epyon) take partial control, they still execute gestures — the player sees the system selecting the gesture on their behalf.

---

## System 2: Witch from Mercury — Duel Arena Protocol

**Series Basis:** Mobile Suit Gundam: The Witch from Mercury  
**System Type:** Match structure and ranked progression  
**Layer:** Meta-game (wraps around individual matches)

### Design Premise

The Witch from Mercury centers the duel as a social and legal institution — the Duel Company system, Miorine's roses, the formal challenge-and-accept protocol. Gundam Nexus maps this narrative institution directly to its PvP ranked structure. Every ranked match is, formally, a duel. The Duel Arena Protocol is not flavor; it is the match flow.

### Duel Challenge Flow

1. **Challenge Declaration:** Challenger selects opponent from ranked lobby, sends formal "Duel Request"
2. **Acceptance Window:** Defender has 60 seconds to accept or decline (decline incurs no penalty; second decline within 10 minutes = 15-minute matchmaking cooldown for challenger)
3. **Arena Selection:** Both players jointly select from 5 canonical arenas; if no agreement in 30 seconds, random selection applies
4. **Pre-Match Ceremony:** 5-second cutscene of both suits entering the arena (skippable; auto-skipped in Tournament mode)
5. **Match execution:** Standard IB4 gesture combat
6. **Post-Match Protocol:** Winning suit performs a 3-second "victory pose" (customizable; unlocked via battle pass); loser's suit kneels briefly (animates the Witch from Mercury tradition of bowing out)

### Arena Roster

Five canonical arenas, each with a distinct environmental mechanic:

**1. Asticassia School Arena**
- The formal dueling ring from the Benerit Group's school
- Mechanic: Bounded arena (no terrain escape possible). Clean symmetrical design. No environmental hazard.
- Use case: The "pure duel" arena. Favored for ranked matches. No environmental variance.
- Visual: Hexagonal ring, Asticassia emblem on floor, spectator stands visible in background

**2. Colony Harbor**
- A retrofitted freight bay in an orbital colony
- Mechanic: "Gravity shift" event — at 2 minutes elapsed, colony segment rotates; all movement directions invert for 10 seconds
- Use case: Experienced players aware of shift timing gain advantage; newcomers disoriented
- Visual: Cargo containers as partial cover (destructible, create temporary obstacles)

**3. Quiet Zero Remnant**
- The ruins of the Quiet Zero facility post-series
- Mechanic: "Gundam Aerial echo" — once per match, a spectral Gundam Aerial silhouette appears and performs a random gesture attack on both players (equal threat); telegraphed 2 seconds before
- Use case: Chaotic; favored for casual matches. Introduces a random element that can swing momentum.
- Visual: Damaged superstructure, floating debris field, GUND-format residue visual effect (faint pink particles)

**4. Earthian Market**
- A street-level market in the Earth Witch from Mercury setting
- Mechanic: "Civilian scatter" — moving too quickly (3+ dodges in 10 seconds) triggers a scatter event; arena temporarily narrows (invisible walls reduce playfield 20%) for 15 seconds
- Use case: Punishes aggressive dodge-spam; rewards controlled movement
- Visual: Market stalls as background detail, cramped urban setting, scale mismatch (Gundam-sized suits in human-scale market)

**5. GUND-Format Reactor Core**
- The interior of an active GUND-format energy system
- Mechanic: "Permet surge" — every 90 seconds, a Permet surge fires; contact with the surge beam deals 15% max HP damage to both players regardless of defense
- Use case: Creates a ticking clock dynamic; both players motivated to secure a lead before next surge
- Visual: Deep space core aesthetic, massive circular energy rings, Permet particle ambient glow

### Duel Ranking System

| Rank | Title | Player Percentile | Visual Badge |
|------|-------|------------------|--------------|
| Bronze | Cadet | Bottom 40% | Bronze shield emblem |
| Silver | Officer | 40–65% | Silver sword emblem |
| Gold | Commander | 65–80% | Gold falcon emblem |
| Platinum | Ace | 80–92% | Platinum star emblem |
| Diamond | Veteran | 92–98% | Diamond crystal emblem |
| Legendary | Champion | 98–99.9% | Animated flame emblem |
| Holder | Duel Holder | Top 1 per region | Animated crown; suit displays gold trim |

**Holder Mechanics**
- One "Duel Holder" title exists per regional server (NA, EU, JP, KR, SEA, etc.)
- Any Legendary-rank player may challenge the Holder at any time
- Holder challenges are publicly announced in-game and streamed on the official broadcast channel
- Holder defenses are mandatory: three defenses per week minimum; failure to defend results in automatic title forfeiture to the highest-ranked challenger
- Holder's suit displays a cosmetic gold trim available only while holding the title; removed upon losing
- Weekly cosmetic trophy awarded to active Holder: a unique emblem variant

### Technical Match Rules

- Duration cap: 5 minutes per match
- If time expires: score-based result (damage dealt as percentage of opponent's max HP is the metric)
- Tie conditions (equal damage): 1-minute sudden death; next hit ends the match
- Disconnect: 60-second reconnect window; failure to reconnect = loss; 3 disconnects in a week = 24-hour ranked suspension
- Match history: last 20 matches stored in "Duel Record" screen, accessible to all players (public by default, opt-in private)

### Suit Restriction Events

Monthly suit restriction events apply to unranked casual play only:

- **HG Cup:** Only suits classified as "HG tier" (Recruit-class stat base). Levels the playing field; showcases that mechanical skill matters more than suit tier.
- **OG Only:** Only original suits (non-canon suits created for the game or Gunpla Builder custom suits). Pure player expression match.
- **Single Series:** All participants must use suits from the same Gundam series (e.g., "SEED Week" — only SEED/SEED Destiny suits). Creates thematic focused events.
- **Striker Pack Locked:** SEED Striker Packs cannot be swapped mid-match (see System 3). Tests pre-match decision-making.

---

## System 3: SEED — Striker Pack System

**Series Basis:** Mobile Suit Gundam SEED / SEED Destiny  
**System Type:** Mid-combat equipment loadout hot-swap  
**Layer:** Strategic (pre-match build) + Tactical (mid-combat adaptation)

### Design Premise

SEED's Strike Gundam was defined by its ability to swap "Striker Packs" — external equipment modules transforming it from a balanced fighter to a melee specialist to a long-range platform within seconds. This is Gundam Nexus's core strategic flexibility system. Any suit can be equipped with Striker Packs (it is retrofitted as a universal docking system in the Nexus game universe); SEED-series suits have native integration (no swap animation delay penalty).

### Striker Pack Types

**Aile Striker (Balanced)**
- Stat modifications: +10% mobility (dodge distance extended), +5% all stats
- Passive: "Phase Shift Activation" — the first hit taken each match is negated (one-use armor effect)
- Visual: Wing-mounted thrusters, standard blue colorway
- Best for: Players who want a safety net and general performance

**Sword Striker (Melee Focus)**
- Stat modifications: +25% melee (saber) damage, -15% ranged (beam rifle) damage
- New action unlocked: "Schwert Gewehr" — held down-swipe triggers a heavy downward blade swing (2.0× base damage, 0.8s startup, can be parried)
- Visual: Anti-Ship sword on backpack, dark blue/gunmetal colorway
- Best for: Aggressive melee players; high-risk, high-reward

**Launcher Striker (Ranged Focus)**
- Stat modifications: +20% ranged (beam cannon) damage, -10% melee damage
- New action unlocked: "Agni Beam Cannon" — tapping and holding the ranged input for 2.5s fires the Agni cannon (3.5× base ranged damage, single use per match; recharges over 3 minutes)
- Visual: Massive shoulder cannon, bright orange warning striping, heavy backpack silhouette
- Best for: Defensive-range players; punishes aggressive opponents from a distance

### Swap Mechanic

- Striker Packs are selected before match start (up to 3 loadout slots; player brings all 3 to the fight)
- Mid-combat swap: Tap the Striker Pack icon in combat UI corner; begins 2.0-second swap animation (suit is vulnerable during animation; taking a hit cancels the swap and initiates a 30-second cooldown)
- Native SEED suits (Strike, Duel, Buster, Blitz, Aegis): 0.5-second swap animation (native integration)
- Visual transformation: The old pack physically detaches in a particle burst; new pack locks in with a mechanical click effect

### Suit-Specific Interactions

**Strike Gundam (Native):** All three packs; 0.5s swap. The reference implementation.

**Strike Dagger / Windam:** Carries Aile and Sword packs only (Launcher too heavy for mass production frame); swap at 1.0s (between native and standard).

**Infinite Justice Gundam:** Pack integrated permanently — Fatum-00 backpack is the "fourth pack type" unique to Justice. Cannot swap. No stat penalty for fixed config; Fatum-00 provides both +15% melee and +10% ranged simultaneously. Balance: no Phase Shift first-hit negation.

**Destiny Gundam:** Ships with a preset Striker configuration (Aile base + integrated weapons). Cannot swap packs mid-combat. Instead, Destiny's unique system is "Mirage Colloid Burst" — a 3-second afterimage decoy triggered by tap-down that absorbs one attack.

**Aegis Gundam:** Unique "Mobile Armor transformation" — instead of Striker swaps, Aegis can toggle between MA form (ranged, +30% beam damage, immobile) and MS form (mobile, standard stats) with a 1.0s transformation. Aegis does not equip standard Striker Packs.

### Economic Model

**How Packs Are Obtained:**
- Crafting via Gunpla Parts (Build Fighters system): Combine 3 Backpack parts + 1 Weapon part = any Striker Pack (standard version)
- Gacha: Cosmetic Striker Pack variants (different color schemes, animated effects) — function identically to standard packs; purely visual
- Shop: Direct purchase of standard packs at 400 Credits each
- AR Bridge: Certain Gunpla kits include a Striker Pack unlock (e.g., MG Strike Gundam with Aile pack includes digital Aile Striker)

Standard Striker Packs are never exclusive or gated by luck. Cosmetic variants (gold Aile, psycho-frame Sword, etc.) may be gacha exclusive, but they function identically to standard packs. No pay-to-win element exists in the Striker system.

---

## System 4: Unicorn — NT-D (Newtype Destroyer) Mode

**Series Basis:** Mobile Suit Gundam Unicorn  
**System Type:** Timed ultimate activation with narrative confrontation  
**Layer:** Ultimate ability (single powerful activation per match)

### Design Premise

Unicorn's NT-D system is the defining "transformation" in post-UC Gundam. The Unicorn Gundam shatters its seals, the psycho-frame ignites, and for thirty seconds it becomes something beyond a mobile suit. Gundam Nexus represents this as the game's most dramatic single activation: a timed power state with extraordinary bonuses, a mandatory narrative moment, and suit-specific variations that express each Unicorn-series unit's distinct relationship with the NT-D system.

### Activation Conditions

- Requires: Newtype Gauge at 100% (see System 10 for gauge mechanics)
- Activation: Player manually triggers (double-tap cockpit icon in combat UI); cannot be triggered accidentally
- Once triggered: 30-second NT-D window begins

### NT-D Mode Effects

During the 30-second NT-D window:

| Stat | Modification |
|------|-------------|
| Damage output | +50% |
| Perfect parry window | Extended to 80ms (vs. standard 50ms) — still requires skill; not auto-parry |
| Auto-parry (passive) | One attack automatically parried per 5 seconds (no gesture required) |
| Newtype gauge drain | Continuous at -5%/second (depletes entirely over NT-D duration) |

Visual effects during NT-D:
- Unicorn Gundam suit breaks open (head horn splits, panels reveal psycho-frame underneath)
- Ambient particle aura (red/white cycling at 2Hz frequency)
- Background slightly desaturated to draw attention to suit's active aura
- Hit effects turn gold-white

**The Narrative Confrontation (Mandatory)**
- At the moment of NT-D activation: a 5-second black screen overlays the match view
- During these 5 seconds, whisper audio plays — specific to the pilot (Banagher's whisper: "I won't deny anyone's potential." Marida's: "Remain in light." Full Cast VO for main suits.)
- The 5-second window does not pause the match — the opponent can still attack (auto-parry activates during the 5 seconds as compensation)
- This is intentional: NT-D activation is a momentary vulnerability, mirroring the series' theme that the NT-D's power comes at a cost to the pilot's psyche

### Post-NT-D Cooldown

- After the 30-second window ends: NT-D enters cooldown of 180 seconds (3 minutes)
- Newtype Gauge refills normally during cooldown (via perfect parries)
- Can be activated again if gauge refills before match ends (in a 5-minute match, theoretical maximum of 2 activations if gauge refills perfectly)

### Suit-Specific NT-D Variants

**Unicorn Gundam (RX-0):** Standard NT-D as described above.

**Banshee Norn (RX-0[N]):** "NTD Override" — aggressive variant
- Duration: 25 seconds (shorter than standard)
- Stat modification: +80% damage output, -50% defense (psycho-frame overwhelms defensive systems)
- Auto-parry: Disabled (pure aggression, no protection)
- Visual: Gold/black aura; Neo Zeong psycommu claws extend as visual effect (no mechanical effect)
- Risk profile: Extremely high damage; one successful hit from opponent during NTD Override is potentially fight-ending. Requires immediate commitment to offense.

**Kshatriya / Full Frontal's Sinanju:** "Red Comet NT Resonance" — villainous variant
- Duration: 20 seconds
- Effect: All 4 auto-targeting Funnels/Bits deploy (Kshatriya's binders open, releasing bits); they attack the opponent independently at 15% damage per bit, per 5 seconds
- Player still controls the main suit normally during funnel deployment
- Funnels can be parried (each parry counts toward Newtype Gauge if perfect); if all 4 are parried, they return to standby
- Best counter: Perfect parry funnels to build gauge and deny opponent's NT-D extension

**Phenex (RX-0 Phenex):** "NT Resonance" — support variant
- Duration: 45 seconds (longer window)
- Damage bonus: None (zero direct damage increase)
- Effect: Every 10 seconds during NT Resonance, one auto-heal for 5% of max HP; if playing in co-op mode, the heal extends to allied suits in range
- Additionally: AoE "Resonance Pulse" on activation — knocks back opponent, 3-second stagger
- Design note: Phenex is a support NT-D that rewards patient play; not optimal in 1v1, extremely powerful in co-op raid content

---

## System 5: 00 — Trans-Am Overdrive

**Series Basis:** Mobile Suit Gundam 00  
**System Type:** Combo-triggered power mode with a significant post-activation penalty  
**Layer:** Tactical (rewarding sustained aggression) with strategic risk (the cost of use)

### Design Premise

Trans-Am in Gundam 00 represents Celestial Being's trump card — a state of maximum output that burns through GN particle reserves, leaving the suit weakened after use. The reward is extraordinary; the cost is real. This risk/reward asymmetry is the heart of the 00 system in Gundam Nexus.

### Trans-Am Meter

The Trans-Am Meter is independent of the Newtype Gauge. It fills exclusively through sustained high-depth combo chains:
- While at combo depth 4+, the Trans-Am Meter fills at 10% per turn
- At depth 3 or below, the meter does not fill (and does not drain)
- Meter fills from 0% to 100% over approximately 3 turns at depth 4+ (10 turns of depth 3 = nothing)
- Meter persists between combo resets — it does not drain when a combo breaks

This design rewards players who can maintain high combo depth. Since combo depth 4+ requires 4 consecutive unhurt hits, reaching it against a skilled opponent who actively parries is a meaningful accomplishment.

### Trans-Am Activation

When the meter reaches 100%, a "TRANS-AM AVAILABLE" indicator pulses in the HUD. The player must manually activate by double-tapping the Trans-Am icon.

**Trans-Am Window: 3 Turns**
During the active 3-turn Trans-Am window:
- All stats multiplied by ×3 (damage, defense, speed, special ability power)
- Beam effects turn red/gold (GN particle aesthetic)
- Background desaturates partially (visual focus on the suit)
- Time appears to slow: opponent's gesture animations play at 0.7× apparent speed (does not actually give more reaction time; purely visual — but the visual creates a sense of control)
- Combo depth is maintained through Trans-Am regardless of opponent's defensive actions (Trans-Am pressure cannot be parried through conventionally; parrying during Trans-Am does not reset the attacker's combo depth)

### Trans-Am Burst (Post-Activation Penalty)

After the 3-turn window expires, the suit enters "Trans-Am Burst" — GN particles depleted:
- Duration: 5 turns
- All stats reduced to -40% of base (net: significantly below normal; -40% effectively means the suit is fighting at 60% effectiveness)
- The 00-series visual: suit dims, GN drive pulsing slows, exhaust trails fade to pale yellow
- The player cannot activate Trans-Am again until the 5-turn Burst ends and the meter refills (meter refill only begins after Burst ends)

The Burst penalty is the strategic crux. A player who activates Trans-Am with 6 turns remaining in a match must survive 5 penalty turns — if they failed to finish the fight in 3. A player who activates with 3 turns left and wins during Trans-Am loses nothing. Timing is everything.

### Suit-Specific Trans-Am Variants

**00 Raiser (GN-0000+GNR-010):** Standard Trans-Am as described above. The ×3 multiplier and Burst penalty apply at full values.

**00 Qan[T]:** "Quantum Burst" variant
- Trans-Am window: 2 turns (shorter)
- Stat multiplier: ×4 (higher ceiling)
- Burst penalty duration: 3 turns (shorter; Qan[T]'s advanced GN drives recover faster)
- Additional: On activation, one "Quantum Teleport" available — instant dodge to any position on the arena (no animation, no cooldown during Trans-Am). One use only per activation.

**Exia Repair (GN-001/Re):** "Improvised Trans-Am" variant
- Meter fills at 50% effectiveness (requires 6 turns at depth 4+ instead of 3)
- Trans-Am window: 3 turns
- Stat multiplier: ×1.5 (only 50% power — the damaged GN Drive cannot output at full capacity)
- No Burst penalty (Exia Repair's degraded drive doesn't have enough reserves to cause a true Burst)
- Design intent: Exia Repair rewards patience (longer to charge) with a low-risk, steady Trans-Am. Good for beginners learning the system.

**Cherudim (GN-006):** "Sniper Trans-Am" variant
- Activation immediately fires a single ultra-damage sniper shot (10× base ranged damage; unblockable by shield, only avoidable by dodge)
- After the shot: Trans-Am mode ends immediately (no sustained window)
- No Burst penalty
- Meter resets to 0% after use; requires full recharge
- Design intent: Cherudim plays completely differently from the sustained ×3 window. Cherudim players save their meter, then spend it on one decisive shot. Miss the dodge-avoid window = instant fight-ending damage.

**Reborns Gundam:** "Twin Trans-Am" variant
- Reborns can switch between Gundam mode and Cannon mode (separate from Striker packs; it's a full MS-to-MA transformation)
- Trans-Am activates both modes' parameters simultaneously (×2.5 in MS mode, ×3.5 in Cannon mode)
- Burst penalty: 7 turns (extended due to dual-system stress)

---

## System 6: Wing — ZERO System

**Series Basis:** New Mobile Report Gundam Wing  
**System Type:** Precognitive assist system with narrative choice mechanic  
**Layer:** Psychological (the ZERO System tests the player's willingness to act)

### Design Premise

The ZERO System in Wing is not a power-up. It is a curse. It shows pilots the future — specifically the future in which they win — but the psychological burden of experiencing that possible future warps the pilot's mind toward ruthlessness. Heero, Zechs, and Quatre all struggle with it. In Gundam Nexus, the ZERO System is represented as a system that offers the player a choice: accept its assistance and its cost, or decline and face a penalty. This is the only game system with a moment of explicit player agency during activation.

### Trigger Condition

5 consecutive perfect parries (50ms window) without taking any damage in between charges the ZERO System. This is an extremely demanding requirement that rewards the most disciplined defensive players.

- Counter resets if: the player takes any damage, the player lands a non-perfect parry (normal 200ms parry resets), or a combo chain begins before the 5th parry completes
- When charged: ZERO System is available to activate (does not auto-activate)

### The Activation Choice

When ZERO System is charged and the player taps the ZERO icon, a cutscene interrupts the match for 3 seconds:

- The screen fills with a vision: the opponent's suit is shown being destroyed by the player's attack (a brief silhouette of the "winning move")
- A binary prompt appears: **"Accept the Vision"** / **"Reject the Vision"**
- The player has 3 seconds to choose (no choice = automatic Rejection)

**If the player Accepts:**
The ZERO System activates. Effects:
- AI attack assist: 60% of the time, the AI selects the optimal attack direction (the correct gesture to counter opponent's current defense posture). The player still executes the attack gesture; the AI highlights which direction to use.
- Attack speed: ×1.5 (the player's gesture inputs resolve 50% faster)
- The player loses directional choice on AI-assisted attacks (on those 60% of turns where AI is assisting, the AI has already committed to a direction; player can override but overriding cancels the speed bonus for that turn)
- Duration: 6 turns, or until the player takes 3 hits (whichever comes first)

**If the player Rejects:**
- ZERO System disengages
- 10-second penalty: all stats reduced by 15% (the psychological rejection costs the pilot)
- The Rejection choice is thematically resonant (Quatre rejected the ZERO System's logic of sacrifice)

### Suit-Specific ZERO Variants

**Wing Gundam Zero (XXXG-00W0):** Full ZERO as described above.

**Epyon (OZ-13MS):** "ZERO Berserker" — Treize's machine, built to channel the ZERO System's darkest potential
- When ZERO System is charged on Epyon, the activation choice is removed: ZERO Berserker triggers automatically on charge
- AI takes full control of the suit for 8 turns (player watches, cannot input)
- During these 8 turns: the AI executes optimal attack chains at 2.0× normal speed; guaranteed kill if opponent is below 40% HP at trigger time
- Guaranteed HP loss: Epyon cannot parry during Berserker (the ZERO overwhelms defensive processing); takes 15% max HP damage as a fixed cost
- The 8-turn wait creates a unique spectator moment — the player watches their own suit fight autonomously

Design note: Epyon Berserker is the game's most dramatic single system. The loss of player control is intentional and disturbing, mirroring the series' theme.

**Tallgeese III (OZ-00MS2B):** "Data Override" — no ZERO System; Zechs replaces psychic prescience with analytical mastery
- Tallgeese III does not trigger ZERO via perfect parries
- Instead, Tallgeese has a passive "Tactical Analysis" system: after being hit 5 times by the same attack type, the HUD permanently displays that attack type with a 2-turn advance warning (a subtle pre-flash on the incoming attack direction)
- This is never full AI control; Tallgeese gives information, not assistance. Player still executes everything.
- Balance: Tallgeese's Tactical Analysis is harder to activate (requires being hit 5 times) but has no activation cost and no duration limit (persists until match ends)

**Mercurius (OZ-13MSX1):** "Planet Defensors" — shield variant
- 5 perfect parries trigger Planet Defensors instead of ZERO
- Defensors: 6 auto-parry orbs deploy and absorb the next 6 attacks automatically (regardless of direction)
- Duration: Until all 6 Defensors are depleted (each absorbed hit destroys one Defensor)
- No damage reduction during depletion phase; when last Defensor falls, suit is briefly vulnerable (1-second exposure, enemy attack during this window deals 1.5× damage)

---

## System 7: Iron-Blooded Orphans — Alaya-Vijnana Interface

**Series Basis:** Mobile Suit Gundam: Iron-Blooded Orphans  
**System Type:** Passive combat enhancement with accumulative HP cost  
**Layer:** Passive/Equip (always active when equipped; cannot be toggled off)

### Design Premise

The Alaya-Vijnana system in IBO is not a choice. It is installed in the pilot's spine. It enhances combat ability in ways that are measurable and decisive; it also extracts a physical toll. Mikazuki Augus, the most skilled AVJ user, has adapted to this toll; other pilots pay for it incrementally. Gundam Nexus captures this through a passive system that is always active but costs HP for non-native users — a persistent tax on the enhanced capability.

### AVJ Passive: Enhanced Parry Windows

When the "Alaya-Vijnana Implant" item is equipped (separate from the suit; it is a pilot-slot item):

| Parry Type | Base Window | AVJ-Enhanced Window |
|-----------|------------|---------------------|
| Normal Parry | 200ms | 300ms (+100ms) |
| Perfect Parry | 50ms | 80ms (+30ms) |

This is a substantial improvement. The 300ms normal parry is nearly 50% more lenient; the 80ms perfect parry (matching NT-D enhanced window) gives significantly more reaction time for precision plays.

**The HP Tax**

Every parry executed while AVJ is active costs HP:
- Normal parry: -5 HP per use (out of standard 1000 HP baseline)
- Perfect parry: -10 HP per use

The cost is intentionally small per-use but accumulates. A defensive player who parries 20 times per match loses 100–200 HP to the implant toll. This is the IBO's recurring theme — Mikazuki's body degrading.

**Full Connection State**

3 consecutive AVJ-enhanced perfect parries (within a single combat session, without taking a hit) trigger "Full Connection":
- Duration: 5 turns
- Effect: All parry HP costs waived (the connection is so complete the toll is temporarily eliminated)
- Bonus: Enemy attacks display their direction as a faint pre-indicator 1 turn before the attack resolves (not certain — 70% accuracy; 30% of the time the indicator is a feint)
- Counter visibility: Full Connection active indicator is visible to the opponent (they know the player is in Full Connection)

### Suit-Specific AVJ Rules

**Barbatos (all forms), Gusion, Flauros, Helmwige Reincar:**
- AVJ built into the suit at the frame level (Third Form compatibility)
- No HP cost for parries (native adaptation; the suit's frame absorbs the toll)
- Full Connection activates with 3 consecutive perfect parries as above

**Graze / Mobile Worker pilot suits (non-native):**
- Standard HP tax applies
- Full Connection requires 4 consecutive perfect parries (harder threshold; non-adapted pilots)

**Mikazuki Augus (pilot item):**
- Zero HP cost (he has adapted beyond Third Form level; his nervous system and AVJ are fully merged)
- Full Connection activates with 2 consecutive perfect parries (most sensitive connection of any pilot)
- Negative: Mikazuki's passive "Third Form Damage" — once per match, at match start, takes 50 damage as fixed (his body's accumulated damage history)

### Interaction with Other Systems

AVJ's enhanced parry windows directly augment:
- Newtype Flash (perfect parry → Newtype Gauge; wider AVJ perfect window makes gauge building easier)
- ZERO System (5 consecutive perfect parries needed; AVJ's 80ms window makes this achievable)
- Combo Depth (parries reset combo; more accurate parries mean more precise defensive reads)

This makes AVJ a foundational competitive tool — players running AVJ are more consistent defenders, which then powers all gauge-based systems faster. The HP cost is the balancing mechanism; aggressive AVJ parry users may find themselves in HP deficit against opponents who force trades.

---

## System 8: Turn A — Black History NG+

**Series Basis:** Turn A Gundam  
**System Type:** New Game Plus meta-progression system  
**Layer:** Meta (applies to the game itself, not to individual matches)

### Design Premise

Turn A Gundam's defining revelation is the "Black History" — a buried record of all the civilizations that rose, fought, and destroyed each other before the current era. The series treats history as cyclical: the same conflicts recurr across millennia. Gundam Nexus literalizes this: your playthrough history becomes the game's history, and the game changes in response.

### Unlock Condition

Complete the Nexus campaign mode (reach credits screen). NG+ automatically unlocks.

### NG+ Revelations

**Loading Screen Replacement**
Standard loading screens are replaced with "Black History Archive" footage. This footage is procedurally generated from the player's own play data, visualized as archival records:
- Your most-used suit appears as a "lost civilization's weapon"
- Your most common arena becomes a "ruins of a great battle site"
- Your most-defeated opponent type is noted as "the enemy of the [N-th] age"

Each loading screen is unique to the player's history. Players cannot experience the same Black History Archive as another player.

**Convergence AI Memory**

The final boss of the campaign mode is the "Convergence AI" — an adaptive combat system designed to challenge any player. In NG+:
- The Convergence AI has memory of your previous run's tactics
- Your most-used attack direction is flagged as a "historical pattern"; the AI preemptively defends against it 40% of the time
- Your most-used suit has a prepared counter-strategy (specific attack type the AI biases toward based on your suit's documented weaknesses)
- Your fastest defeated enemy type is now 20% more aggressive (it "remembers" being beaten too quickly)

This makes NG+ genuinely harder for experienced players in a personalized way. The AI fights *you*, not a generic player.

### The Moonlight Butterfly (One-Per-Run Ultimate)

Turn A Gundam's final weapon is the Moonlight Butterfly — a swarm of nanomachines that dissolves all technology. In Gundam Nexus:

- The Moonlight Butterfly is available once per NG+ campaign run (not in multiplayer)
- Activation: A dedicated button in campaign combat UI, available only against boss-type enemies
- Effect: All of the boss's equipped items and attachments are removed (Striker Packs forced off, Funnels recalled, any active mode immediately canceled)
- Cannot be blocked, parried, or evaded
- Does not deal damage directly; strips defenses and allows the player to fight a "naked" boss
- The boss's base stats remain; its systems are what's removed
- Can only be used once per run — use it wisely

This system is the ultimate expression of Turn A's anti-technology stance: the most powerful weapon is not destruction but reversal.

### Narrative Revelation

NG+ reveals through interstitial story scenes:
- The player character in Nexus is not new to this conflict; they existed in the "previous era" (your first playthrough)
- The "Convergence Era" of the Nexus game universe was itself caused by a previous player's uncompleted campaign (Black History)
- Turn A is the only suit capable of "closing the loop" — its existence signals the end of the cycle
- Players who complete NG+ see an alternate ending where the Convergence AI's design specs are submitted to the Turn A's development database, canonically preventing the next era's conflict

Completing NG+ awards the "Black History Archive" cosmetic set — a weathered, ancient-looking alternate appearance for any owned suit, as if it's a relic discovered from a lost civilization.

---

## System 9: Build Fighters — Gunpla Builder

**Series Basis:** Gundam Build Fighters / Build Divers  
**System Type:** Equipment crafting and part-combination system  
**Layer:** Persistent (pre-match customization affects all match behavior)

### Design Premise

Build Fighters fundamentally repositioned Gundam from "war narrative" to "hobbyist creativity." Sei and Reiji win not because they have the most powerful suit, but because Sei's builder craftsmanship has produced a Gunpla that expresses his understanding of the machine. Gundam Nexus operationalizes this through a modular part system where mixing parts across timelines, scales, and series is not just allowed — it's the point.

### Part Categories

Every suit in Gundam Nexus is composed of 6 part slots:

| Slot | Stat Influence | Examples |
|------|---------------|---------|
| Head | Radar range (enemy distance visibility in arena) | Zaku monoeye, Wing fin antennae, Barbatos sensor unit |
| Torso | HP modifier (+/- max HP%) | Strike chest armor, Nu Gundam psycoframe torso, Epyon energy system |
| Arms | Attack damage (+/- ATK base) | Exia GN blade arm, Barbatos mace arm, Destiny dual-saber arms |
| Legs | Dodge range and speed | Freedom high-speed legs, Zaku long-range legs, Barbatos Lupus Rex high-mobility legs |
| Backpack | Special system slot (Striker Packs, Funnels, NT-D, etc.) | Fin Funnels, Strike Freedom's DRAGOON Wings, Impulse silhouette packs |
| Weapon | Attack type and damage | Beam sabers, beam rifles, heat hawks, ZZ Hi-Mega Cannon |

### Part Statistics

Each part has 5 numerical stats plus one passive ability text:

| Stat | Abbreviation | Description |
|------|-------------|-------------|
| Attack | ATK | Scales base damage of attacks |
| Defense | DEF | Reduces damage taken |
| Speed | SPD | Affects dodge distance and animation speed |
| Special | SPC | Scales special system effects (Funnel damage, Trans-Am multiplier extension, etc.) |
| Resistance | RES | Reduces status effects (combo reset resistance, Newtype Gauge drain reduction) |

Each part's passive ability is a short text description of a constant effect (e.g., "Zaku Monoeye Head — Passive: Radar reveals opponent's equipped Striker Pack at match start").

### Visual Mixing

Parts are visually distinct. A player who equips:
- RX-78-2 Head
- Barbatos Torso
- Zaku Arms
- Freedom Legs
- Nu Gundam Fin Funnel Backpack
- Exia GN Saber Weapon

...will see a composite suit that looks exactly as described — the proportions and colors of each part from its source, assembled onto a single body. The visual fidelity of the combination is part of Build Fighters' entire creative thesis. Looking wild is a feature.

### Competitive Restrictions

**Ranked (Canonical) Mode:**
- All 6 parts must come from the same canonical series or timeline
- UC must use UC parts; SEED must use SEED parts; IBO must use IBO parts
- Exception: Real Grade and Master Grade variants of canonical suits are interchangeable within the same series

**Open Mode (Unranked):**
- No restrictions; any part combination is legal
- Open mode is labeled "BUILDER'S ARENA" in the UI
- Open mode has its own separate leaderboard (Open Champion title, distinct from Duel Holder)

**Builder's Cup Events:**
- Monthly event: a specific creative constraint is given (e.g., "must include at least one part from 3 different timelines")
- Players submit builds; top builds are featured in the in-game gallery
- Winner receives a "Master Builder" emblem valid for 1 month

### Collectible Scale

847 unique parts at launch, mirroring the approximate count of distinct Gunpla kit variants as of the game's production date. Each part has 3 quality tiers (Standard, Ace, Legend), for a total of 2,541 part variants to collect. The Legend-tier version of any part has a secondary passive ability (printed in gold text in the inventory UI).

Part drop rates:
- Standard parts: drop from daily mission completion and match rewards
- Ace parts: drop from weekly challenge completion and ranked match bonuses
- Legend parts: craftable (3 Ace parts of same type = 1 Legend) or AR Bridge unlock (PG-tier kits grant Legend parts)

---

## System 10: Universal Century — Newtype Flash

**Series Basis:** Mobile Suit Gundam (original / One Year War through Char's Counterattack)  
**System Type:** Gauge-based resonance system feeding into an I-Field barrier and suit-specific passive abilities  
**Layer:** Passive/Gauge (always building; applies to every match regardless of other systems)

### Design Premise

Newtypes — humanity's next evolutionary step — perceive things others cannot: incoming attacks, emotional states, the flow of a battle. The original Gundam series codified this as "Newtype Flash," the bright resonance that occurs when two Newtypes' senses clash. In Gundam Nexus, the Newtype Flash is the visual signature of a perfect parry, and the Newtype Gauge represents accumulated resonance — the player's connection to the battle growing sharper the more precisely they fight.

### Newtype Flash (Perfect Parry Visual)

Every perfect parry (50ms window) triggers a full-screen pulse:
- A gold/white resonance aura expands from the contact point and flashes across the entire arena
- Duration: 0.3 seconds (brief but unmistakable)
- Auditory: A distinct harmonic tone (unique to the Newtype Flash; distinguishable from the normal parry sound effect)
- Purpose: Instant feedback that distinguishes perfect parries from normal parries

### Newtype Gauge

The Newtype Gauge is universal — it applies to every match for every suit unless the suit has a specific system override (Char's Sazabi drains the opponent's gauge; Full Frontal's Sinanju has Counter-Newtype effects; see below).

| Action | Gauge Change |
|--------|-------------|
| Perfect Parry | +10% |
| Normal Parry | +0% (no contribution) |
| Taking damage | -0% (damage does not drain gauge) |
| Each turn (idle drain) | -1% |

At 100% Newtype Gauge: "Newtype Awakening" available. Player can spend the full gauge to project an I-Field barrier.

**I-Field Barrier**
- Effect: Blocks the next 3 attacks completely (damage nullified; attacker takes 5% reflected damage per blocked hit)
- Duration: Active until 3 attacks are blocked or 60 seconds elapse (whichever comes first)
- Visual: Hexagonal barrier glow around the suit; each blocked attack causes one hexagonal plate to crack and fall
- Tactical note: The I-Field does not protect against the Moonlight Butterfly (Turn A, NG+ only) or any system that explicitly states "unblockable"

### Suit-Specific Newtype Interactions

**Amuro Ray's RX-78-2 (or any Amuro pilot item):**
- At Newtype Gauge 80%+: Auto-trigger "Fin Funnel Formation" — 4 auto-targeting shots fire at the opponent in sequence
- Each shot: 15% base damage, targeting accuracy 85% (15% chance each shot misses if opponent is in motion)
- Formation fires once per gauge fill cycle; does not spend the gauge (gauge continues to build/drain normally)

**Nu Gundam (RX-93):**
- Passive: Fin Funnels deploy as dedicated auto-parry system at gauge 50%+
- Effect: Once every 8 turns, one incoming attack is automatically parried (regardless of the player's gesture input) if gauge is at 50% or higher
- The auto-parry does not count toward AVJ combo tracking or ZERO System consecutive parry count (it is the Funnel parrying, not the pilot)
- At gauge 100% and I-Field active simultaneously: Nu Gundam displays "Psycho-Frame Resonance" — I-Field lasts 5 blocks instead of 3

**Char's Sazabi (MSN-04):**
- Passive "Counter-Newtype": Each hit Sazabi lands on an opponent drains 5% of the opponent's Newtype Gauge
- At the opponent's gauge 0%: Sazabi gains "Red Comet" passive — +20% speed (all movement animations faster; dodge distance extended by 25%)
- Design note: Sazabi actively dismantles opponent gauge-based strategies. Facing Sazabi while trying to build NT-D or I-Field requires accepting that your gauge will drain under pressure.
- "Red Comet" passive fades when opponent's gauge rebuilds above 30%; reasserts when drained below 30% again

**Qubeley (AMX-004):**
- Haman Karn's machine; psycho-control funnels instead of Amuro-style resonance
- Funnels: Auto-deploy at 60% gauge; 6 funnels orbit the Qubeley
- Each funnel fires every 10 turns (independent timers per funnel; 6 separate 10-turn cooldowns staggered)
- Funnel damage: 10% base each
- Funnels can be targeted and destroyed by the opponent (each funnel has 50 HP; destroyed funnel requires 20-turn cooldown to respawn)
- Haman's passive: "Purus Echo" — if opposing pilot is a Newtype-class pilot, Qubeley's funnels gain +20% accuracy

**Lalah Sune (pilot item) on any suit:**
- Lalah's Newtype Gauge fills from normal parries at +3% per normal parry (in addition to the standard +10% per perfect parry)
- She perceives danger at lower thresholds; I-Field triggers at 80% gauge instead of 100%
- I-Field blocks 4 attacks instead of 3 when triggered by Lalah's early warning

---

## System Cross-Interaction Matrix

The following is a summary of how all 10 systems interact at their most consequential intersection points. This matrix is not exhaustive (all 10 systems are always simultaneously active), but highlights the design-intentional intersections:

| System A | System B | Interaction |
|----------|----------|-------------|
| AVJ (Sys 7) | ZERO System (Sys 6) | AVJ's 80ms perfect parry window makes accumulating 5 consecutive perfect parries (ZERO trigger) achievable against skilled opponents |
| Trans-Am (Sys 5) | Combo Foundation (Sys 1) | Trans-Am preserves combo depth during its window; 3× multiplier acting on 3.0× combo depth = 9.0× base damage at peak |
| NT-D (Sys 4) | Newtype Flash (Sys 10) | NT-D consumes the Newtype Gauge (100% required); NT-D's enhanced parry window makes perfect parries easier; but NT-D is unavailable until gauge refills after use |
| Build Fighters (Sys 9) | Striker Packs (Sys 3) | Backpack slot determines Striker Pack type; Legend Aile Striker grants an additional 5% to all stats atop the Aile's standard +10% mobility |
| ZERO System (Sys 6) | NT-D (Sys 4) | Both consume Newtype Gauge; they cannot be active simultaneously. Pilots choose which ultimate to build toward. |
| Black History NG+ (Sys 8) | All systems | NG+ AI targets the player's most-used system specifically. Mono-system specialists suffer most; versatile players fare better in NG+. |
| Duel Arena (Sys 2) | Trans-Am (Sys 5) | Arena selection matters: Colony Harbor gravity shift mid-Trans-Am can disorient movement during the power window |
| AVJ Full Connection (Sys 7) | ZERO Data Override (Sys 6) | Tallgeese's "Tactical Analysis" (5 hits = advance warning) and AVJ's Full Connection (3 perfect parries = 1-turn advance) can stack — a Tallgeese with AVJ equipped gets both indicators simultaneously |

---

## Implementation Notes for Engineers

### Input Latency Compensation
The 200ms / 50ms parry windows are hardware-tuned. At game init, measure device touch-to-render latency (target < 50ms). If device latency exceeds 50ms, apply a compensation multiplier: adjusted window = displayed window × (1 + (measured_latency_ms / 100)). Cap compensation at 2.0× to prevent trivial parry windows on low-end hardware.

### AVJ HP Tax Server Validation
HP tax from AVJ parries is server-authoritative. Client reports parry event with timestamp; server deducts HP and returns authoritative health value. Client-side HP prediction is permitted for visual smoothness but reconciles to server value on next tick. Anti-cheat: HP values that diverge from server by more than 5% trigger a flag.

### Trans-Am Meter Persistence
Trans-Am Meter is stored per-match in the match state object, not the player profile. Meter value is validated server-side on activation attempt: if client claims 100% but server records < 100%, activation is rejected with client-correction packet.

### ZERO System Activation Choice
The "Accept/Reject" choice event uses a 3-second server-locked timer. If no input received in 3 seconds, Rejection is applied automatically. The timer is not extensible via any mechanism (no pause, no interrupt). Choice data is logged for telemetry (accept/reject ratio by suit/player segment used in balance monitoring).

### Moonlight Butterfly (NG+ Only)
Boss-type enemy detection uses enemy classification flags set at spawn. "Moonlight Butterfly" button renders only when `enemy.type === "boss"` and `player.ngPlusActive === true` and `player.moonlightButterflyUsed === false`. After use: `player.moonlightButterflyUsed = true` — persists until run completion resets it.

### Black History AI Memory
NG+ Convergence AI pulls from the `player.campaignHistory` object which logs: `mostUsedSuitId`, `mostUsedAttackDirection`, `mostUsedArenaId`, `fastestEnemyKillType`. These four inputs feed the AI's behavior weights. History object is read-only from the game client; modifiable only by the campaign completion event server-side.

---

## Balance Principles

**No system is universally dominant.** Each system has a meaningful counter:
- AVJ parry advantage: countered by Char's gauge drain (removes NT-D/I-Field access)
- Trans-Am: countered by opponents who survive the burst and punish the 5-turn penalty window
- NT-D: countered by fast opponents who deal damage during the 5-second activation blackout
- ZERO System: countered by opponents who randomize attack direction, reducing AI assist accuracy below 60%
- Build Fighters open builds: countered by canonical builds in Ranked mode (canonical builds have optimized synergies; mixed builds trade synergy for visual expression)

**Skill floors and ceilings are explicit, not hidden.** The 50ms perfect parry window is stated in the tutorial. The Trans-Am Burst penalty is shown in the activation animation. The ZERO System vision is explicitly a "choice." Players who understand the systems make better decisions; the game never obscures how it works.

**New players have entry points.** Exia Repair's low-risk Trans-Am, the Aile Striker's balanced simplicity, and the free-track battle pass mean a day-one player has functional options without mastering all 10 systems.

---

*Document Version: 1.0 | Gundam Nexus Pre-Production | June 2026*
