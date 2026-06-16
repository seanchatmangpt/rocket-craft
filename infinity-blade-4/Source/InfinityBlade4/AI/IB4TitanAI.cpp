// Copyright 2024 Infinity Blade 4. All Rights Reserved.

#include "AI/IB4TitanAI.h"
#include "Kismet/GameplayStatics.h"
#include "GameFramework/CharacterMovementComponent.h"
#include "Particles/ParticleSystemComponent.h"
#include "Sound/SoundCue.h"
#include "Engine/World.h"
#include "TimerManager.h"
#include "Math/UnrealMathUtility.h"

//-----------------------------------------------------------------------------
// Construction
//-----------------------------------------------------------------------------

AIB4TitanAI::AIB4TitanAI()
{
    PrimaryActorTick.bCanEverTick = true;

    // Override base stats for a Titan
    MaxHealth     = 800.f;
    CurrentHealth = 800.f;
    AttackDamage  = 60.f;
    MoveSpeed     = 380.f;
    TitanType     = ETitanType::Warrior;

    // Phase flags
    bCanBreakParry    = false;
    bIsEnraged        = false;
    ParryCounterChance = 0.4f;   // 40 % chance to counter a parry in Phase 1
    WeaponThrowCooldown = 8.f;
    bWeaponThrowReady = false;

    // FX assets — assigned in Blueprint defaults
    PhaseRoarSound          = nullptr;
    PhaseTransitionParticle = nullptr;
    EnragedAuraParticle     = nullptr;
}

//-----------------------------------------------------------------------------
// Lifecycle
//-----------------------------------------------------------------------------

void AIB4TitanAI::BeginPlay()
{
    Super::BeginPlay();
    // Movement speed was applied by base class; nothing extra needed here.
}

//-----------------------------------------------------------------------------
// SelectAttack override
//-----------------------------------------------------------------------------

EAttackDirection AIB4TitanAI::SelectAttack()
{
    switch (CurrentPhase)
    {
    case 1:
    {
        // Phase 1: telegraphed — telegraph indicates Left, Right, or Up only.
        // Down attacks have no telegraph (kept for Phase 3), so exclude them here.
        const int32 Roll = FMath::RandRange(0, 2);
        switch (Roll)
        {
        case 0:  return EAttackDirection::Left;
        case 1:  return EAttackDirection::Right;
        default: return EAttackDirection::Up;
        }
    }
    case 2:
    {
        // Phase 2: mix — 50% chance of a fast double-strike (Down added),
        // 50% chance of the normal telegraphed three.
        if (FMath::FRand() < 0.5f)
        {
            // Standard three, chosen at random
            const int32 Roll = FMath::RandRange(0, 2);
            switch (Roll)
            {
            case 0:  return EAttackDirection::Left;
            case 1:  return EAttackDirection::Right;
            default: return EAttackDirection::Up;
            }
        }
        else
        {
            // Fast double-strike pattern — return Down (sweep) this tick
            return EAttackDirection::Down;
        }
    }
    case 3:
    default:
    {
        // Phase 3: completely random across all four directions with no telegraph.
        const int32 Roll = FMath::RandRange(0, 3);
        switch (Roll)
        {
        case 0:  return EAttackDirection::Left;
        case 1:  return EAttackDirection::Right;
        case 2:  return EAttackDirection::Up;
        default: return EAttackDirection::Down;
        }
    }
    }
}

//-----------------------------------------------------------------------------
// OnPhaseTransition override
//-----------------------------------------------------------------------------

