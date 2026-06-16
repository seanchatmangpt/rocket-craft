// Copyright 2024 Infinity Blade 4. All Rights Reserved.

#include "AI/IB4GodKingAI.h"
#include "AI/IB4TitanAI.h"
#include "Character/IB4PlayerCharacter.h"
#include "Kismet/GameplayStatics.h"
#include "GameFramework/CharacterMovementComponent.h"
#include "Particles/ParticleSystemComponent.h"
#include "Sound/SoundCue.h"
#include "Engine/World.h"
#include "TimerManager.h"
#include "Math/UnrealMathUtility.h"
#include "NavigationSystem.h"

//-----------------------------------------------------------------------------
// Construction
//-----------------------------------------------------------------------------

AIB4GodKingAI::AIB4GodKingAI()
{
    PrimaryActorTick.bCanEverTick = true;

    // Boss-tier stats
    MaxHealth     = 2000.f;
    CurrentHealth = 2000.f;
    AttackDamage  = 80.f;
    MoveSpeed     = 340.f;
    TitanType     = ETitanType::GodKing;

    // Phase 1 shield
    bShieldActive          = true;
    PerfectParriesReceived = 0;
    ParriesToBreakShield   = 3;

    // Phase 2 QIP Scar
    QIPScarStacks    = 0;
    MaxQIPScarStacks = 3;
    TrackedPlayer    = nullptr;

    // Phase 3 time distortion
    MinTimeDilation        = 0.7f;
    MaxTimeDilation        = 1.3f;
    TimeDilationInterval   = 4.f;
    ReinforcementCount     = 2;
    ReinforcementClass     = AIB4TitanAI::StaticClass();
    ReinforcementSpawnRadius = 500.f;

    // FX
    ShieldBreakParticle    = nullptr;
    ShieldBreakSound       = nullptr;
    DualBladeDrawParticle  = nullptr;
    DualBladeDrawSound     = nullptr;
    RealityDistortionParticle = nullptr;
    ActiveRealityFX        = nullptr;
}

//-----------------------------------------------------------------------------
// Lifecycle
//-----------------------------------------------------------------------------

void AIB4GodKingAI::BeginPlay()
{
    Super::BeginPlay();

    // Cache the player reference early; refreshed in OnPhaseTransition(2)
    if (APawn* Pawn = UGameplayStatics::GetPlayerPawn(GetWorld(), 0))
    {
        TrackedPlayer = Cast<AIB4PlayerCharacter>(Pawn);
    }
}

void AIB4GodKingAI::EndPlay(const EEndPlayReason::Type EndPlayReason)
{
    // Always restore global time dilation when the God King leaves the world
    StopTimeDilation();
    Super::EndPlay(EndPlayReason);
}

//-----------------------------------------------------------------------------
// SelectAttack override
//-----------------------------------------------------------------------------

EAttackDirection AIB4GodKingAI::SelectAttack()
{
    // Phase 1 — all attacks are launched "through" the hard-light shield.
    // The shield absorbs the blow; only a perfect parry bypasses it.
    // We still choose a direction so animation notifies fire correctly.
    if (CurrentPhase == 1)
    {
        // Always attack from the shield face — Right in lore terms, but
        // mechanically the direction matters for the parry check only.
        return EAttackDirection::Right;
    }

    // Phase 2 and 3 — faster, less predictable patterns.
    // 25 % chance of a cross-directional follow-up (Down sweep after any hit).
    const float CrossChance = (CurrentPhase == 3) ? 0.35f : 0.25f;
    if (FMath::FRand() < CrossChance)
    {
        return EAttackDirection::Down;
    }

    // Otherwise distribute evenly across Left / Right / Up
    const int32 Roll = FMath::RandRange(0, 2);
    switch (Roll)
    {
    case 0:  return EAttackDirection::Left;
    case 1:  return EAttackDirection::Right;
    default: return EAttackDirection::Up;
    }
}

//-----------------------------------------------------------------------------
// OnPhaseTransition override
//-----------------------------------------------------------------------------

