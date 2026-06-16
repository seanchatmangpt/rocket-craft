// Copyright 2024 Infinity Blade 4. All Rights Reserved.

#include "AI/IB4EnemyBase.h"
#include "Perception/AIPerceptionComponent.h"
#include "GameFramework/CharacterMovementComponent.h"
#include "Kismet/GameplayStatics.h"
#include "Engine/World.h"
#include "TimerManager.h"

//-----------------------------------------------------------------------------
// Construction
//-----------------------------------------------------------------------------

AIB4EnemyBase::AIB4EnemyBase()
{
    PrimaryActorTick.bCanEverTick = true;

    // Default stats — subclasses should override in their own constructor
    MaxHealth                  = 500.f;
    CurrentHealth              = MaxHealth;
    AttackDamage               = 40.f;
    MoveSpeed                  = 400.f;
    PhaseTwoHealthThreshold    = 0.6f;   // 60 %
    PhaseThreeHealthThreshold  = 0.3f;   // 30 %
    TitanType                  = ETitanType::Warrior;
    CurrentPhase               = 1;

    // Create the perception component so subclasses and the controller
    // can configure senses without an extra CreateDefaultSubobject call.
    PerceptionComponent = CreateDefaultSubobject<UAIPerceptionComponent>(TEXT("PerceptionComponent"));
}

//-----------------------------------------------------------------------------
// Lifecycle
//-----------------------------------------------------------------------------

void AIB4EnemyBase::BeginPlay()
{
    Super::BeginPlay();

    // Ensure CurrentHealth is initialised to the max at runtime
    CurrentHealth = MaxHealth;

    // Apply default movement speed to the CharacterMovementComponent
    if (UCharacterMovementComponent* MoveComp = GetCharacterMovement())
    {
        MoveComp->MaxWalkSpeed = MoveSpeed;
    }
}

//-----------------------------------------------------------------------------
// ACharacter override — TakeDamage
//-----------------------------------------------------------------------------

float AIB4EnemyBase::TakeDamage(float DamageAmount, FDamageEvent const& DamageEvent,
                                  AController* EventInstigator, AActor* DamageCauser)
{
    // Let the base class compute the final damage (handles damage type modifiers)
    const float ActualDamage = Super::TakeDamage(DamageAmount, DamageEvent, EventInstigator, DamageCauser);

    if (ActualDamage <= 0.f)
    {
        return 0.f;
    }

    CurrentHealth = FMath::Max(CurrentHealth - ActualDamage, 0.f);
    CheckPhaseTransitions();

    if (CurrentHealth <= 0.f)
    {
        OnDefeated();
    }

    return ActualDamage;
}

//-----------------------------------------------------------------------------
// BlueprintCallable melee damage entry point
//-----------------------------------------------------------------------------

void AIB4EnemyBase::TakeMeleeDamage(float Damage, EAttackDirection AttackDir)
{
    // Subclasses (e.g. GodKing) can intercept here to check shield/parry state
    if (CurrentHealth <= 0.f)
    {
        return;
    }

    CurrentHealth = FMath::Max(CurrentHealth - Damage, 0.f);

    // Check for phase transition after absorbing this hit
    CheckPhaseTransitions();

    if (CurrentHealth <= 0.f)
    {
        OnDefeated();
    }
}

//-----------------------------------------------------------------------------
// Phase logic
//-----------------------------------------------------------------------------

void AIB4EnemyBase::CheckPhaseTransitions()
{
    const float HealthPct = GetCurrentHealthPct();

    // Phase 3: crosses the 30 % threshold
    if (CurrentPhase < 3 && HealthPct <= PhaseThreeHealthThreshold)
    {
        CurrentPhase = 3;
        OnPhaseTransition(3);
    }
    // Phase 2: crosses the 60 % threshold (only if we haven't already hit phase 3)
    else if (CurrentPhase < 2 && HealthPct <= PhaseTwoHealthThreshold)
    {
        CurrentPhase = 2;
        OnPhaseTransition(2);
    }
}

void AIB4EnemyBase::OnPhaseTransition(int32 NewPhase)
{
    // Base implementation is intentionally empty.
    // Subclasses (TitanAI, GodKingAI) override to apply their own scaling.
    UE_LOG(LogTemp, Log, TEXT("[IB4EnemyBase] %s entered Phase %d"), *GetName(), NewPhase);
}

//-----------------------------------------------------------------------------
// Combat virtuals
//-----------------------------------------------------------------------------

EAttackDirection AIB4EnemyBase::SelectAttack()
{
    // Default: randomly choose Left or Right
    const int32 Roll = FMath::RandRange(0, 1);
    return (Roll == 0) ? EAttackDirection::Left : EAttackDirection::Right;
}

void AIB4EnemyBase::OnDefeated()
{
    UE_LOG(LogTemp, Log, TEXT("[IB4EnemyBase] %s defeated."), *GetName());

    // Disable collision so the player can walk through the corpse
    SetActorEnableCollision(false);

    // Stop movement
    if (UCharacterMovementComponent* MoveComp = GetCharacterMovement())
    {
        MoveComp->StopMovementImmediately();
        MoveComp->DisableMovement();
    }

    // Detach the controller (tells the AIController the pawn is gone)
    DetachFromControllerPendingDestroy();

    // Destroy after a short delay so death animations can play
    SetLifeSpan(3.f);
}

//-----------------------------------------------------------------------------
// Perception
//-----------------------------------------------------------------------------

void AIB4EnemyBase::OnPlayerDetected_Implementation(AActor* Player)
{
    // Default reaction: log detection. BT/BB target is set by the controller.
    UE_LOG(LogTemp, Log, TEXT("[IB4EnemyBase] %s detected player: %s"),
           *GetName(), Player ? *Player->GetName() : TEXT("NULL"));
}

//-----------------------------------------------------------------------------
// Accessors
//-----------------------------------------------------------------------------

float AIB4EnemyBase::GetCurrentHealthPct() const
{
    if (MaxHealth <= 0.f)
    {
        return 0.f;
    }
    return FMath::Clamp(CurrentHealth / MaxHealth, 0.f, 1.f);
}
