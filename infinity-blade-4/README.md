# Infinity Blade 4

> An open-source AAA mobile action game built on Unreal Engine 4.24

## Overview

IB4 is the spiritual successor to ChAIR Entertainment's Infinity Blade trilogy — the groundbreaking iOS franchise that redefined what a mobile device could render in real time. This open-source implementation rebuilds IB4 from scratch on UE4.24 using the SurvivalGame sample as its foundation, then extends it with a complete gesture-based sword combat system, a deep bloodline progression tree, and a fully-featured PWA HUD that runs independently in any modern browser.

The project is intentionally self-contained. Every subsystem is implemented in C++ with clean Blueprint exposure, so designers can iterate on AI behaviors, combat tuning, and arena layouts without touching C++. All C++ classes follow Unreal Engine 4 coding conventions: UCLASS/USTRUCT/UENUM macros, UPROPERTY replication where appropriate, and BlueprintNativeEvents for the hooks designers most commonly need to override.

---

## Features

### Gesture-Based Sword Combat
- **8-directional swipe detection** translated to three attack lanes: Overhead, Left, Right
- Touch start/end positions are captured in `AIB4PlayerController` and rereaddressed into `EAttackDirection` values using a configurable minimum swipe distance
- Keyboard/gamepad fallback bindings for desktop development
- Combo chain up to 4 hits with increasing damage multipliers: 1× → 1.5× → 2× → 3×

### Bloodline Rebirth System
- 15-perk progression tree tracked in `UIB4BloodlineComponent`
- Persistent stat bonuses (`FBloodlineStats`) carry across death/rebirth cycles: Attack, Defense, Magic, XP multiplier
- Non-linear XP curve configurable per blueprint: `XPRequired = XPBase * (Level ^ XPExponent)`
- Full-screen cinematic transition when a new bloodline level activates

### Equipment System
- 5 equipment slots: Weapon, Shield, Helmet, Armor, Ring
- 22+ weapon configurations plus 15+ armor sets
- Gem socketing system managed by `UIB4EquipmentComponent`
- Stat contributions rereaddressed at equip time, no per-tick overhead

### Arena & Enemy AI
- 5 unique arena maps with environmental hazards (spike traps, fire geysers, falling pillars)
- 15 enemy archetypes including the 7 Titan boss types: Warlord, Knight, Assassin, Berserker, Sorcerer, Defiler, and God King
- Phase-based boss AI: Titans change attack patterns and unlock new abilities at health thresholds (75%, 50%, 25%)
- Enemy AI implemented with UE4's Behavior Tree and `UAIModule`

### Magic System
- Three elemental types: Fire, Lightning, Ice — unlocked through bloodline progression
- Projectile physics handled by `AIB4MagicProjectile` with element-specific collision responses
- Mana cost system draining from `AIB4Character::Magic` resource pool

### PWA Combat HUD
- TypeScript HUD runs in any modern browser via Progressive Web App manifests
- Real-time health/mana bars, combo counter, magic type indicators
- Bloodline level display with animated perk unlock notifications
- Equipment loadout panel for pre-battle gear selection
- Fully offline-capable via Service Worker caching

### Blueprint T3D Files
Five ready-to-import Blueprint assets covering the highest-value systems:
- `BP_CombatChain.t3d` — visual combo chain editor
- `BP_BloodlineLevelUp.t3d` — rebirth sequence controller
- `BP_EquipmentPickup.t3d` — world pickup with auto-equip logic
- `BP_MagicCasting.t3d` — elemental projectile spawner
- `BP_TitanBossFight.t3d` — phase-based boss encounter manager

---

## Architecture

### C++ Class Hierarchy

```
ACharacter
└── AIB4Character          (Base: Health/Magic/Bloodline, TakeDamage, OnDeath)
    └── AIB4PlayerCharacter (Camera rig, input bindings, XP management)

APlayerController
└── AIB4PlayerController   (Touch/swipe detection, pause, bloodline restart)

AHUD
└── AIB4HUD                (UMG widget host, debug overlay, combo bridge)

UAnimInstance
└── UIB4AnimInstance       (State machine driver, montage helpers)

UActorComponent
├── UIB4CombatComponent    (Attack, parry, dodge, magic, combo state machine)
├── UIB4EquipmentComponent (Slot management, stat contributions)
└── UIB4BloodlineComponent (Perk tree, rebirth, inherited stats)

AActor
└── AIB4WeaponBase         (Hitbox, damage type, swing VFX)

UObject
├── UIB4AttackChain        (Data: direction × combo level montage table)
└── UIB4ParrySystem        (Timing windows, perfect parry detection)

AProjectile → AIB4MagicProjectile (Elemental projectile: Fire/Lightning/Ice)
AGameMode   → AIB4GameMode        (Arena lifecycle, titan spawning, restart)
```

