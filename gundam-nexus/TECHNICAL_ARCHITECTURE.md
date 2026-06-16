# Gundam Nexus — Technical Architecture

**Engine:** Unreal Engine 4.24 (C++)
**Base Codebase:** `infinity-blade-4/Source/InfinityBlade4/`
**Extension Module:** `infinity-blade-4/Source/GundamNexus/`
**Date:** 2026-06-16

---

## Table of Contents

1. [Architecture Overview](#1-architecture-overview)
2. [Mobile Suit System](#2-mobile-suit-system)
3. [Special System Architecture](#3-special-system-architecture)
4. [Combat Extension Layer](#4-combat-extension-layer)
5. [AI Architecture Extension](#5-ai-architecture-extension)
6. [Newtype Gauge System](#6-newtype-gauge-system)
7. [Duel Arena System](#7-duel-arena-system)
8. [Gunpla AR Bridge Technical Spec](#8-gunpla-ar-bridge-technical-spec)
9. [Progression System Extension](#9-progression-system-extension)
10. [Monetization Systems](#10-monetization-systems)
11. [Platform Architecture](#11-platform-architecture)
12. [New Source Files Directory Structure](#12-new-source-files-directory-structure)
13. [Performance Targets](#13-performance-targets)
14. [Build Targets](#14-build-targets)

---

## 1. Architecture Overview

### Design Principle

Gundam Nexus is built as a **strict extension layer** on top of the Infinity Blade 4 (IB4) codebase. No IB4 source files are modified. All new functionality lives in the `GundamNexus` Unreal module, which depends on `InfinityBlade4` as a module dependency declared in `GundamNexus.Build.cs`.

This ensures:
- IB4 combat feel (parry windows, combo chains, weapon hit detection) is preserved unmodified
- GundamNexus features are independently testable and removable
- Future IB4 patches can be merged without merge conflicts on game-specific code
- Shared combat systems benefit from IB4's existing balance tuning

### Module Dependency Graph

```
[ThirdParty / Engine]
        |
        v
[InfinityBlade4]  (Core module — read-only, never modified)
  ├── Core/IB4Types.h
  ├── Combat/{IB4CombatComponent, IB4ParrySystem, IB4WeaponBase, IB4AttackChain, IB4MagicProjectile}
  ├── Characters/{IB4Character, IB4PlayerCharacter, IB4PlayerController}
  ├── AI/{IB4EnemyBase, IB4TitanAI, IB4GodKingAI, IB4AIController}
  ├── Equipment/{IB4EquipmentBase, Sword, Shield, Ring, Helmet}
  ├── Progression/{IB4XPSystem, IB4NewGamePlus, IB4BloodlinePerkTree}
  ├── Animation/IB4AnimInstance
  └── UI/IB4HUD
        |
        | (PublicDependencyModuleNames)
        v
[GundamNexus]  (Extension module — all new code lives here)
  ├── Core/{GNTypes, GNGameMode}
  ├── Suits/{GNMobileSuit, GNGunplaPart, GNMobileSuitRegistry}
  ├── SpecialSystems/{GNSpecialSystem, GNFinFunnelSystem, GNNTDSystem, ...}
  ├── Combat/{GNComboComponent, GNParrySystem, GNMobileArsenal}
  ├── AI/{GNPilotAI, GNPilotPersonality}
  ├── Progression/{GNPilotProgression, GNBloodlineEcho, GNNewtypeGauge}
  ├── Arena/{GNDuelArena, GNMatchmaking, GNSpectatorComponent}
  ├── AR/GNARBridgeSubsystem
  └── Monetization/{GNGachaSubsystem, GNBattlePassManager, GNCollabEventManager}
        |
        v
[GundamNexusServer]  (Dedicated server build — thin overlay, no rendering)
```

### Key Inheritance Principles

| IB4 Base Class | GundamNexus Subclass | Extension Purpose |
|---|---|---|
| `UIB4EquipmentBase` | `UGNMobileSuit` | Mobile suit as equipment slot |
| `UIB4CombatComponent` | `UGNComboComponent` | Trans-Am / ZERO meter tracking |
| `UIB4ParrySystem` | `UGNParrySystem` | AVJ windows, NT resonance flash |
| `UIB4WeaponBase` | `UGNMobileArsenal` | Ranged beam / funnel attacks |
| `AIB4EnemyBase` | `UGNPilotAI` | Pilot personality system |
| `UIB4XPSystem` | `UGNPilotProgression` | Per-suit mastery tracking |
| `UIB4NewGamePlus` | `UGNBloodlineEcho` | Black History Echo (NG+) |
| `AGameModeBase` | `AGNDuelArena` | WfM Duel Arena game mode |

### Composition-Over-Inheritance Points

Not all GundamNexus systems map to an IB4 base. The following are **new `UActorComponent` additions** attached to the GundamNexus player character at spawn:

- `UGNSpecialSystem` (subclassed per suit) — suit-specific mechanics
- `UGNNewtypeGauge` — Newtype energy tracking
- `UGNARBridgeSubsystem` — UGameInstanceSubsystem for AR scanning
- `UGNGachaSubsystem` — UGameInstanceSubsystem for gacha pulls
- `UGNBattlePassManager` — UGameInstanceSubsystem for battle pass

---

## 2. Mobile Suit System

The Mobile Suit is the central equipment unit of Gundam Nexus. It replaces the IB4 sword/shield pairing as the dominant piece of equipment, inheriting from `UIB4EquipmentBase` to slot into the existing equipment binding and stat application pipeline.

### Class Hierarchy

```cpp
// infinity-blade-4/Source/GundamNexus/Suits/GNMobileSuit.h

UENUM(BlueprintType)
enum class EGundamSeries : uint8
{
    UC                  UMETA(DisplayName = "Universal Century"),
    Wing                UMETA(DisplayName = "Wing"),
    SEED                UMETA(DisplayName = "SEED"),
    DoubleOh            UMETA(DisplayName = "00"),
    IBO                 UMETA(DisplayName = "Iron-Blooded Orphans"),
    WitchFromMercury    UMETA(DisplayName = "Witch from Mercury"),
    TurnA               UMETA(DisplayName = "Turn A"),
};

USTRUCT(BlueprintType)
struct FMobileSuitStats
{
    GENERATED_BODY()

    UPROPERTY(EditDefaultsOnly) float BaseATK = 100.f;   // maps to IB4 damage multiplier
    UPROPERTY(EditDefaultsOnly) float BaseDEF = 100.f;   // maps to IB4 damage reduction
    UPROPERTY(EditDefaultsOnly) float BaseSPD = 100.f;   // maps to IB4 animation rate scale
    UPROPERTY(EditDefaultsOnly) float BaseSPC = 100.f;   // Special System charge rate
    UPROPERTY(EditDefaultsOnly) float BaseRES = 100.f;   // magic/beam resistance
};

USTRUCT(BlueprintType)
struct FGundamAR
{
    GENERATED_BODY()

    UPROPERTY(EditDefaultsOnly) FString KitBarcode;           // EAN-13 barcode string
    UPROPERTY(EditDefaultsOnly) FString KitTier;              // "HG", "MG", "PG", "PG_UNLEASHED"
    UPROPERTY(EditDefaultsOnly) bool    bUnlockedByAR = false; // false = base roster; true = AR unlock required
};

UCLASS(BlueprintType)
class GUNDAMNEXUS_API UGNMobileSuit : public UIB4EquipmentBase
{
    GENERATED_BODY()

public:
    // Series identifier — drives special system selection and AI personality
    UPROPERTY(EditDefaultsOnly, Category = "Suit")
    EGundamSeries Series;

    // Base combat stats — applied on top of IB4's equipment stat pipeline
    UPROPERTY(EditDefaultsOnly, Category = "Stats")
    FMobileSuitStats SuitStats;

    // The suit's unique mechanic component (subclassed per series/unit)
    UPROPERTY(VisibleAnywhere, Category = "Special")
    UGNSpecialSystem* SpecialSystemComponent;

    // Gunpla Builder: player-attached parts that modify FMobileSuitStats
    UPROPERTY(VisibleAnywhere, Category = "Gunpla")
    TArray<UGNGunplaPart*> EquippedParts;

    // AR integration data — populated by UGNARBridgeSubsystem on scan
    UPROPERTY(VisibleAnywhere, Category = "AR")
    FGundamAR ARIntegrationData;

    // Overrides UIB4EquipmentBase::ApplyStats() to add SuitStats on top of base
    virtual void ApplyStats(AIB4Character* TargetCharacter) override;

    // Called when any EquippedParts entry changes; recalculates effective stats
    void OnPartEquipped(UGNGunplaPart* NewPart, EPartSlot Slot);
    void OnPartUnequipped(EPartSlot Slot);

    // Returns aggregate stat modifications from all equipped parts
    FMobileSuitStats GetAggregatedPartStats() const;
};
```

### Gunpla Part System

```cpp
// infinity-blade-4/Source/GundamNexus/Suits/GNGunplaPart.h

UENUM(BlueprintType)
enum class EPartSlot : uint8
{
    Head     UMETA(DisplayName = "Head"),
    Torso    UMETA(DisplayName = "Torso"),
    Arms     UMETA(DisplayName = "Arms"),
    Legs     UMETA(DisplayName = "Legs"),
    Backpack UMETA(DisplayName = "Backpack"),
    Weapon   UMETA(DisplayName = "Weapon"),
};

USTRUCT(BlueprintType)
struct FPartStats
{
    GENERATED_BODY()

    UPROPERTY(EditDefaultsOnly) float ATKMod = 0.f;  // additive modifier to BaseATK
    UPROPERTY(EditDefaultsOnly) float DEFMod = 0.f;
    UPROPERTY(EditDefaultsOnly) float SPDMod = 0.f;
    UPROPERTY(EditDefaultsOnly) float SPCMod = 0.f;
    UPROPERTY(EditDefaultsOnly) float RESMod = 0.f;
};

UCLASS(BlueprintType)
class GUNDAMNEXUS_API UGNGunplaPart : public UObject
{
    GENERATED_BODY()

public:
    UPROPERTY(EditDefaultsOnly, Category = "Part")
    EPartSlot Slot;

    UPROPERTY(EditDefaultsOnly, Category = "Part")
    FPartStats Stats;

    // Identifies a passive behavior registered in GNPassiveAbilityRegistry
    UPROPERTY(EditDefaultsOnly, Category = "Part")
    FName PassiveAbilityName;

    // Source tier of this part (affects visual quality and stat ceiling)
    UPROPERTY(EditDefaultsOnly, Category = "Part")
    FString SourceTier; // "HG", "MG", "PG", "PG_UNLEASHED"
};
```

### Mobile Suit Registry

`UGNMobileSuitRegistry` is a static catalog (populated from a DataTable asset) holding all 20 base suits plus AR-unlock suits. It is accessed via `UGNARBridgeSubsystem` for unlock operations and `UGNGachaSubsystem` for banner pool construction.

```
Catalog entries (20 base suits):
  UC:             RX-78-2, Nu Gundam, Unicorn Gundam
  Wing:           Wing Zero (EW), Tallgeese III
  SEED:           Strike Gundam, Freedom Gundam
  DoubleOh:       00 Raiser, Exia
  IBO:            Barbatos Lupus Rex, Gusion Rebake Full City
  WitchFromMercury: Gundam Aerial, Gundam Pharact
  TurnA:          Turn A Gundam, Turn X

  AR-unlock only: PG Unleashed RX-78-2, PG Unleashed Nu Gundam,
                  PG Unleashed Strike Freedom, PG Unleashed Unicorn,
                  Nightingale (collab event)
```

---

## 3. Special System Architecture

Each mobile suit has exactly one Special System component attached at runtime. The component is selected by `UGNMobileSuit::BeginPlay()` based on the `Series` field and a DataTable row specifying the concrete subclass.

### Abstract Base

```cpp
// infinity-blade-4/Source/GundamNexus/SpecialSystems/GNSpecialSystem.h

UCLASS(Abstract, BlueprintType)
class GUNDAMNEXUS_API UGNSpecialSystem : public UActorComponent
{
    GENERATED_BODY()

public:
    // Called by UGNParrySystem when a perfect parry is confirmed
    virtual void OnPerfectParry(const FHitResult& ParryResult) {}

    // Called when the character enters combat (first hit received or dealt in 5s)
    virtual void OnCombatStart() {}

    // Called by UGNComboComponent when combo depth reaches the given value
    virtual void OnComboDepthReached(int32 Depth) {}

    // Normal Tick — runs only when character is in combat
    virtual void TickActive(float DeltaTime) {}

    // Charge level [0, 1]; subclasses set this; drives HUD display
    UPROPERTY(BlueprintReadOnly, Category = "Special")
    float ChargeLevel = 0.f;

    // Whether the special ability is currently active
    UPROPERTY(BlueprintReadOnly, Category = "Special")
    bool bIsActive = false;

protected:
    // Reference back to owning suit for stat access
    UPROPERTY()
    UGNMobileSuit* OwnerSuit;

    // Cached combat component reference (set at BeginPlay)
    UPROPERTY()
    UGNComboComponent* CombatComp;
};
```

### Concrete Implementations

#### UGNFinFunnelSystem (Nu Gundam — UC)

Fin Funnels are semi-autonomous beam emitters that activate passively at Newtype Gauge >= 50 and enter Full Burst at >= 80.

```
State machine:
  IDLE       → PASSIVE_ORBIT   (on Newtype Gauge >= 50)
  PASSIVE_ORBIT → INTERCEPT    (on incoming projectile detected within 800 UU)
  PASSIVE_ORBIT → I_FIELD      (on taking melee hit while in PASSIVE_ORBIT)
  PASSIVE_ORBIT → FULL_BURST   (on Newtype Gauge == 100 AND piloting Nu)
  FULL_BURST → COOLDOWN        (on burst duration elapsed, 8s)
  INTERCEPT  → PASSIVE_ORBIT   (after intercept sequence)
  COOLDOWN   → IDLE            (after 30s cooldown)

Fin Funnel count: 6 funnels
I-Field: blocks 3 melee hits before dispersing; visual: cyan wireframe sphere overlay
Full Burst: fires 6x simultaneous beam sweeps; damage = BaseATK * 2.4 (ignores DEF)
```

#### UGNNTDSystem (Unicorn Gundam — UC)

The NT-D (Newtype Destroyer) System is a mode-shift triggered by combat pressure accumulation.

```
Trigger: 3 consecutive hits received within 4 seconds, OR Newtype Gauge >= 80
Duration: 60 seconds active; then 120-second cooldown
Effects while active:
  - SPD multiplier: +80% (animation rate scale boost)
  - All IB4 parry windows extended: 200ms → 340ms; 50ms → 85ms (PerfectParry)
  - Beam Magnum attack becomes available (UIB4MagicProjectile subclass, Light type)
  - Visual: suit shifts from white to Destroy Mode (red/gold psycho-frame glow)
Psycho-Frame resonance: while NT-D active, nearby UC-series enemies have their
  perfect parry window reduced by 30ms (psycho-wave disruption)
```

#### UGNTransAmSystem (00 Raiser — DoubleOh)

Trans-Am is a temporary performance overdrive consuming stored GN Particles.

```
Charge: GN Particle meter fills +5/combo hit, +20/perfect parry
        drains -1/second passive while charged above 0
Activation: manual (player input), requires >= 40% charge
Active effects:
  - ATK: +100%, SPD: +50%, beam attacks leave particle trails (visual)
  - All beam attacks fire triplicate (3 projectiles in spread)
  - Duration: proportional to charge at activation; 100% charge = 15 seconds
Quantum Burst (00 Raiser exclusive): at 100% charge + Trans-Am active for >= 5s,
  player can trigger Quantum Burst: AoE GN field (1200 UU radius), stuns all
  enemies 3 seconds, drain all charge on use
```

#### UGNZEROSystem (Wing Zero — Wing)

ZERO System provides prescient combat data at the cost of pilot sanity simulation.

```
Counter: ZERO counter starts at 0; increments +1 per second while suit is equipped
Threshold states:
  0–29:   DORMANT     — no effect
  30–59:  ACTIVE      — +40% ATK, AI telegraphing is visible 300ms earlier,
                        IBO_Brutal enemies telegraph normally (ZERO sees through bluffs)
  60–89:  OVERLOAD    — +80% ATK, SPD +30%, but camera shake constant (5 UU/s drift),
                        friendly AI cannot be assigned in co-op (ZERO locks targeting)
  90–100: CRITICAL    — +120% ATK, SPD +60%; incoming damage auto-parried (not perfect),
                        duration limited: force-resets to 0 after 30s at CRITICAL
Decay: counter decays -5/second while out of combat; resets on suit swap
```

#### UGNAVJSystem (Barbatos — IBO)

The Alaya-Vijnana System is a direct neural link that enhances parry precision.

```
AVJ Grade: starts at Grade 1; upgrades through combat milestones
  Grade 1 (default):     PerfectParry window +10ms (60ms total)
  Grade 2 (50 parfect parries): PerfectParry window +25ms (75ms total)
  Grade 3 (200 perfect parries): PerfectParry window +50ms (100ms total)

Mace activation: at Grade 2+, after 5-hit combo, Barbatos can spawn
  the Great Mace (UIB4WeaponBase override) for 10 seconds:
  - 2.5x ATK multiplier, AoE cleave on every swing (cone 90 degrees)
  - Combo chain resets to 0 on spawn (mace is a new chain)

Feedback damage: at Grade 3, taking a non-parried hit deals 5% damage
  back to the attacker (neural feedback bleed)
```

#### UGNStrikerPackSystem (Strike Gundam — SEED)

Strike's Striker Pack system is a loadout swap mid-combat.

```
Packs available (player selects 2 before battle):
  Aile:    +30% SPD, unlock Jet Dash (double-tap dodge = projectile dodge)
  Sword:   +40% ATK, unlock Schwert Gewehr (IB4 sword override, extended range)
  Launcher: +20% RES, unlock Agni (single-shot, high-damage beam; 12s cooldown)

Swap cost: 1.5 second animation (vulnerable window); costs 25 Special charge
SEED Mode: activates automatically when HP <= 20%; lasts 30s
  Effects: ATK +50%, SPD +30%, all parry windows +40ms, visual: red iris VFX on HUD
```

#### UGNGUNDFormatSystem (Gundam Aerial — WitchFromMercury)

GUND-Format interfaces with the Eri AI (represented as a background subsystem).

```
Eri AI state: runs on UGNGUNDFormatSystem::TickActive() every frame
  - Monitors enemy attack telegraphs (reads EAttackDirection from enemy state)
  - Builds prediction confidence: +5% per correct prediction, -10% per wrong
  - At >= 60% confidence: displays direction prompt to player 100ms before enemy commits
  - At >= 85% confidence: auto-perfect-parry ONE attack per 15 seconds (Eri intervenes)

Permet Score: each Eri-assisted perfect parry increments Permet Score
  Score thresholds unlock Aerial expansion modes:
    Score 4:  Bit-On Form — 2 remote bits orbit, intercept one projectile/10s each
    Score 8:  Full Remote — 4 bits, intercept projectiles freely; 
              beam rifle becomes multi-lock (up to 4 targets)
    Score 12: Permet burst — one-use per battle; AoE beam sweep (same as FinFunnel Full Burst)
              resets Permet Score to 0 after use

GUND pain feedback: at Score >= 8, taking damage has 30% chance to trigger
  pain feedback (0.2s time dilation on player — warning to avoid further hits)
```

#### UGNMoonlightButterflySystem (Turn A Gundam — TurnA)

Moonlight Butterfly is an escalating nanotechnology dissolution field.

```
Ramp-up: Moonlight Butterfly charges +2% per second in combat; no manual trigger
Active threshold: >= 70% charge triggers passive aura (200 UU radius)
  Aura effect: dissolves projectiles within radius (all incoming beams nullified)
  
Full activation: manual at >= 100% charge; costs entire charge bar
  Full Butterfly:
    - 800 UU radius AoE, instant KO on standard enemies (non-boss)
    - Boss enemies: -60% of current HP, cannot kill below 1 HP
    - Visual: white nanomachine wing spread (particle system, 500-particle budget)
    - Full butterfly locks Turn A out of all attacks for 3 seconds (dissolution aftermath)
    - Cooldown: 90 seconds before charge bar refills again

Cross-series note: Turn X's Black History System (if Turn X is added as suit)
  mirrors Moonlight Butterfly but targets ally buffs instead of enemy HP.
```

#### UGNPermitScoreSystem (Aerial Alternative)

When a non-Aerial suit is equipped but the player has Permet Score banked from a
previous Aerial session, PermitScore system passively applies a legacy bonus.

```
Legacy bonus per Permet Score point: +0.5% to all stat modifiers (additive)
Maximum legacy Permet Score: 12 (same ceiling as Aerial's in-battle maximum)
This score persists across sessions and decays -1 point per 24 real-world hours
  (simulates Eri's gradual memory fade without Aerial connection)
```

---

## 4. Combat Extension Layer

### UGNComboComponent : UIB4CombatComponent

Extends IB4's combo tracking with Gundam-specific resource meters.

```cpp
// infinity-blade-4/Source/GundamNexus/Combat/GNComboComponent.h

UCLASS()
class GUNDAMNEXUS_API UGNComboComponent : public UIB4CombatComponent
{
    GENERATED_BODY()

public:
    // === Trans-Am Meter (00 Raiser) ===
    UPROPERTY(BlueprintReadOnly) float TransAmCharge = 0.f;     // [0, 100]
    UPROPERTY(BlueprintReadOnly) bool  bTransAmActive = false;

    void AddTransAmCharge(float Amount);
    void ActivateTransAm();
    void DeactivateTransAm();

    // === ZERO System Counter (Wing Zero) ===
    UPROPERTY(BlueprintReadOnly) float ZEROCounter = 0.f;       // [0, 100]

    void TickZEROCounter(float DeltaTime);
    EZEROThreshold GetZEROThreshold() const;

    // === Newtype Gauge Passthrough ===
    // UGNNewtypeGauge is a separate component; combo events notify it
    void NotifyPerfectParry();
    void NotifyNewtypeInteraction();

    // === Override hooks ===
    // Called by parent after each combo hit is confirmed
    virtual void OnComboHitConfirmed(int32 CurrentDepth, float BaseDamage) override;

    // Called by parent on parry evaluation complete
    virtual void OnParryResult(EParryResult Result, float ResponseTime) override;

private:
    UPROPERTY()
    UGNNewtypeGauge* NewtypeGaugeComp; // cached on BeginPlay

    UPROPERTY()
    UGNSpecialSystem* SpecialSystemComp; // cached on BeginPlay
};
```

### UGNParrySystem : UIB4ParrySystem

Extends parry evaluation with AVJ-enhanced windows and NT resonance flash.

```cpp
// infinity-blade-4/Source/GundamNexus/Combat/GNParrySystem.h

UCLASS()
class GUNDAMNEXUS_API UGNParrySystem : public UIB4ParrySystem
{
    GENERATED_BODY()

public:
    // AVJ-adjusted perfect parry window (base 50ms, modified by UGNAVJSystem grade)
    UPROPERTY(BlueprintReadOnly) float EffectivePerfectParryWindow = 50.f;

    // Called by UGNAVJSystem when AVJ grade changes
    void SetAVJBonus(float BonusMs);

    // NT resonance flash: if suit is UC-series and Newtype Gauge >= 60,
    // incoming attack direction is flashed on HUD 80ms before the parry window opens
    UPROPERTY(BlueprintReadOnly) bool bNTResonanceActive = false;

    virtual EParryResult EvaluateParry(float ResponseTime, EAttackDirection Direction) override;
    virtual void OnPerfectParryConfirmed(const FHitResult& HitResult) override;

private:
    // Broadcasts perfect parry to UGNComboComponent and UGNSpecialSystem
    void BroadcastPerfectParry(const FHitResult& HitResult);

    // Returns the active perfect parry window in milliseconds
    float GetActivePerfectParryWindow() const;
};
```

### UGNMobileArsenal : UIB4WeaponBase

Adds ranged beam attacks to the IB4 melee weapon framework.

```cpp
// infinity-blade-4/Source/GundamNexus/Combat/GNMobileArsenal.h

UENUM()
enum class EBeamWeaponType : uint8
{
    BeamRifle,         // Standard: single projectile, moderate damage
    TwinBusterRifle,   // Wing Zero: dual barrel, high damage, long cooldown
    BeamSaberThrow,    // Melee suit ranged option, short range
    FinFunnelArray,    // Nu Gundam: 6 simultaneous homing beams
    GNRifle,           // 00 Raiser: rapid-fire, low individual damage
    AerialBitBeam,     // Aerial: multi-lock homing, damage scales with Permet Score
};

UCLASS()
class GUNDAMNEXUS_API UGNMobileArsenal : public UIB4WeaponBase
{
    GENERATED_BODY()

public:
    UPROPERTY(EditDefaultsOnly) EBeamWeaponType RangedWeaponType;
    UPROPERTY(EditDefaultsOnly) float           BeamDamageMultiplier = 1.0f;
    UPROPERTY(EditDefaultsOnly) float           BeamCooldownSeconds  = 3.0f;
    UPROPERTY(EditDefaultsOnly) int32           MaxConcurrentBeams   = 1;

    // Fire ranged attack; spawns UIB4MagicProjectile (Light type) subclass
    void FireBeamAttack(const FVector& Origin, const FVector& Direction);

    // Fin Funnel specific: launches all 6 funnels simultaneously
    void LaunchFinFunnelArray(TArray<FVector> TargetPositions);

    // Twin Buster Rifle specific: charges for 1.5s then fires dual beam
    void BeginTwinBusterCharge();
    void ReleaseTwinBusterRifle();

    // Trans-Am modifier: if TransAm active, beam count is tripled
    void ApplyTransAmBeamModifier(bool bActive);

protected:
    virtual void BeginPlay() override;

private:
    float LastFireTime = -9999.f;
    bool  bTwinBusterCharging = false;
    float TwinBusterChargeStartTime = 0.f;
};
```

---

## 5. AI Architecture Extension

### UGNPilotAI : AIB4EnemyBase

Enemy pilots are distinct from IB4 Titans. Where IB4 enemies use `ETitanType` to drive 3-phase AI, Gundam Nexus pilots use `EGNPilotPersonality` which affects telegraphing, bluffing, and aggression curves across all 3 inherited phases.

```cpp
// infinity-blade-4/Source/GundamNexus/AI/GNPilotAI.h

UENUM(BlueprintType)
enum class EGNPilotPersonality : uint8
{
    UC_Stoic          UMETA(DisplayName = "Universal Century — Stoic"),
    Wing_Tragic       UMETA(DisplayName = "Wing — Tragic"),
    SEED_Emotional    UMETA(DisplayName = "SEED — Emotional"),
    IBO_Brutal        UMETA(DisplayName = "IBO — Brutal"),
    WfM_Tactical      UMETA(DisplayName = "Witch from Mercury — Tactical"),
};

UCLASS()
class GUNDAMNEXUS_API UGNPilotAI : public AIB4EnemyBase
{
    GENERATED_BODY()

public:
    UPROPERTY(EditDefaultsOnly, Category = "Pilot")
    EGNPilotPersonality Personality;

    UPROPERTY(EditDefaultsOnly, Category = "Pilot")
    EGundamSeries PilotSeries;

    // Pilot name for dialogue/UI
    UPROPERTY(EditDefaultsOnly, Category = "Pilot")
    FText PilotName;

    // Overrides AIB4EnemyBase attack selection to apply personality modifiers
    virtual EAttackDirection SelectNextAttackDirection() override;

    // Overrides telegraphing to allow bluffing
    virtual EAttackDirection GetTelegraphedDirection() const override;

private:
    // Returns true if this attack should be bluffed (telegraphed incorrectly)
    bool ShouldBluffTelegraph() const;

    // Phase 3 override: some personalities have special Phase 3 behaviors
    virtual void EnterPhaseThree() override;
};
```

### Pilot Personality Behaviors

| Personality | Telegraph Accuracy | Bluff Rate | Aggression Curve | Phase 3 Override |
|---|---|---|---|---|
| `UC_Stoic` | 100% honest | 0% | Linear escalation | None — Stoic to the end |
| `Wing_Tragic` | 90% honest | 10% | Slow then sudden spike | Desperation combo at 15% HP (5-hit chain) |
| `SEED_Emotional` | 60% honest | 40% | Erratic (random variance ±20%) | SEED mode mirror: gains ATK +50% burst |
| `IBO_Brutal` | 0% telegraph shown | N/A (no signal) | Aggressive from Phase 1 | Nothing changes — already at max aggression |
| `WfM_Tactical` | 80% honest | 20% (feints only) | Methodical, reads player pattern | Switches attack pattern after 3 player parries |

### Telegraph System Detail

IB4's base `AIB4EnemyBase` has a `GetTelegraphedDirection()` virtual that the `IB4HUD` reads to display the overhead/left/right indicator. `UGNPilotAI` overrides this:

```
UC_Stoic:    return SelectNextAttackDirection();   // always truth
IBO_Brutal:  return EAttackDirection::None;        // no indicator shown
SEED_Emotional: if (FMath::FRand() < 0.4f) return RandomOtherDirection();
WfM_Tactical:   if (RecentPlayerParryCount >= 3) SwapNextAttack(); return TelegraphedDir;
```

### WfM Pattern Reading

`WfM_Tactical` tracks the last 5 player parry directions. If the player parries left 3 times in a row, the AI registers this pattern and begins attacking right. Pattern memory resets each phase transition.

```cpp
// Stored in UGNPilotAI private state:
TArray<EAttackDirection, TFixedAllocator<5>> RecentPlayerParries;
int32 RecentPlayerParryCount = 0;

void UGNPilotAI::OnPlayerSuccessfulParry(EAttackDirection ParriedDirection)
{
    RecentPlayerParries.Add(ParriedDirection);
    if (RecentPlayerParries.Num() > 5) RecentPlayerParries.RemoveAt(0);
    RecentPlayerParryCount++;
}
```

---

## 6. Newtype Gauge System

The Newtype Gauge is a floating-point resource (`[0, 100]`) that powers UC-series abilities and serves as a combat mastery indicator across all series.

### Component Definition

```cpp
// infinity-blade-4/Source/GundamNexus/Progression/GNNewtypeGauge.h

UCLASS()
class GUNDAMNEXUS_API UGNNewtypeGauge : public UActorComponent
{
    GENERATED_BODY()

public:
    UPROPERTY(BlueprintReadOnly) float CurrentGauge = 0.f;      // [0, 100]

    static constexpr float MaxGauge               = 100.f;
    static constexpr float GainPerPerfectParry    = 10.f;
    static constexpr float GainPerNTInteraction   = 5.f;
    static constexpr float PassiveDrainPerSecond  = 2.f;
    static constexpr float DrainOnDamageTaken     = 15.f;

    // === Threshold queries ===
    bool IsAbove50() const  { return CurrentGauge >= 50.f; }
    bool IsAbove80() const  { return CurrentGauge >= 80.f; }
    bool IsAt100()  const   { return CurrentGauge >= 100.f; }

    // === Modification ===
    void AddGauge(float Amount);
    void DrainGauge(float Amount);
    void OnDamageTaken(float DamageAmount);  // triggers -15 drain

    // === Event broadcasts ===
    DECLARE_DYNAMIC_MULTICAST_DELEGATE(FOnThreshold50);
    DECLARE_DYNAMIC_MULTICAST_DELEGATE(FOnThreshold80);
    DECLARE_DYNAMIC_MULTICAST_DELEGATE(FOnThreshold100);
    DECLARE_DYNAMIC_MULTICAST_DELEGATE(FOnGaugeDepleted);

    UPROPERTY(BlueprintAssignable) FOnThreshold50   OnReachedThreshold50;
    UPROPERTY(BlueprintAssignable) FOnThreshold80   OnReachedThreshold80;
    UPROPERTY(BlueprintAssignable) FOnThreshold100  OnReachedThreshold100;
    UPROPERTY(BlueprintAssignable) FOnGaugeDepleted OnGaugeDepleted;

    virtual void TickComponent(float DeltaTime, ...) override;
};
```

### Threshold Effects Table

| Gauge Level | Effect | Suit Requirement |
|---|---|---|
| >= 50% | Nu Gundam Fin Funnel passive activates (orbit mode) | Nu Gundam equipped |
| >= 50% | NT resonance flash activates on `UGNParrySystem` | Any UC-series suit |
| >= 80% | Amuro Ray Full Burst available | Nu Gundam equipped |
| >= 80% | Unicorn NT-D trigger becomes available (manual) | Unicorn equipped |
| >= 80% | Eri AI prediction confidence boost +15% | Aerial equipped |
| == 100% | I-Field barrier (3 blocks) OR Newtype Awakening | Nu Gundam or Unicorn |
| == 100% | Quantum Burst available (if Trans-Am >= 5s concurrent) | 00 Raiser |
| Depleted (0%) | All special threshold effects deactivate; 5s lockout | Any |

### Newtype Rank Progression

Newtype Rank is a meta-progression stat that persists across sessions (stored in `UGNPilotProgression`). It is separate from the in-battle gauge.

```
EGNNewtypeRank:
  Latent      — starting rank; no bonus
  Awakening   — unlock criteria: 50 perfect parries total
                bonus: +0.5% Newtype Gauge gain rate
  Veteran     — unlock criteria: 500 perfect parries OR Duel Arena Rank B
                bonus: +1% gain rate, PassiveDrainPerSecond reduced to 1.5
  Ace         — unlock criteria: 2000 perfect parries OR Duel Arena Rank A
                bonus: +2% gain rate, DrainOnDamageTaken reduced to 10
  Legend      — unlock criteria: 10000 perfect parries AND Duel Arena Rank S
                bonus: +5% gain rate, gauge can exceed 100 (cap 110, overflow = bonus I-Field charge)
```

---

## 7. Duel Arena System (Witch from Mercury)

The Duel Arena is a dedicated asynchronous and real-time PvP mode inspired by the school duel system in Witch from Mercury. It is implemented as a separate `AGameModeBase` subclass that loads alongside `GNDuelArena` map assets.

### Game Mode

```cpp
// infinity-blade-4/Source/GundamNexus/Arena/GNDuelArena.h

UENUM(BlueprintType)
enum class EGNDuelRuleSet : uint8
{
    Open              UMETA(DisplayName = "Open — any suit"),
    HGOnly            UMETA(DisplayName = "HG Only — no MG/PG parts"),
    CanonicalOnly     UMETA(DisplayName = "Canonical — no Gunpla Builder parts"),
    SeriesRestricted  UMETA(DisplayName = "Series Restricted — same series only"),
};

USTRUCT(BlueprintType)
struct FGNDuelChallenge
{
    GENERATED_BODY()

    UPROPERTY() FGuid  ChallengerId;
    UPROPERTY() FGuid  DefenderId;
    UPROPERTY() FName  ArenaId;       // references a map asset name
    UPROPERTY() EGNDuelRuleSet RuleSet;
    UPROPERTY() FDateTime ExpiresAt;  // challenge expires after 24 hours
};

UCLASS()
class GUNDAMNEXUS_API AGNDuelArena : public AGameModeBase
{
    GENERATED_BODY()

public:
    // Loads a FGNDuelChallenge and initializes the duel session
    void InitializeDuel(const FGNDuelChallenge& Challenge);

    // Called when a player's HP reaches 0 or they time out (5 minutes max per duel)
    void OnDuelComplete(APlayerController* Winner, APlayerController* Loser);

    // Validates a suit against the rule set; returns false and kicks if invalid
    bool ValidateSuitForRuleSet(UGNMobileSuit* Suit, EGNDuelRuleSet Rules);

    // ELO delta calculation (standard Elo with K=32, modified for series bracket)
    static int32 CalculateEloDelta(int32 WinnerElo, int32 LoserElo, bool bWon);

protected:
    UPROPERTY() FGNDuelChallenge ActiveChallenge;
    UPROPERTY() UGNMatchmaking*  MatchmakingComponent;
};
```

### ELO Matchmaking System

```
ELO brackets:
  Bronze:   0–999     (open matchmaking, no bracket restriction)
  Silver:   1000–1499
  Gold:     1500–1999
  Platinum: 2000–2499
  Diamond:  2500–2999
  S-Rank:   3000+      (Legend Newtype Rank required to enter)

Series-restriction brackets: within SeriesRestricted ruleset, ELO pools are
  separated per series. A player has one global ELO and per-series sub-ELO.
  Matchmaking prioritizes within 150 ELO; widens to 300 after 60s wait.

Backend: Supabase table `duel_challenges` with realtime subscription.
  Challenge flow:
    1. Client A sends challenge (REST POST to `gunpla-nexus.api/v1/duel/challenge`)
    2. Supabase realtime pushes to Client B's subscription
    3. Client B accepts → dedicated server slot allocated (GundamNexusServer build)
    4. Both clients connect to dedicated server via UE4 Online Subsystem
    5. Duel result pushed back to Supabase; ELO updated server-side
```

### Spectator Mode

```cpp
// infinity-blade-4/Source/GundamNexus/Arena/GNSpectatorComponent.h

UCLASS()
class GUNDAMNEXUS_API UGNSpectatorComponent : public UActorComponent
{
    GENERATED_BODY()

public:
    // Max spectators per duel session; enforced on the dedicated server
    static constexpr int32 MaxSpectators = 50;

    // Spectators receive a deferred snapshot (200ms delay) to prevent timing cheats
    static constexpr float SpectatorDelayMs = 200.f;

    // Spectator joins: server adds to spectator list, begins sending deferred state
    bool RequestJoinAsSpectator(APlayerController* Spectator);

    // Camera: spectator can switch between free-cam, follow-challenger, follow-defender
    void SetSpectatorCameraMode(EGNSpectatorCamera Mode);

    // UI: spectator HUD shows both players' HP/gauge/special meter
    void UpdateSpectatorHUD(const FGNDuelState& State);
};
```

---

## 8. Gunpla AR Bridge Technical Spec

The AR Bridge connects physical Gunpla kit purchases to in-game mobile suit unlocks. The system operates through barcode scanning (all kits) and NFC reading (PG Unleashed exclusively).

### Barcode Scan Flow

```
1. Player opens AR Scanner in-game (UGNARBridgeSubsystem::BeginScan())
2. Platform camera feed activated (FIOSInputInterface on iOS, Android camera API)
3. UE4 UUserWidget overlays barcode scan reticle (IB4HUD extended widget)
4. On barcode detected: UGNARBridgeSubsystem::OnBarcodeDetected(FString Barcode)

API call:
  POST https://gunpla-nexus.api/v1/scan
  Headers: Authorization: Bearer {account_jwt}
  Body: { "barcode": "4573102610010", "account_id": "uuid-v4-string" }

Response (200 OK):
  {
    "suit_id":       "nu_gundam_pg_unleashed",
    "tier":          "PG_UNLEASHED",
    "digital_bonus": ["fin_funnel_gold_skin", "newtype_gauge_boost_permanent_5pct"]
  }

Response (409 Conflict):
  { "error": "BARCODE_ALREADY_CLAIMED", "claimed_by": "same_account"|"other_account" }

Response (404 Not Found):
  { "error": "BARCODE_NOT_RECOGNIZED" }
```

### NFC Flow (PG Unleashed Only)

```
PG Unleashed boxes include an NXP NTAG216 NFC chip embedded in the box insert.
Chip payload: JSON { "suit_id": string, "nfc_token": string (256-bit HMAC) }

iOS path:
  FGenericPlatformMisc::GetDeviceMake() → "Apple"
  → UGNARBridgeSubsystem::BeginNFCScan() → calls iOS CoreNFC via JNI-equivalent
    (UE4 ObjC++ bridge in GNARBridgeSubsystem.cpp platform section)

Android path:
  FGenericPlatformMisc::GetDeviceMake() → not "Apple"
  → BeginNFCScan() → calls Android NFC API via JNI

Server validation:
  POST https://gunpla-nexus.api/v1/nfc-verify
  Body: { "suit_id": string, "nfc_token": string, "account_id": uuid }
  Server validates HMAC with shared secret; never exposes secret to client
```

### Unlock Flow

```cpp
// infinity-blade-4/Source/GundamNexus/AR/GNARBridgeSubsystem.h

UCLASS()
class GUNDAMNEXUS_API UGNARBridgeSubsystem : public UGameInstanceSubsystem
{
    GENERATED_BODY()

public:
    // Begin camera-based barcode scan session
    void BeginScan();

    // Begin NFC scan session (PG Unleashed only)
    void BeginNFCScan();

    // Callback from platform camera/NFC layer
    void OnBarcodeDetected(const FString& Barcode);
    void OnNFCPayloadReceived(const FString& NFCJson);

    // Called after server returns a valid scan result
    void OnScanComplete(const FScanResult& Result);

    // Unlock the suit: add to player's UGNMobileSuitRegistry entry
    void UnlockSuit(const FString& SuitId, const TArray<FString>& DigitalBonuses);

    // Check if a suit is already unlocked for the current account
    bool IsSuitUnlocked(const FString& SuitId) const;

private:
    // Pending HTTP request handle (UE4 HTTP module)
    TSharedPtr<IHttpRequest> PendingRequest;

    // Cache of unlocked suit IDs (populated at game startup from server)
    TSet<FString> UnlockedSuitIds;
};
```

### Fraud Prevention

```
One-time use per barcode:
  - Server maintains `scanned_barcodes` table with columns:
      barcode TEXT PRIMARY KEY, account_id UUID, scanned_at TIMESTAMPTZ
  - On second scan of same barcode: 409 Conflict returned regardless of account

Account binding:
  - JWT token required on all AR API calls; server extracts account_id from JWT
  - Barcode → account_id mapping is immutable once written

NFC token validation:
  - nfc_token is HMAC-SHA256(suit_id + nonce) signed with server-side secret
  - Client never sees the secret; cannot forge NFC payloads
  - nonce is single-use; server marks it consumed on first verify

Rate limiting:
  - Max 10 scan attempts per account per hour (prevents brute-force barcode guessing)
  - Failed scans (404) count toward rate limit
```

---

## 9. Progression System Extension

### UGNPilotProgression : UIB4XPSystem

```cpp
// infinity-blade-4/Source/GundamNexus/Progression/GNPilotProgression.h

USTRUCT()
struct FGNSuitMastery
{
    GENERATED_BODY()

    FString SuitId;
    int32   TotalBattles       = 0;
    int32   TotalPerfectParries = 0;
    float   TotalDamageDealt   = 0.f;
    int32   MasteryLevel       = 0;    // [0, 10]
    float   MasteryXP          = 0.f;

    // Each mastery level unlocks a suit-specific cosmetic or passive bonus
    // Level thresholds: 0=0, 1=500, 2=1500, 3=3000, 4=5000, 5=8000,
    //                   6=12000, 7=17000, 8=23000, 9=30000, 10=40000 XP
};

UCLASS()
class GUNDAMNEXUS_API UGNPilotProgression : public UIB4XPSystem
{
    GENERATED_BODY()

public:
    // Per-suit mastery tracking
    UPROPERTY(SaveGame) TMap<FString, FGNSuitMastery> SuitMasteries;

    // Newtype Rank (meta-progression)
    UPROPERTY(SaveGame) EGNNewtypeRank NewtypeRank = EGNNewtypeRank::Latent;

    // Duel Arena ELO (global)
    UPROPERTY(SaveGame) int32 DuelArenaELO = 1000;

    // Per-series sub-ELO for SeriesRestricted bracket
    UPROPERTY(SaveGame) TMap<EGundamSeries, int32> SeriesELO;

    // Add mastery XP after a battle; handles level-up logic
    void AddSuitMasteryXP(const FString& SuitId, float XPAmount);

    // Evaluate and advance Newtype Rank if criteria met
    void EvaluateNewtypeRank();

    // Override UIB4XPSystem::AddXP() to also route to active suit mastery
    virtual void AddXP(float Amount) override;
};
```

### UGNBloodlineEcho : UIB4NewGamePlus

"Black History Echo" is the Gundam Nexus name for New Game+, referencing Turn A's Black History records of past civilizations.

```
Black History Echo mechanics (extends IB4 NG+ which already scales enemy HP/ATK):
  - IB4 NG+ scaling applies (inherited, unmodified)
  - Additional GN overlay: on Echo start, player retains:
      * All unlocked mobile suits (including AR unlocks)
      * All Gunpla Builder parts
      * Newtype Rank
      * Suit Mastery levels (but not suit mastery XP — starts fresh)
      * Duel Arena ELO (unchanged)
  - Enemy pilots gain "Black History Awakening" if player is on Echo 3+:
      * All pilots gain Newtype Gauge equivalent (+30% accuracy on attack chain selection)
      * IBO_Brutal pilots gain limited telegraph (20% chance) — they are "remembered" by Turn A's history
  - Black History Echo depth is displayed in Roman numerals: Echo I, II, III...
  - Max tracked Echo depth: XV (beyond that, difficulty asymptotes)

UGNBloodlineEcho:
  - Inherits UIB4NewGamePlus::BeginNewGamePlus()
  - Overrides UIB4NewGamePlus::GetEnemyScaleMultiplier() to add Echo-depth escalation
  - New method: GetBlackHistoryBonuses() → returns FGNBlackHistoryState
```

---

## 10. Monetization Systems (Technical)

All monetization state is **server-authoritative**. Client code holds read-only copies synchronized at login and after each transaction. The client never computes pull results, pity counters, or ownership flags — these are always server responses.

### UGNGachaSubsystem

```cpp
// infinity-blade-4/Source/GundamNexus/Monetization/GNGachaSubsystem.h

UCLASS()
class GUNDAMNEXUS_API UGNGachaSubsystem : public UGameInstanceSubsystem
{
    GENERATED_BODY()

public:
    // Current banner data (fetched from server at startup)
    UPROPERTY(BlueprintReadOnly) TArray<FGNBannerData> ActiveBanners;

    // Client-side pity display (read-only; authoritative value is server-side)
    UPROPERTY(BlueprintReadOnly) int32 DisplayPityCount = 0;

    // Pull history (last 100 pulls, for display; no gameplay logic runs on this)
    UPROPERTY(BlueprintReadOnly) TArray<FGNPullRecord> PullHistory;

    // Initiates a gacha pull; result delivered via callback after server round-trip
    void RequestPull(const FGuid& BannerId, int32 PullCount,
                     TFunction<void(TArray<FGNPullResult>)> OnComplete);

    // Refresh banner data from server
    void RefreshBanners(TFunction<void()> OnComplete);

    // Called on login: sync pity counter and pull history from server
    void SyncStateFromServer();
};
```

### UGNBattlePassManager

```cpp
// infinity-blade-4/Source/GundamNexus/Monetization/GNBattlePassManager.h

USTRUCT()
struct FGNBattlePassReward
{
    GENERATED_BODY()
    int32   Tier;
    FString RewardType;  // "suit_skin", "gunpla_part", "currency", "title"
    FString RewardId;
    bool    bPremiumOnly;
};

UCLASS()
class GUNDAMNEXUS_API UGNBattlePassManager : public UGameInstanceSubsystem
{
    GENERATED_BODY()

public:
    UPROPERTY(BlueprintReadOnly) int32  CurrentSeason;
    UPROPERTY(BlueprintReadOnly) int32  CurrentTier = 0;       // [0, 100]
    UPROPERTY(BlueprintReadOnly) float  CurrentTierXP = 0.f;
    UPROPERTY(BlueprintReadOnly) bool   bHasPremiumPass = false;
    UPROPERTY(BlueprintReadOnly) TArray<FGNBattlePassReward> SeasonRewards;
    UPROPERTY(BlueprintReadOnly) TSet<int32> ClaimedTiers;

    // Claim a reward tier; server validates ownership before granting
    void ClaimReward(int32 Tier, TFunction<void(bool bSuccess)> OnComplete);

    // Add XP to battle pass; XP sources: battles, duel wins, daily missions
    void AddBattlePassXP(float XPAmount);

    // Sync full battle pass state from server (on login and season transitions)
    void SyncFromServer();
};
```

### UGNCollabEventManager

```cpp
// infinity-blade-4/Source/GundamNexus/Monetization/GNCollabEventManager.h

UCLASS()
class GUNDAMNEXUS_API UGNCollabEventManager : public UGameInstanceSubsystem
{
    GENERATED_BODY()

public:
    // Active collab events; populated from server config at startup
    UPROPERTY(BlueprintReadOnly) TArray<FGNCollabEvent> ActiveEvents;

    // Check if a collab event is active right now (server time, not client)
    bool IsEventActive(const FName& EventId) const;

    // Inject a collab suit into the active banner pool temporarily
    void InjectCollabSuit(const FString& SuitId, const FGNCollabEvent& Event);

    // Remove all suits associated with an expired event
    void ExpireCollabEvent(const FName& EventId);

    // Time-gated content: return remaining seconds for an event
    float GetEventRemainingSeconds(const FName& EventId) const;
};
```

### Server Authority Guarantees

```
Gacha pulls:
  - Pull result generated server-side using cryptographically random seed
  - Pity counter stored in `player_pity` Supabase table; never trusted from client
  - Client sends: { banner_id, pull_count, account_jwt }
  - Server returns: { results: [...], new_pity_count, updated_inventory }

Battle Pass:
  - XP is posted to server after each battle with a signed battle report
  - Battle report includes: battle_id (UUID, server-issued), duration, result, xp_earned
  - Server validates XP against known battle parameters (anti-cheat floor/ceiling check)
  - Tier advancement and reward grant happen server-side

Collab Events:
  - Event activation flag is server-read-only; client cannot enable events
  - Event suits are injected into banner pools via server config push (Supabase realtime)
  - Client-side FGNCollabEvent.bIsActive is set from server JSON, never from client logic
```

---

## 11. Platform Architecture

### Mobile (Primary Platform)

```
iOS:
  UE4 packaging target: iOS 15.0+ (covers A13 Bionic and later)
  Touch gesture mapping:
    Swipe Up:        Overhead attack
    Swipe Left:      Left attack
    Swipe Right:     Right attack
    Swipe Down:      Parry stance
    Double-tap:      Dodge
    Hold + Swipe:    Charged attack (hold 0.5s then swipe)
    Pinch:           Zoom camera (arena spectator only)
  Input handler: FIOSInputInterface gesture recognizers bridged to IB4's EAttackDirection
  NFC: CoreNFC framework (iOS 11+); bridged via ObjC++ in GNARBridgeSubsystem

Android:
  Min SDK: API 31 (Android 12); targets Snapdragon 8 Gen 2+
  Touch gestures: same as iOS; implemented via Android InputManager
  NFC: android.nfc.NfcAdapter; called via JNI from GNARBridgeSubsystem
  GPU: Vulkan renderer (UE4 Vulkan backend); OpenGL ES 3.2 fallback for older devices
```

### PC (Steam)

```
Keyboard/Mouse gesture emulation:
  W + Space:   Overhead attack
  A:           Left attack (hold 0.3s = charged left)
  D:           Right attack (hold 0.3s = charged right)
  S:           Parry stance (hold = sustained parry)
  Left Shift:  Dodge
  Q:           Ranged beam fire
  E:           Special System activation
  Tab:         Striker Pack swap (Strike Gundam only)
  Mouse look:  Camera orbit

Input abstraction: UGNInputRouter maps platform-specific gesture events to
  IB4's existing input action names (IB4Attack_Overhead, IB4Attack_Left, etc.)
  so the combat system requires zero changes for PC input.

Steam integration:
  Achievements: map to IB4BloodlinePerkTree milestones + GN-specific milestones
  Steam Cloud: mirrors UGNSaveGame data (secondary to Supabase cloud save)
```

### Cross-Platform Account

```
Auth provider: Supabase Auth (email/password + OAuth: Google, Apple, Steam)
Account JWT: issued by Supabase; passed in Authorization header to all API calls
  including gunpla-nexus.api (which validates JWT against Supabase public key)

Unified account data (server-side, Supabase Postgres):
  - player_profile:      Newtype Rank, ELO, display name, avatar suit
  - player_inventory:    unlocked suits, gunpla parts, skins
  - player_mastery:      per-suit mastery XP and level
  - player_pity:         per-banner gacha pity counter
  - player_battlepass:   current season tier and claimed rewards
  - player_duel_history: last 100 duel results

Platform linking: one Supabase account can have linked Steam + Apple + Google identities.
  Progression is shared; platform-exclusive cosmetics are flagged in inventory table.
```

### Save System

```cpp
// UGNSaveGame : USaveGame
// Stores local cache of server state for offline display
// On launch: loads from UE4 SaveGame slot, then Supabase sync overwrites with server truth

UCLASS()
class GUNDAMNEXUS_API UGNSaveGame : public USaveGame
{
    GENERATED_BODY()

public:
    // Cached progression (overwritten from server on login)
    UPROPERTY(SaveGame) FGNPlayerProfile  CachedProfile;
    UPROPERTY(SaveGame) TArray<FString>   CachedUnlockedSuits;
    UPROPERTY(SaveGame) TArray<FGNSuitMastery> CachedMasteries;
    UPROPERTY(SaveGame) int32             CachedBattlePassTier = 0;

    // Local settings (not synced to server)
    UPROPERTY(SaveGame) float MusicVolume  = 1.0f;
    UPROPERTY(SaveGame) float SFXVolume    = 1.0f;
    UPROPERTY(SaveGame) bool  bHapticOn    = true;
    UPROPERTY(SaveGame) FString PreferredSuitId;
};
```

### Analytics

```
Provider: Custom UE4 Analytics provider (IAnalyticsProvider implementation)
  - Records events as JSON to a local buffer
  - Flushes every 60 seconds OR on app background
  - Endpoint: POST https://gunpla-nexus.api/v1/analytics/events

Key events tracked:
  battle_start:      { suit_id, pilot_personality, arena_id }
  battle_end:        { suit_id, result, duration_s, perfect_parry_count }
  special_activated: { suit_id, special_type, charge_at_activation }
  ar_scan:           { tier, success: bool } (no barcode stored for privacy)
  gacha_pull:        { banner_id, pull_count } (no result stored client-side)
  duel_complete:     { result, elo_delta, rule_set }
  newtype_threshold: { threshold: 50|80|100, suit_id, time_in_battle }
```

---

## 12. New Source Files Directory Structure

```
infinity-blade-4/Source/GundamNexus/
├── GundamNexus.Build.cs                  ← Module definition; depends on InfinityBlade4
│
├── Core/
│   ├── GNTypes.h                         ← EGundamSeries, EGNDuelRuleSet, FMobileSuitStats,
│   │                                        EPartSlot, EGNNewtypeRank, EGNPilotPersonality,
│   │                                        EBeamWeaponType, EZEROThreshold, FScanResult
│   └── GNGameMode.h / GNGameMode.cpp     ← Base game mode; extends AGameModeBase;
│                                            hooks into IB4 game flow for non-duel play
│
├── Suits/
│   ├── GNMobileSuit.h / GNMobileSuit.cpp         ← UIB4EquipmentBase subclass
│   ├── GNGunplaPart.h / GNGunplaPart.cpp         ← UObject; stat modifier pieces
│   └── GNMobileSuitRegistry.h / .cpp             ← Catalog of 20 base suits + AR unlocks
│                                                     populated from DataTable asset
│
├── SpecialSystems/
│   ├── GNSpecialSystem.h                         ← Abstract UActorComponent base
│   ├── GNFinFunnelSystem.h / .cpp                ← Nu Gundam; I-Field; Full Burst
│   ├── GNNTDSystem.h / .cpp                      ← Unicorn; mode-shift; Destroy Mode
│   ├── GNTransAmSystem.h / .cpp                  ← 00 Raiser; GN particle meter; Quantum Burst
│   ├── GNZEROSystem.h / .cpp                     ← Wing Zero; counter states; CRITICAL
│   ├── GNAVJSystem.h / .cpp                      ← Barbatos; parry window grades; Great Mace
│   ├── GNStrikerPackSystem.h / .cpp              ← Strike; pack swap; SEED Mode mirror
│   ├── GNGUNDFormatSystem.h / .cpp               ← Aerial; Eri AI; Permet Score
│   └── GNMoonlightButterflySystem.h / .cpp       ← Turn A; nanotech aura; Full Butterfly
│
├── Combat/
│   ├── GNComboComponent.h / .cpp         ← UIB4CombatComponent subclass; Trans-Am/ZERO meters
│   ├── GNParrySystem.h / .cpp            ← UIB4ParrySystem subclass; AVJ windows; NT flash
│   └── GNMobileArsenal.h / .cpp          ← UIB4WeaponBase subclass; beam weapon types
│
├── AI/
│   ├── GNPilotAI.h / .cpp               ← AIB4EnemyBase subclass; personality-driven AI
│   └── GNPilotPersonality.h             ← EGNPilotPersonality enum + behavior constants
│
├── Progression/
│   ├── GNPilotProgression.h / .cpp      ← UIB4XPSystem subclass; suit mastery; Newtype Rank
│   ├── GNBloodlineEcho.h / .cpp         ← UIB4NewGamePlus subclass; Black History Echo
│   └── GNNewtypeGauge.h / .cpp          ← UActorComponent; gauge resource; threshold events
│
├── Arena/
│   ├── GNDuelArena.h / .cpp             ← AGameModeBase subclass; WfM duel rules
│   ├── GNMatchmaking.h / .cpp           ← ELO logic; Supabase realtime duel challenge flow
│   └── GNSpectatorComponent.h / .cpp   ← Deferred snapshot spectator; max 50; camera modes
│
├── AR/
│   └── GNARBridgeSubsystem.h / .cpp    ← UGameInstanceSubsystem; barcode + NFC flow
│
└── Monetization/
    ├── GNGachaSubsystem.h / .cpp        ← UGameInstanceSubsystem; banner + pull API
    ├── GNBattlePassManager.h / .cpp     ← UGameInstanceSubsystem; season progress + rewards
    └── GNCollabEventManager.h / .cpp    ← UGameInstanceSubsystem; time-gated collab events
```

### GundamNexus.Build.cs (summary)

```csharp
// infinity-blade-4/Source/GundamNexus/GundamNexus.Build.cs

PublicDependencyModuleNames.AddRange(new string[] {
    "Core", "CoreUObject", "Engine", "InputCore",
    "InfinityBlade4",     // IB4 base systems
    "HTTP",               // AR Bridge API calls
    "Json",               // JSON parsing for scan responses
    "JsonUtilities",
    "OnlineSubsystem",    // Duel Arena matchmaking
    "OnlineSubsystemUtils",
    "UMG",                // Extended HUD widgets
});

PrivateDependencyModuleNames.AddRange(new string[] {
    "Analytics",          // Custom analytics provider
    "Slate", "SlateCore",
});
```

---

## 13. Performance Targets

| Metric | Target | Notes |
|---|---|---|
| Frame rate (mobile) | 60 fps | iOS A15 Bionic+; Android Snapdragon 8 Gen 2+ |
| Frame rate (PC) | 120 fps | Uncapped on high-end; 60 fps floor |
| Suit polygon count | Max 80,000 triangles per suit | vs. IB4 base suits at ~40,000 |
| Beam effect particles | Max 500/frame (Trans-Am Full Burst) | 200 standard; 500 peak |
| Fin Funnel beams | 6 simultaneous homing projectiles | Recycled via object pool |
| Network latency (Duel) | < 100ms acceptable; < 60ms target | Dedicated server, same region |
| Spectator delay | 200ms deferred snapshot | Anti-cheat; built into UGNSpectatorComponent |
| Arena load time | < 4 seconds on device | Streaming assets; no full-level load |
| Initial download size | < 150 MB | Base suits + UC content; others stream |
| Streaming asset chunks | ~50 MB per series pack | Downloaded on first suit unlock |
| Memory budget (mobile) | < 800 MB RAM total | UE4 memory pool; suits unloaded when not active |
| Texture resolution | Max 2048×2048 per suit (mobile) | 4096 on PC; runtime quality setting |
| Battle save checkpoint | < 0.5 seconds to write | `UGNSaveGame::AsyncSave()` non-blocking |

### Optimization Notes

```
Mobile particle budget is enforced by UGNParticleGuard (a tick function that monitors
  GPU particle count per frame and LODs active effects if budget is exceeded).

Gunpla Builder part combinations: stat aggregation is O(6) (max 6 part slots);
  no performance concern. Mesh combination for part swapping uses UE4 Runtime
  Mesh Component to avoid full actor respawn.

Fin Funnel pooling: 6 funnel actors are pre-spawned at arena load and reused;
  they are repositioned rather than spawned/destroyed, avoiding GC pressure.

Duel Arena network: Unreal Online Subsystem with custom Supabase signaling for
  connection establishment; peer-to-peer is NOT used (server-authoritative only).
```

---

## 14. Build Targets

```
GundamNexus           — Mobile shipping binary (iOS + Android)
                        Includes: GundamNexus + InfinityBlade4 modules
                        Excludes: Editor tooling, server-only code
                        Packaging: UE4 Project Launcher; 
                        iOS: Xcode archive; Android: Gradle APK/AAB

GundamNexusEditor     — UE4 Editor build for content creation
                        Includes: GundamNexusEditor module (DataTable tools,
                          suit preview widget, AR barcode test panel)
                        Used by: art team for suit rigging, combat team for
                          combo chain tuning via IB4AttackChain DataTables

GundamNexusServer     — Dedicated server for Duel Arena
                        Headless build (no rendering modules)
                        Hosted on: Linux x86_64 (Docker container, Rocket SDK deployment)
                        Includes: GundamNexus + InfinityBlade4 modules; 
                          UGNDuelArena, UGNMatchmaking, server-side ELO computation
                        Excludes: AR subsystem, gacha client code, UI modules

GundamNexusPC         — Steam desktop binary
                        Built from same source as GundamNexus mobile;
                        PLATFORM_PC preprocessor enables keyboard input routes
                        and disables mobile-only NFC/camera code paths
```

### CI/CD Pipeline

```
Build pipeline (GitHub Actions, referencing .github/workflows/ in rocket-craft repo):
  1. On push to `claude/master-integration`:
       a. Compile check: UnrealBuildTool GundamNexus (Editor target, Linux)
       b. Run unit tests: UE4 Automation Framework tests in GundamNexus/Tests/
       c. Static analysis: clang-tidy on GundamNexus Source/ 
  2. On tag `release/*`:
       a. Full cook + package: GundamNexus (iOS + Android)
       b. Package: GundamNexusServer (Linux)
       c. Upload: TestFlight (iOS) + Firebase App Distribution (Android)
  3. On merge to main:
       a. Deploy: GundamNexusServer containers via Rocket SDK deployment tooling
       b. Notify: Supabase config updated with new server endpoint

Module boundary enforcement:
  InfinityBlade4 module has no dependency on GundamNexus.
  If any InfinityBlade4 header includes a GundamNexus header, CI fails.
  This is enforced via UnrealBuildTool module dependency graph check.
```

---

*Document version: 1.0 — Generated 2026-06-16*
*Maintained by: Gundam Nexus engineering team*
*IB4 base codebase: `infinity-blade-4/Source/InfinityBlade4/` (read-only)*
*GN extension module: `infinity-blade-4/Source/GundamNexus/` (all new code)*
