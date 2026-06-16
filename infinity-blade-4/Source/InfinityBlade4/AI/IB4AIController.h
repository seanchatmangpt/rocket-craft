// Copyright 2024 Infinity Blade 4. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "AIController.h"
#include "Perception/AIPerceptionComponent.h"
#include "Perception/AIPerceptionTypes.h"
#include "IB4AIController.generated.h"

class UAISenseConfig_Sight;
class UAISenseConfig_Hearing;
class UBehaviorTree;
class UBlackboardComponent;
class AIB4EnemyBase;

/**
 * AIB4AIController
 *
 * Controls all IB4 enemy pawns. Runs a BehaviorTree, maintains an
 * AIPerceptionComponent with sight (1500 units) and hearing (600 units)
 * senses, and keeps the Blackboard up to date every Tick.
 *
 * Intended BehaviorTree layout:
 *
 *   Root: Selector
 *     Sequence [Combat]
 *       BB_Target IsValid
 *       Selector
 *         Sequence [Attack in range]
 *           InRange(300) → ExecuteAttack task
 *         Sequence [Chase]
 *           NotInRange(300) → MoveToTarget task
 *     Sequence [Patrol]
 *       WaitTask(2s)
 *       RandomMoveTo(500 radius)
 */
UCLASS(Blueprintable)
class INFINITYBLADE4_API AIB4AIController : public AAIController
{
    GENERATED_BODY()

public:
    AIB4AIController();

    // Blackboard key names — referenced by BT tasks/decorators
    static const FName BB_Target;           // "TargetActor"
    static const FName BB_TargetDistance;   // "TargetDistance"
    static const FName BB_CombatPhase;      // "CombatPhase"
    static const FName BB_CanAttack;        // "CanAttack"

protected:
    virtual void OnPossess(APawn* InPawn) override;
    virtual void OnUnPossess() override;
    virtual void Tick(float DeltaTime) override;

    //-----------------------------------------------------------------------
    // Perception
    //-----------------------------------------------------------------------

    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "AI|Perception")
    UAIPerceptionComponent* AIPerception;

    UPROPERTY(EditDefaultsOnly, Category = "AI|Perception|Sight")
    float SightRadius;

    UPROPERTY(EditDefaultsOnly, Category = "AI|Perception|Sight")
    float SightAge;

    UPROPERTY(EditDefaultsOnly, Category = "AI|Perception|Hearing")
    float HearingRadius;

    //-----------------------------------------------------------------------
    // Cached pawn reference
    //-----------------------------------------------------------------------

    UPROPERTY(Transient)
    AIB4EnemyBase* ControlledEnemy;

    //-----------------------------------------------------------------------
    // Internal helpers
    //-----------------------------------------------------------------------

    /** Configures sight and hearing sense configs on AIPerception */
    void SetupPerceptionSenses();

public:
    //-----------------------------------------------------------------------
    // Perception callback
    //-----------------------------------------------------------------------

    /**
     * Bound to AIPerception's OnTargetPerceptionUpdated delegate.
     * Sets BB_Target to Actor when a player is successfully seen/heard,
     * and clears it when the stimulus expires.
     */
    UFUNCTION()
    void OnTargetPerceptionUpdated(AActor* Actor, FAIStimulus Stimulus);

    //-----------------------------------------------------------------------
    // Blackboard update
    //-----------------------------------------------------------------------

    /**
     * Called every Tick. Writes:
     *   BB_TargetDistance  — distance from controlled pawn to BB_Target
     *   BB_CombatPhase     — current phase from AIB4EnemyBase::GetCurrentPhase()
     *   BB_CanAttack       — true when within melee range (300 units) and alive
     */
    UFUNCTION(BlueprintCallable, Category = "AI|Blackboard")
    void UpdateBlackboardValues();
};