### Data Flow: Input → Combat → Animation

```
Touch Event (iOS/Android)
    → AIB4PlayerController::OnInputTouchEnd()
    → DetectSwipeDirection() → EAttackDirection
    → AIB4PlayerCharacter::OnAttackInput(Direction)
    → UIB4CombatComponent::BeginAttack(Dir)
    → SelectAttackMontage(Dir, ComboCount)
    → UIB4AnimInstance::TriggerAttackAnim(Dir, Depth)
    → UAnimInstance::Montage_Play()
    → Anim Notify: OnAttackHit(Target)
    → TakeDamage() → AIB4GameMode::OnEnemyDefeated()
    → AIB4HUD::UpdateComboDisplay(Count, Multiplier)
```

### Key Types (`IB4Types.h`)

```cpp
// Attack direction rereaddressed from swipe gesture
UENUM(BlueprintType)
enum class EAttackDirection : uint8 { Overhead, Left, Right };

// Elemental magic unlocked via bloodline
UENUM(BlueprintType)
enum class EMagicType : uint8 { Fire, Lightning, Ice };

// Per-bloodline inherited stat bonuses
USTRUCT(BlueprintType)
struct FBloodlineStats
{
    float AttackBonus;   // Flat physical damage bonus
    float DefenseBonus;  // Flat damage reduction
    float MagicBonus;    // Flat magic damage bonus
    float XPMultiplier;  // Multiplicative XP gain
};
```

---

## Setup

### Prerequisites
- Unreal Engine 4.24 (install via Epic Games Launcher or build from source)
- Visual Studio 2019 with C++ game development workload (Windows) or Xcode 11 (macOS/iOS)
- Android NDK r14b if targeting Android
- Node.js 16+ for PWA HUD development

### Integrate into UE4.24 SurvivalGame Base

1. Clone this repository alongside your UE4.24 SurvivalGame checkout:
   ```bash
   git clone <repo-url> rocket-craft
   cd rocket-craft
   ```

2. Copy the `infinity-blade-4/Source/InfinityBlade4/` directory tree into your SurvivalGame project's `Source/` folder, or use `infinity-blade-4/InfinityBlade4.uproject` as a standalone project.

3. Generate project files:
   ```bash
   # Windows
   "C:/Program Files/Epic Games/UE_4.24/Engine/Binaries/DotNET/UnrealBuildTool.exe" \
       -projectfiles -project="InfinityBlade4.uproject" -game -rocket -progress

   # macOS
   /Users/Shared/Epic\ Games/UE_4.24/Engine/Build/BatchFiles/Mac/GenerateProjectFiles.sh \
       -project="$(pwd)/InfinityBlade4.uproject" -game
   ```

4. Build the Editor target:
   ```bash
   # Windows (Visual Studio)
   msbuild InfinityBlade4.sln /t:InfinityBlade4Editor /p:Configuration=Development

   # macOS
   xcodebuild -workspace InfinityBlade4.xcworkspace -scheme InfinityBlade4Editor
   ```

5. Open the project in the Unreal Editor and load the `InfinityBlade4` map from the Content Browser.

6. Import T3D Blueprint files via **File → Import Into Level** or the Content Browser import dialog. Select the `.t3d` files from `infinity-blade-4/Blueprints/`.

### PWA HUD (optional)

```bash
cd infinity-blade-4/pwa-hud
npm install
npm run build   # outputs to pwa-hud/dist/
npm run dev     # hot-reload dev server on localhost:5173
```

Open `http://localhost:5173` in a mobile browser or desktop Chrome with device emulation. The HUD communicates with the game via `window.ib4*` events dispatched from the UE4 WebView integration.

---

## File Reference

