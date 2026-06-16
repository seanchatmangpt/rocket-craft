# Infinity Blade IV — Technical Architecture

**Engine:** Unreal Engine 4.24 | **Base:** SurvivalGame template refactored

---

## Module Structure

```
InfinityBlade4 (runtime module)
├── Core/          — IB4Types.h (all enums/structs), IB4GameMode
├── Characters/    — IB4Character, IB4PlayerCharacter, IB4PlayerController
├── Combat/        — CombatComponent, WeaponBase, AttackChain, ParrySystem, MagicProjectile
├── AI/            — IB4EnemyBase, IB4TitanAI, IB4GodKingAI, IB4AIController
├── Equipment/     — EquipmentBase, Sword, Shield, Ring, Helmet
├── Progression/   — XPSystem, NewGamePlus, BloodlinePerkTree
├── Animation/     — IB4AnimInstance
└── UI/            — IB4HUD
```

---

## Data Flow: Touch Input → Combat → Animation

```
TouchBegin (FVector2D Start)
  └─ AIB4PlayerController::OnInputTouchBegin
       └─ store TouchStartPosition

TouchEnd (FVector2D End)
  └─ AIB4PlayerController::OnInputTouchEnd
       └─ DetectSwipeDirection(Start, End)
            ├─ |deltaX| > 50 AND |deltaX| > |deltaY| → Right or Left
            ├─ |deltaY| > 50 AND |deltaY| > |deltaX| → Overhead
            └─ dist < 50                              → Dodge

            → AIB4PlayerCharacter::OnAttackInput(EAttackDirection)
                 └─ UIB4CombatComponent::BeginAttack(Direction)
                      ├─ Play montage slot [Direction][ComboDepth]
                      ├─ UIB4WeaponBase::EnableHitCollision()
                      └─ FTimerHandle → ResetComboIfIdle(2.0s)

AnimNotifyState_HitWindow::NotifyTick
  └─ UCapsuleComponent overlap → UIB4WeaponBase::OnOverlapBegin
       ├─ Dedup check (AlreadyHitActors)
       ├─ Roll crit (RarityScaledCritChance)
       ├─ BaseDamage × ComboMultiplier × DirectionMultiplier
       └─ AIB4Character::TakeDamage

UIB4AnimInstance::NativeUpdateAnimation (per-tick)
  └─ reads: Velocity, ECombatState, TouchDelta, Health01, Magic01
       → drives: bIsMoving, Speed, CombatState, SwipeDirection, HealthPct, MagicPct
```

---

## Bloodline Rebirth Flow

```
AIB4Character::OnDeath (BlueprintNativeEvent)
  └─ UIB4NewGamePlus::TriggerRebirth(bPlayerDied=true)
       ├─ CurrentBloodline++
       ├─ Reset Gold/Equipment via UObject::ProcessEvent reflection
       ├─ Grant 1 PerkPoint
       ├─ MasteryXPMultiplier *= 2.0
       ├─ Save via UFUNCTION reflection on GameMode
       └─ FOnRebirth.Broadcast(BloodlineData)
            └─ AIB4GodKingAI::OnRebirth → scale HP/Damage by (50 × Bloodline)
```

---

## AI Architecture

Each enemy uses the Unreal BehaviorTree system via `AIB4AIController`:

```
BT_IB4Enemy (BehaviorTree)
├─ Selector
│   ├─ Sequence [BB_CanAttack == true]
│   │   └─ BTTask_ExecuteAttack → AIB4EnemyBase::SelectAttack()
│   │        ├─ IB4TitanAI: phase-dependent attack selection
│   │        └─ IB4GodKingAI: shield-check → parry-bait → QIP scar
│   └─ Sequence [BB_CanAttack == false]
│       ├─ BTTask_MoveToTarget (NavMesh path)
│       └─ BTTask_UpdateDistance → set BB_TargetDistance
└─ Service_PerceptionUpdate (0.1s interval)
     └─ UAISenseConfig_Sight (1500u radius) + UAISenseConfig_Hearing (600u)
```

---

## Parry Evaluation

```
UIB4ParrySystem::AttemptParry(AttackTimestamp)
  └─ Delta = Now - AttackTimestamp
       ├─ Delta < 0.05s  → PerfectParry → SetGlobalTimeDilation(0.2, 1.5s)
       ├─ Delta < 0.20s  → NormalParry  → stagger enemy
       └─ Delta >= 0.20s → Miss         → full damage
```

---

## Equipment Gem System

```
UIB4EquipmentBase
  └─ TArray<FGemSocket> GemSlots  (count = f(Rarity))
       └─ FGemSocket { EGemType Type; int32 BonusValue }
            ├─ Fire   → AttackBonus += BonusValue
            ├─ Ice    → DefenseBonus += BonusValue
            ├─ Light  → MagicBonus += BonusValue
            └─ Dark   → XPGainMultiplier += BonusValue * 0.01
```

Gem slot counts: Common=0, Uncommon=1, Rare=1, Epic=2, Legendary=2, Infinity=3.

---

## PWA HUD Bridge

The TypeScript PWA HUD (`pwa-hud/src/ib4-hud.ts`) mirrors the C++ game state via `CustomEvent` dispatches:

| C++ Event | CustomEvent Name | Payload |
|---|---|---|
| `FOnLevelUp` | `ib4:levelup` | `{ level, statPoints }` |
| `FOnRebirth` | `ib4:rebirth` | `{ bloodline }` |
| `OnPerfectParry` | `ib4:perfectparry` | — |
| `UpdateComboDisplay` | `ib4:combo` | `{ depth, multiplier }` |
| `HealthChanged` | `ib4:health` | `{ current, max }` |

In production this bridge is replaced by the native UMG widget (`W_IB4HUD`) which `AIB4HUD` adds to viewport in `BeginPlay`.

---

## Serialization

`UIB4XPSystem` hand-rolls a JSON serialiser (no external dependency) for `USaveGame` compatibility:

```
FString Serialize() const
  → "{\"Level\":%d,\"XP\":%lld,\"Stats\":{...},\"PerkPoints\":%d}"

static UIB4XPSystem* Deserialize(const FString& Json)
  → line-scan parser (no heap alloc beyond FString)
```

`UIB4BloodlinePerkTree` stores selected perks as `TSet<FName>` serialized to comma-delimited `FString`.

---

## Build Targets

| Target | Config | Platform |
|---|---|---|
| InfinityBlade4 | Development / Shipping | iOS, Android, Win64 |
| InfinityBlade4Editor | Development | Win64 |

`InfinityBlade4.Build.cs` public deps: Core, CoreUObject, Engine, InputCore, AIModule, NavigationSystem, UMG, HeadMountedDisplay, GameplayTasks.

---

## Key File Index

| Path | Purpose |
|---|---|
| `Source/.../Core/IB4Types.h` | Canonical enum/struct definitions |
| `Source/.../Combat/IB4CombatComponent.h` | Central combat state machine |
| `Source/.../AI/IB4GodKingAI.h` | Final boss — Corrupted Galath |
| `Source/.../Progression/IB4NewGamePlus.h` | Bloodline rebirth and perk selection |
| `Data/enemies.csv` | All 15 enemy types with scaling |
| `Blueprints/BP_TitanBossFight.t3d` | Phase gate logic |
| `pwa-hud/src/ib4-hud.ts` | PWA overlay HUD |
| `GDD.md` | Full game design document |