void AIB4GodKingAI::OnPhaseTransition(int32 NewPhase)
{
    Super::OnPhaseTransition(NewPhase);

    if (NewPhase == 2)
    {
        // --- Dual Infinity Blades drawn ---
        if (DualBladeDrawParticle)
        {
            UGameplayStatics::SpawnEmitterAtLocation(GetWorld(), DualBladeDrawParticle,
                                                     GetActorLocation());
        }
        if (DualBladeDrawSound)
        {
            UGameplayStatics::PlaySoundAtLocation(GetWorld(), DualBladeDrawSound,
                                                  GetActorLocation());
        }

        // Refresh the tracked player pointer
        if (APawn* Pawn = UGameplayStatics::GetPlayerPawn(GetWorld(), 0))
        {
            TrackedPlayer = Cast<AIB4PlayerCharacter>(Pawn);
        }

        // Attack frequency doubles — in a full implementation this would be a
        // BT Decorator float param; we halve the internal attack cooldown here
        // so the AI task layer picks it up via the blackboard.
        // The BT "CanAttack" key is refreshed in IB4AIController::UpdateBlackboardValues.

        UE_LOG(LogTemp, Log, TEXT("[IB4GodKingAI] %s Phase 2 — Dual Infinity Blades drawn."),
               *GetName());
    }
    else if (NewPhase == 3)
    {
        // --- Reality Distortion begins ---
        if (RealityDistortionParticle)
        {
            ActiveRealityFX = UGameplayStatics::SpawnEmitterAttached(
                RealityDistortionParticle,
                GetMesh(),
                NAME_None);
        }

        // Start random time-dilation fluctuations
        GetWorldTimerManager().SetTimer(TimerHandle_TimeDilation,
                                        this,
                                        &AIB4GodKingAI::FluctuateTimeDilation,
                                        TimeDilationInterval,
                                        true,   // looping
                                        0.5f);  // first tick after 0.5 s

        // Spawn reinforcement Titans
        SpawnReinforcements();

        UE_LOG(LogTemp, Log, TEXT("[IB4GodKingAI] %s Phase 3 — Reality distortion and "
               "%d reinforcements spawned."), *GetName(), ReinforcementCount);
    }
}

//-----------------------------------------------------------------------------
// Hard-light shield
//-----------------------------------------------------------------------------

void AIB4GodKingAI::RegisterPerfectParry()
{
    if (!bShieldActive)
    {
        // Shield is already broken; treat as a normal parry hit against health
        const float ParryDamage = AttackDamage * 0.5f;
        TakeMeleeDamage(ParryDamage, EAttackDirection::Right);
        return;
    }

    ++PerfectParriesReceived;

    UE_LOG(LogTemp, Log, TEXT("[IB4GodKingAI] Perfect parry registered (%d / %d)."),
           PerfectParriesReceived, ParriesToBreakShield);

    if (PerfectParriesReceived >= ParriesToBreakShield)
    {
        BreakHardLightShield();
    }
}

void AIB4GodKingAI::BreakHardLightShield()
{
    if (!bShieldActive)
    {
        return;
    }

    bShieldActive = false;

    // --- Destruction VFX / SFX ---
    if (ShieldBreakParticle)
    {
        UGameplayStatics::SpawnEmitterAtLocation(GetWorld(), ShieldBreakParticle,
                                                 GetActorLocation(),
                                                 FRotator::ZeroRotator,
                                                 FVector(2.f));
    }
    if (ShieldBreakSound)
    {
        UGameplayStatics::PlaySoundAtLocation(GetWorld(), ShieldBreakSound,
                                              GetActorLocation());
    }

    UE_LOG(LogTemp, Log, TEXT("[IB4GodKingAI] Hard-light shield shattered!"));

    // If health is already below 60 %, we skip Phase 2 directly to Phase 3
    // (edge case: player perfectly parried 3 times from 100 % to 59 %).
    const float HealthPct = GetCurrentHealthPct();
    if (HealthPct <= PhaseThreeHealthThreshold && CurrentPhase < 3)
    {
        CurrentPhase = 3;
        OnPhaseTransition(3);
    }
    else if (HealthPct <= PhaseTwoHealthThreshold && CurrentPhase < 2)
    {
        CurrentPhase = 2;
        OnPhaseTransition(2);
    }
    // Otherwise normal combat continues from Phase 1 (shield broken but health > 60 %)
}

//-----------------------------------------------------------------------------
// QIP Scar (Phase 2)
//-----------------------------------------------------------------------------