| Path | Purpose |
|------|---------|
| `InfinityBlade4.uproject` | UE4.24 project descriptor, plugin list, target platforms |
| `Source/InfinityBlade4/InfinityBlade4.Build.cs` | Module build rules, public/private dependencies |
| `Source/InfinityBlade4/IB4Module.cpp` | IMPLEMENT_MODULE entry point |
| `Source/InfinityBlade4/Core/IB4Types.h` | Shared enums and structs (EAttackDirection, EMagicType, FBloodlineStats) |
| `Source/InfinityBlade4/Core/IB4GameMode.h/.cpp` | Arena lifecycle, titan spawning, score tracking |
| `Source/InfinityBlade4/Characters/IB4Character.h/.cpp` | Abstract base: Health, Magic, TakeDamage, OnDeath |
| `Source/InfinityBlade4/Characters/IB4PlayerCharacter.h/.cpp` | Player: camera, input, XP, equipment hookup |
| `Source/InfinityBlade4/Characters/IB4PlayerController.h/.cpp` | Touch/swipe detection, pause, bloodline restart |
| `Source/InfinityBlade4/Combat/IB4CombatComponent.h/.cpp` | Core combat state machine: attack, parry, dodge, magic |
| `Source/InfinityBlade4/Combat/IB4WeaponBase.h/.cpp` | Weapon actor: hitbox, damage types, swing VFX |
| `Source/InfinityBlade4/Combat/IB4AttackChain.h/.cpp` | Data asset: montage table indexed by direction × combo depth |
| `Source/InfinityBlade4/Combat/IB4ParrySystem.h/.cpp` | Perfect-parry timing windows and clash resolution |
| `Source/InfinityBlade4/Combat/IB4MagicProjectile.h/.cpp` | Elemental projectile with element-specific behavior |
| `Source/InfinityBlade4/Components/IB4BloodlineComponent.h/.cpp` | Perk tree, rebirth, persistent stat bonuses |
| `Source/InfinityBlade4/Components/IB4EquipmentComponent.h/.cpp` | Five-slot equipment manager |
| `Source/InfinityBlade4/Animation/IB4AnimInstance.h/.cpp` | Animation Blueprint driver: state flags, montage helpers |
| `Source/InfinityBlade4/UI/IB4HUD.h/.cpp` | UMG host, debug overlay, transition screens |
| `Blueprints/BP_CombatChain.t3d` | Visual combo chain editor Blueprint |
| `Blueprints/BP_BloodlineLevelUp.t3d` | Rebirth sequence controller Blueprint |
| `Blueprints/BP_EquipmentPickup.t3d` | World pickup with auto-equip Blueprint |
| `Blueprints/BP_MagicCasting.t3d` | Elemental projectile spawner Blueprint |
| `Blueprints/BP_TitanBossFight.t3d` | Phase-based boss encounter Blueprint |
| `pwa-hud/src/ib4-hud.ts` | Main PWA HUD TypeScript — health, mana, combo |
| `pwa-hud/src/ib4-bloodline.ts` | Bloodline progression panel TypeScript |
| `pwa-hud/src/ib4-equipment.ts` | Equipment loadout panel TypeScript |
| `pwa-hud/index.html` | PWA shell with Service Worker registration |
| `tools/generate_blueprints.rs` | Rust tool: generates T3D Blueprint files |

---

## Combat Tuning

Key combat parameters are exposed as `EditDefaultsOnly` properties so designers can tune from the Blueprint class defaults without recompiling:

| Property | Class | Default | Effect |
|----------|-------|---------|--------|
| `ComboResetTime` | UIB4CombatComponent | 1.5s | Seconds before combo chain expires |
| `DodgeInvincibilityDuration` | UIB4CombatComponent | 0.3s | I-frame window during dodge |
| `DodgeLaunchSpeed` | UIB4CombatComponent | 800 cm/s | Dodge impulse strength |
| `MagicCost` | UIB4CombatComponent | 25.0 | Mana consumed per cast |
| `MinSwipeDistance` | AIB4PlayerController | 60 px | Tap vs. swipe threshold |
| `XPBase` | AIB4PlayerCharacter | 100.0 | XP curve base coefficient |
| `XPExponent` | AIB4PlayerCharacter | 1.5 | XP curve exponent |

---

## Credits

Built with:
- **Rocket SDK** — project scaffolding and multi-project build orchestration
- **unify-rs** — Rust workspace providing Blueprint generation (`unify-bp`), process management (`unify-pm`), and MCP tool integration (`unify-mcp`)
- **blueprint-rs** (`tools/generate_blueprints.rs`) — procedural T3D Blueprint file generator
- **Unreal Engine 4.24** — Epic Games SurvivalGame sample as base layer

Spiritual inspiration: ChAIR Entertainment's Infinity Blade I, II, and III (2010–2013), Epic Games' Unreal Engine showcase titles.

---

## License

This project is released as open-source software. See LICENSE in the repository root for details. Unreal Engine 4 source code is subject to the Epic Games EULA.