void AIB4TitanAI::OnPhaseTransition(int32 NewPhase)
{
    Super::OnPhaseTransition(NewPhase);

    // --- Shared FX: roar sound + particle burst ---
    if (PhaseRoarSound)
    {
        UGameplayStatics::PlaySoundAtLocation(GetWorld(), PhaseRoarSound,
                                              GetActorLocation());
    }
    if (PhaseTransitionParticle)
    {
        UGameplayStatics::SpawnEmitterAtLocation(GetWorld(), PhaseTransitionParticle,
                                                 GetActorLocation(),
                                                 FRotator::ZeroRotator,
                                                 FVector(1.5f));
    }

    // --- Phase-specific scaling ---
    if (NewPhase == 2)
    {
        // Enraged stance: faster and harder-hitting, can now break player parries
        MoveSpeed    *= 1.4f;
        AttackDamage *= 1.25f;
        bIsEnraged    = true;
        bCanBreakParry = true;

        if (UCharacterMovementComponent* MoveComp = GetCharacterMovement())
        {
            MoveComp->MaxWalkSpeed = MoveSpeed;
        }

        // Spawn an enraged aura that stays attached to the Titan
        if (EnragedAuraParticle)
        {
            UGameplayStatics::SpawnEmitterAttached(EnragedAuraParticle,
                                                   GetMesh(),
                                                   NAME_None);
        }

        UE_LOG(LogTemp, Log, TEXT("[IB4TitanAI] %s entered Enraged Phase 2 — "
               "Speed=%.1f, Damage=%.1f, CanBreakParry=%d"),
               *GetName(), MoveSpeed, AttackDamage, (int32)bCanBreakParry);
    }
    else if (NewPhase == 3)
    {
        // Berserker stance: maximum damage, random directions, weapon throw
        // Note: cumulative — damage was already scaled at Phase 2 entry
        AttackDamage *= 1.5f;

        // Arm the weapon-throw ability with a short initial delay
        bWeaponThrowReady = false;
        GetWorldTimerManager().SetTimer(TimerHandle_WeaponThrow,
                                        this,
                                        &AIB4TitanAI::ResetWeaponThrowCooldown,
                                        WeaponThrowCooldown,
                                        false);

        UE_LOG(LogTemp, Log, TEXT("[IB4TitanAI] %s entered Berserker Phase 3 — "
               "Damage=%.1f, WeaponThrow armed"), *GetName(), AttackDamage);
    }
}

//-----------------------------------------------------------------------------
// Weapon throw helpers
//-----------------------------------------------------------------------------

void AIB4TitanAI::ResetWeaponThrowCooldown()
{
    bWeaponThrowReady = true;
    // Once ready, fire immediately; then the BT / tick logic can re-arm it.
    ExecuteWeaponThrow();
}

void AIB4TitanAI::ExecuteWeaponThrow()
{
    if (!bWeaponThrowReady || CurrentPhase < 3)
    {
        return;
    }

    bWeaponThrowReady = false;

    // Locate the player by querying GameplayStatics for the pawn.
    // A full implementation would spawn a dedicated projectile actor;
    // here we apply damage directly using the UE4 damage pipeline.
    APawn* PlayerPawn = UGameplayStatics::GetPlayerPawn(GetWorld(), 0);
    if (PlayerPawn && !PlayerPawn->IsPendingKillPending())
    {
        const float ThrowDamage = AttackDamage * 0.8f;   // slightly weaker than melee
        UGameplayStatics::ApplyDamage(PlayerPawn, ThrowDamage,
                                      GetController(), this, nullptr);

        UE_LOG(LogTemp, Log, TEXT("[IB4TitanAI] %s threw weapon — %.1f damage"),
               *GetName(), ThrowDamage);
    }

    // Re-arm after cooldown
    GetWorldTimerManager().SetTimer(TimerHandle_WeaponThrow,
                                    this,
                                    &AIB4TitanAI::ResetWeaponThrowCooldown,
                                    WeaponThrowCooldown,
                                    false);
}

//-----------------------------------------------------------------------------
// OnDefeated override
//-----------------------------------------------------------------------------

void AIB4TitanAI::OnDefeated()
{
    // Clear any pending timers before destruction
    GetWorldTimerManager().ClearTimer(TimerHandle_WeaponThrow);

    // Spawn a death particle burst
    if (PhaseTransitionParticle)
    {
        UGameplayStatics::SpawnEmitterAtLocation(GetWorld(), PhaseTransitionParticle,
                                                 GetActorLocation(),
                                                 FRotator::ZeroRotator,
                                                 FVector(2.f));
    }

    // Play one last roar
    if (PhaseRoarSound)
    {
        UGameplayStatics::PlaySoundAtLocation(GetWorld(), PhaseRoarSound,
                                              GetActorLocation());
    }

    UE_LOG(LogTemp, Log, TEXT("[IB4TitanAI] %s has been defeated."), *GetName());

    // Delegate remaining logic (collision off, lifespan, etc.) to the base class
    Super::OnDefeated();
}