void AIB4GodKingAI::ApplyQIPScar(AIB4PlayerCharacter* Player)
{
    if (!Player)
    {
        return;
    }

    ++QIPScarStacks;

    UE_LOG(LogTemp, Log, TEXT("[IB4GodKingAI] QIP Scar applied — stacks: %d / %d"),
           QIPScarStacks, MaxQIPScarStacks);

    if (QIPScarStacks >= MaxQIPScarStacks)
    {
        QIPScarStacks = 0;  // Reset so the counter can build again after rebirth

        // Trigger the IB4 rebirth mechanic — preserves bloodline XP,
        // resets health, plays the rebirth cinematic, and respawns the player.
        Player->TriggerRebirth();

        UE_LOG(LogTemp, Log, TEXT("[IB4GodKingAI] QIP Scar at 3 stacks — "
               "player rebirth triggered!"));
    }
}

//-----------------------------------------------------------------------------
// Phase 3 — time dilation
//-----------------------------------------------------------------------------

void AIB4GodKingAI::FluctuateTimeDilation()
{
    if (CurrentPhase < 3)
    {
        StopTimeDilation();
        return;
    }

    const float NewDilation = FMath::RandRange(MinTimeDilation, MaxTimeDilation);
    UGameplayStatics::SetGlobalTimeDilation(GetWorld(), NewDilation);

    UE_LOG(LogTemp, Log, TEXT("[IB4GodKingAI] Reality distortion — time dilation: %.2f"),
           NewDilation);
}

void AIB4GodKingAI::StopTimeDilation()
{
    GetWorldTimerManager().ClearTimer(TimerHandle_TimeDilation);
    if (UWorld* World = GetWorld())
    {
        UGameplayStatics::SetGlobalTimeDilation(World, 1.f);
    }
}

void AIB4GodKingAI::SpawnReinforcements()
{
    if (!ReinforcementClass)
    {
        return;
    }

    UWorld* World = GetWorld();
    if (!World)
    {
        return;
    }

    for (int32 i = 0; i < ReinforcementCount; ++i)
    {
        // Find a random navigable point near the God King
        const FVector Origin = GetActorLocation();
        FNavLocation SpawnNavLoc;
        FVector SpawnLocation = Origin;

        UNavigationSystemV1* NavSys = UNavigationSystemV1::GetCurrent(World);
        if (NavSys)
        {
            // Pick a point on the nav mesh within the spawn radius
            NavSys->GetRandomReachablePointInRadius(Origin,
                                                    ReinforcementSpawnRadius,
                                                    SpawnNavLoc);
            SpawnLocation = SpawnNavLoc.Location;
        }
        else
        {
            // Fallback: spawn at an offset from the God King's position
            const float Angle = (360.f / ReinforcementCount) * i;
            const FVector Offset = FVector(FMath::Cos(FMath::DegreesToRadians(Angle)),
                                           FMath::Sin(FMath::DegreesToRadians(Angle)),
                                           0.f) * ReinforcementSpawnRadius;
            SpawnLocation = Origin + Offset;
        }

        FActorSpawnParameters SpawnParams;
        SpawnParams.SpawnCollisionHandlingOverride =
            ESpawnActorCollisionHandlingMethod::AdjustIfPossibleButAlwaysSpawn;

        AIB4TitanAI* Reinforcement = World->SpawnActor<AIB4TitanAI>(
            ReinforcementClass,
            SpawnLocation,
            FRotator::ZeroRotator,
            SpawnParams);

        if (Reinforcement)
        {
            UE_LOG(LogTemp, Log, TEXT("[IB4GodKingAI] Spawned reinforcement Titan at %s"),
                   *SpawnLocation.ToString());
        }
    }
}

//-----------------------------------------------------------------------------
// OnDefeated override
//-----------------------------------------------------------------------------

void AIB4GodKingAI::OnDefeated()
{
    // Stop time distortion before anything else
    StopTimeDilation();

    // Destroy active reality FX component
    if (ActiveRealityFX)
    {
        ActiveRealityFX->DestroyComponent();
        ActiveRealityFX = nullptr;
    }

    // Final shield-break effect repurposed as death burst
    if (ShieldBreakParticle)
    {
        UGameplayStatics::SpawnEmitterAtLocation(GetWorld(), ShieldBreakParticle,
                                                 GetActorLocation(),
                                                 FRotator::ZeroRotator,
                                                 FVector(3.f));
    }

    UE_LOG(LogTemp, Log, TEXT("[IB4GodKingAI] Corrupted Galath (God King) has been "
           "permanently defeated. The QIP is finally at rest."));

    Super::OnDefeated();
}
