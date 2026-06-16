// Copyright 2024 Infinity Blade 4. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "AI/IB4EnemyBase.h"
#include "IB4GodKingAI.generated.h"

class USoundCue;
class UParticleSystem;
class UParticleSystemComponent;
class AIB4TitanAI;

// Forward-declare the player character; full include is in the .cpp to avoid
// circular dependency (Player includes Combat which includes nothing from AI).
class AIB4PlayerCharacter;

/**
 * AIB4GodKingAI
 *
 * Final boss — "Corrupted Galath" awakened in his adult QIP form.
 *
 * Phase 1  (100%–60%) — Hard-light shield active. Only perfect parries damage
 *                        him. Requires 3 perfect parries to break the shield.
 * Phase 2   (60%–30%) — Dual Infinity Blades. Attack frequency doubles.
 *                        Each hit applies "QIP Scar" debuff (3 stacks = instant
 *                        rebirth for the player).
 * Phase 3   (30%–0%)  — Reality distortion. Random FTimeDilation fluctuations
 *                        (0.7×–1.3×). Spawns 2 IB4TitanAI reinforcements.
 */
UCLASS(Blueprintable)
class INFINITYBLADE4_API AIB4GodKingAI : public AIB4EnemyBase
{
    GENERATED_BODY()

public:
    AIB4GodKingAI();

protected:
    virtual void BeginPlay() override;
    virtual void EndPlay(const EEndPlayReason::Type EndPlayReason) override;

    //-----------------------------------------------------------------------
    // Phase 1 — Hard-light shield
    //-----------------------------------------------------------------------

    /** True while the hard-light shield is protecting the God King */
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "AI|GodKing|Shield")
    bool bShieldActive;

    /** Running count of perfect parries received this phase */
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "AI|GodKing|Shield")
    int32 PerfectParriesReceived;

    /** Number of perfect parries required to shatter the shield */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "AI|GodKing|Shield")
    int32 ParriesToBreakShield;

    //-----------------------------------------------------------------------
    // Phase 2 — QIP Scar
    //-----------------------------------------------------------------------

    /** Current stack count of QIP Scar applied to the player */
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "AI|GodKing|QIP")
    int32 QIPScarStacks;

    /** Maximum stacks before rebirth is triggered (always 3) */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "AI|GodKing|QIP")
    int32 MaxQIPScarStacks;

    /** Raw pointer to the tracked player — refreshed in OnPhaseTransition */
    UPROPERTY(Transient)
    AIB4PlayerCharacter* TrackedPlayer;

    //-----------------------------------------------------------------------
    // Phase 3 — Reality distortion
    //-----------------------------------------------------------------------

    /** Handle for the recurring time-dilation fluctuation timer */
    FTimerHandle TimerHandle_TimeDilation;

    /** Minimum time-dilation factor applied during Phase 3 */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "AI|GodKing|Reality")
    float MinTimeDilation;

    /** Maximum time-dilation factor applied during Phase 3 */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "AI|GodKing|Reality")
    float MaxTimeDilation;

    /** Seconds between each time-dilation shift */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "AI|GodKing|Reality")
    float TimeDilationInterval;

    /** How many reinforcements to spawn when Phase 3 begins */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "AI|GodKing|Reality")
    int32 ReinforcementCount;

    /** TitanAI subclass to spawn as reinforcements */
    UPROPERTY(EditDefaultsOnly, Category = "AI|GodKing|Reality")
    TSubclassOf<AIB4TitanAI> ReinforcementClass;

    /** Radius around the God King within which reinforcements are spawned */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "AI|GodKing|Reality")
    float ReinforcementSpawnRadius;

    //-----------------------------------------------------------------------
    // FX assets
    //-----------------------------------------------------------------------

    UPROPERTY(EditDefaultsOnly, Category = "AI|GodKing|FX")
    UParticleSystem* ShieldBreakParticle;

    UPROPERTY(EditDefaultsOnly, Category = "AI|GodKing|FX")
    USoundCue* ShieldBreakSound;

    UPROPERTY(EditDefaultsOnly, Category = "AI|GodKing|FX")
    UParticleSystem* DualBladeDrawParticle;

    UPROPERTY(EditDefaultsOnly, Category = "AI|GodKing|FX")
    USoundCue* DualBladeDrawSound;

    UPROPERTY(EditDefaultsOnly, Category = "AI|GodKing|FX")
    UParticleSystem* RealityDistortionParticle;

    UPROPERTY(VisibleAnywhere, Category = "AI|GodKing|FX")
    UParticleSystemComponent* ActiveRealityFX;

    //-----------------------------------------------------------------------
    // Internal helpers
    //-----------------------------------------------------------------------

    /** Tick called by TimerHandle_TimeDilation to randomise global time dilation */
    void FluctuateTimeDilation();

    /** Spawns reinforcement Titans around the God King */
    void SpawnReinforcements();

    /** Resets global time dilation to 1.0 and kills the fluctuation timer */
    void StopTimeDilation();

public:
    //-----------------------------------------------------------------------
    // AIB4EnemyBase overrides
    //-----------------------------------------------------------------------

    /**
     * Phase 1 — all attacks pass through the shield; only perfect-parry
     *           damage (routed through RegisterPerfectParry) counts.
     * Phase 2/3 — faster patterns; 25% chance of a cross-directional follow-up.
     */
    virtual EAttackDirection SelectAttack() override;

    /**
     * Phase 2: plays dual-blade draw anim/sound, doubles internal attack timer.
     * Phase 3: starts time-dilation fluctuations and spawns reinforcements.
     */
    virtual void OnPhaseTransition(int32 NewPhase) override;

    /** Stops all timers, resets time dilation, plays death FX */
    virtual void OnDefeated() override;

    //-----------------------------------------------------------------------
    // God King specific interface
    //-----------------------------------------------------------------------

    /**
     * Called by the combat system when the player lands a perfect parry
     * against the God King. Increments PerfectParriesReceived and shatters
     * the shield at the threshold.
     */
    UFUNCTION(BlueprintCallable, Category = "AI|GodKing")
    void RegisterPerfectParry();

    /**
     * Destroys the hard-light shield, plays destruction VFX/SFX, and
     * triggers Phase 2 if current HP is already below 60%.
     */
    UFUNCTION(BlueprintCallable, Category = "AI|GodKing")
    void BreakHardLightShield();

    /**
     * Increments the QIP Scar debuff stack on the given player.
     * At MaxQIPScarStacks (3) immediately triggers the player's rebirth.
     */
    UFUNCTION(BlueprintCallable, Category = "AI|GodKing")
    void ApplyQIPScar(AIB4PlayerCharacter* Player);

    FORCEINLINE bool IsShieldActive()          const { return bShieldActive; }
    FORCEINLINE int32 GetPerfectParriesReceived() const { return PerfectParriesReceived; }
    FORCEINLINE int32 GetQIPScarStacks()       const { return QIPScarStacks; }
};
