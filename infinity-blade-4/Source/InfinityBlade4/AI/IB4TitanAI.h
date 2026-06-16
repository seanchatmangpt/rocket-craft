// Copyright 2024 Infinity Blade 4. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "AI/IB4EnemyBase.h"
#include "IB4TitanAI.generated.h"

class USoundCue;
class UParticleSystem;

/**
 * AIB4TitanAI
 *
 * Powerful humanoid melee boss with three distinct combat phases:
 *   Phase 1 (100%–60%) — telegraphed attacks, parry counter 40% chance
 *   Phase 2  (60%–30%) — Enraged: faster, harder-hitting, can break parry
 *   Phase 3   (30%–0%) — Berserker: maximum aggression, ranged weapon throw
 */
UCLASS(Blueprintable)
class INFINITYBLADE4_API AIB4TitanAI : public AIB4EnemyBase
{
    GENERATED_BODY()

public:
    AIB4TitanAI();

protected:
    virtual void BeginPlay() override;

    //-----------------------------------------------------------------------
    // Phase flags
    //-----------------------------------------------------------------------

    /** When true the Titan can shatter a player's perfect parry */
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "AI|Titan")
    bool bCanBreakParry;

    /** Set when the Titan enters Phase 2 "Enraged" stance */
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "AI|Titan")
    bool bIsEnraged;

    /** How long (seconds) the parry-counter window stays open */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "AI|Titan")
    float ParryCounterChance;

    /** Timer used to poll for a weapon-throw cooldown in Phase 3 */
    FTimerHandle TimerHandle_WeaponThrow;

    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "AI|Titan")
    float WeaponThrowCooldown;

    bool bWeaponThrowReady;

    //-----------------------------------------------------------------------
    // FX assets (set in Blueprint defaults)
    //-----------------------------------------------------------------------

    UPROPERTY(EditDefaultsOnly, Category = "AI|Titan|FX")
    USoundCue* PhaseRoarSound;

    UPROPERTY(EditDefaultsOnly, Category = "AI|Titan|FX")
    UParticleSystem* PhaseTransitionParticle;

    UPROPERTY(EditDefaultsOnly, Category = "AI|Titan|FX")
    UParticleSystem* EnragedAuraParticle;

    //-----------------------------------------------------------------------
    // Internal helpers
    //-----------------------------------------------------------------------

    /** Resets the weapon-throw cooldown timer so another throw becomes available */
    void ResetWeaponThrowCooldown();

    /** Executes a ranged weapon throw toward the current target (Phase 3 only) */
    void ExecuteWeaponThrow();

public:
    //-----------------------------------------------------------------------
    // AIB4EnemyBase overrides
    //-----------------------------------------------------------------------

    /**
     * Phase 1: random selection from Left/Right/Up (telegraphed, excludes Down).
     * Phase 2: mix — 50% chance of a faster double-strike pattern.
     * Phase 3: fully random across all four directions with no telegraph delay.
     */
    virtual EAttackDirection SelectAttack() override;

    /**
     * Applies stat scaling, plays roar + particle burst, and sets phase flags.
     * Phase 2: MovementSpeed *= 1.4, AttackDamage *= 1.25, bIsEnraged = true.
     * Phase 3: AttackDamage *= 1.5 (cumulative), arms weapon throw.
     */
    virtual void OnPhaseTransition(int32 NewPhase) override;

    /** Plays death animation and grants XP/loot before destroying the actor */
    virtual void OnDefeated() override;
};
