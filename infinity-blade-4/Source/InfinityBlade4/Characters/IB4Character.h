// Copyright 2024 Infinity Blade 4 Project. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "GameFramework/Character.h"
#include "Core/IB4Types.h"
#include "IB4Character.generated.h"

/**
 * AIB4Character is the abstract base character for all Infinity Blade 4 combatants,
 * both player-controlled and AI-driven titans.
 *
 * Responsibilities:
 *  - Health and Magic resource management with full replication
 *  - TakeDamage override applying bloodline defense reduction
 *  - Heal / RestoreMagic utilities callable from Blueprint
 *  - BlueprintNativeEvent death callback (OnDeath)
 *  - Bloodline level and XP tracking
 */
UCLASS(Abstract, BlueprintType, Blueprintable)
class INFINITYBLADE4_API AIB4Character : public ACharacter
{
    GENERATED_BODY()

public:

    AIB4Character(const FObjectInitializer& ObjectInitializer);

    virtual void BeginPlay() override;

    virtual void GetLifetimeReplicatedProps(TArray<FLifetimeProperty>& OutLifetimeProps) const override;

    // ---------------------------------------------------------------------------
    // Health
    // ---------------------------------------------------------------------------

    /** Current hit points. Replicated to all clients. */
    UPROPERTY(BlueprintReadOnly, Category = "Character|Health", ReplicatedUsing = OnRep_Health)
    float Health;

    /** Maximum hit points. Replicated so clients can display the bar correctly. */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Character|Health", Replicated)
    float MaxHealth;

    /** Returns current health as a 0..1 fraction. */
    UFUNCTION(BlueprintCallable, Category = "Character|Health")
    float GetHealthPercent() const;

    /** True as long as Health > 0. */
    UFUNCTION(BlueprintCallable, Category = "Character|Health")
    bool IsAlive() const;

    /** Restore HealthAmount hit points, clamped to MaxHealth. */
    UFUNCTION(BlueprintCallable, Category = "Character|Health")
    void Heal(float HealthAmount);

    // ---------------------------------------------------------------------------
    // Magic
    // ---------------------------------------------------------------------------

    /** Current magic resource. Replicated to all clients. */
    UPROPERTY(BlueprintReadOnly, Category = "Character|Magic", ReplicatedUsing = OnRep_Magic)
    float Magic;

    /** Maximum magic resource. */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Character|Magic", Replicated)
    float MaxMagic;

    /** Returns current magic as a 0..1 fraction. */
    UFUNCTION(BlueprintCallable, Category = "Character|Magic")
    float GetMagicPercent() const;

    /** Restore MagicAmount magic points, clamped to MaxMagic. */
    UFUNCTION(BlueprintCallable, Category = "Character|Magic")
    void RestoreMagic(float MagicAmount);

    // ---------------------------------------------------------------------------
    // Bloodline / XP
    // ---------------------------------------------------------------------------

    /** Current bloodline rebirth level. Persisted across death/rebirth cycles. */
    UPROPERTY(BlueprintReadOnly, Category = "Character|Bloodline", Replicated)
    int32 BloodlineLevel;

    /** Accumulated XP within the current bloodline cycle. */
    UPROPERTY(BlueprintReadOnly, Category = "Character|Bloodline", Replicated)
    float CurrentXP;

    /** Cached bloodline bonus stats, updated when BloodlineLevel changes. */
    UPROPERTY(BlueprintReadOnly, Category = "Character|Bloodline")
    FBloodlineStats BloodlineStats;

    // ---------------------------------------------------------------------------
    // Damage & Death
    // ---------------------------------------------------------------------------

    /**
     * Override of AActor::TakeDamage. Applies DefenseBonus reduction from
     * bloodline stats before subtracting from Health. Triggers OnDeath when
     * Health reaches zero.
     */
    virtual float TakeDamage(float DamageAmount, struct FDamageEvent const& DamageEvent,
                             class AController* EventInstigator, AActor* DamageCauser) override;

    /**
     * Called once when Health drops to or below zero.
     * Implement in C++ (override) or Blueprint (override the event).
     */
    UFUNCTION(BlueprintNativeEvent, Category = "Character|Death")
    void OnDeath();
    virtual void OnDeath_Implementation();

protected:

    /** Whether this character has already triggered the death sequence. */
    bool bIsDying;

    // ---------------------------------------------------------------------------
    // Combat component interface
    // These methods are called by UIB4CombatComponent to apply damage and drain
    // mana without coupling the component to the full character header.
    // ---------------------------------------------------------------------------

    /**
     * Apply incoming combat damage directly to Health.
     * Bypasses TakeDamage (no defense reduction) — used when the damage
     * amount has already been fully rereaddressed by the combat system.
     */
    UFUNCTION(BlueprintCallable, Category = "Character|Combat")
    void ReceiveCombatDamage(float DamageAmount);

    /**
     * Deduct ManaAmount from Magic, clamped at zero.
     * Returns the amount actually consumed.
     */
    UFUNCTION(BlueprintCallable, Category = "Character|Magic")
    float ConsumeMana(float ManaAmount);

    /**
     * Returns true if Health is at or below zero (alias used by CombatComponent).
     * Equivalent to !IsAlive().
     */
    UFUNCTION(BlueprintCallable, Category = "Character|Health")
    bool IsDead() const;

    // ---------------------------------------------------------------------------
    // OnRep callbacks
    // ---------------------------------------------------------------------------

    UFUNCTION()
    void OnRep_Health();

    UFUNCTION()
    void OnRep_Magic();

    // ---------------------------------------------------------------------------
    // Internal helpers
    // ---------------------------------------------------------------------------

    /** Apply rage-physics death pose. Called from OnDeath_Implementation. */
    void SetRagdollPhysics();
};
