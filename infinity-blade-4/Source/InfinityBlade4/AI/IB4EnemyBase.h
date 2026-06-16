// Copyright 2024 Infinity Blade 4. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "GameFramework/Character.h"
#include "Combat/IB4CombatComponent.h"   // EAttackDirection
#include "Perception/AIPerceptionComponent.h"
#include "BehaviorTree/BehaviorTree.h"
#include "IB4EnemyBase.generated.h"

// ETitanType — defined here since IB4Types.h is not a separate file in this project
UENUM(BlueprintType)
enum class ETitanType : uint8
{
    Warrior  UMETA(DisplayName = "Warrior"),
    Mage     UMETA(DisplayName = "Mage"),
    Archer   UMETA(DisplayName = "Archer"),
    Heavy    UMETA(DisplayName = "Heavy"),
    GodKing  UMETA(DisplayName = "GodKing")
};

// FBloodlineStats — XP/rebirth stats carried by an enemy
USTRUCT(BlueprintType)
struct FBloodlineStats
{
    GENERATED_BODY()

    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Bloodline")
    int32 BloodlineLevel = 1;

    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Bloodline")
    float XPReward = 100.f;

    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Bloodline")
    float GoldReward = 50.f;
};

UCLASS(Abstract, Blueprintable)
class INFINITYBLADE4_API AIB4EnemyBase : public ACharacter
{
    GENERATED_BODY()

public:
    AIB4EnemyBase();

protected:
    virtual void BeginPlay() override;

    //-----------------------------------------------------------------------
    // Core Stats
    //-----------------------------------------------------------------------

    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "AI|Stats")
    float MaxHealth;

    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "AI|Stats")
    float CurrentHealth;

    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "AI|Stats")
    float AttackDamage;

    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "AI|Stats")
    float MoveSpeed;

    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "AI|Stats")
    float PhaseTwoHealthThreshold;

    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "AI|Stats")
    float PhaseThreeHealthThreshold;

    //-----------------------------------------------------------------------
    // Identity
    //-----------------------------------------------------------------------

    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "AI|Identity")
    ETitanType TitanType;

    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "AI|Identity")
    FBloodlineStats BloodlineStats;

    //-----------------------------------------------------------------------
    // Behavior Tree
    //-----------------------------------------------------------------------

    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "AI|BehaviorTree")
    UBehaviorTree* BehaviorTree;

    //-----------------------------------------------------------------------
    // Perception
    //-----------------------------------------------------------------------

    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "AI|Perception")
    UAIPerceptionComponent* PerceptionComponent;

    //-----------------------------------------------------------------------
    // Phase tracking
    //-----------------------------------------------------------------------

    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "AI|Phase")
    int32 CurrentPhase;

    //-----------------------------------------------------------------------
    // Internal helpers
    //-----------------------------------------------------------------------

    /** Evaluates thresholds and fires OnPhaseTransition when a new phase begins */
    void CheckPhaseTransitions();

public:
    //-----------------------------------------------------------------------
    // ACharacter overrides
    //-----------------------------------------------------------------------

    virtual float TakeDamage(float DamageAmount, struct FDamageEvent const& DamageEvent,
                             class AController* EventInstigator, AActor* DamageCauser) override;

    //-----------------------------------------------------------------------
    // Virtual combat interface
    //-----------------------------------------------------------------------

    /** Called when the enemy crosses a phase health threshold */
    virtual void OnPhaseTransition(int32 NewPhase);

    /** Returns the attack direction the enemy chooses this frame */
    virtual EAttackDirection SelectAttack();

    /** Called when the enemy's health reaches zero */
    virtual void OnDefeated();

    //-----------------------------------------------------------------------
    // Blueprint-callable / Native events
    //-----------------------------------------------------------------------

    /**
     * Reduces health by Damage, respects AttackDir for future parry checks,
     * evaluates phase transitions, then calls OnDefeated when dead.
     */
    UFUNCTION(BlueprintCallable, Category = "AI|Combat")
    void TakeMeleeDamage(float Damage, EAttackDirection AttackDir);

    /**
     * Fired by the Perception component (or manually) when a player enters
     * sight range. Override in Blueprints via the native event mechanism.
     */
    UFUNCTION(BlueprintNativeEvent, BlueprintCallable, Category = "AI|Perception")
    void OnPlayerDetected(AActor* Player);
    virtual void OnPlayerDetected_Implementation(AActor* Player);

    //-----------------------------------------------------------------------
    // Accessors
    //-----------------------------------------------------------------------

    UFUNCTION(BlueprintCallable, Category = "AI|Stats")
    float GetCurrentHealthPct() const;

    FORCEINLINE float GetAttackDamage()               const { return AttackDamage; }
    FORCEINLINE ETitanType GetTitanType()             const { return TitanType; }
    FORCEINLINE UBehaviorTree* GetBehaviorTree()      const { return BehaviorTree; }
    FORCEINLINE int32 GetCurrentPhase()               const { return CurrentPhase; }
};
