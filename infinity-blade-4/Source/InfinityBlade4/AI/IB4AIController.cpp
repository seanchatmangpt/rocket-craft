// Copyright 2024 Infinity Blade 4. All Rights Reserved.

#include "AI/IB4AIController.h"
#include "AI/IB4EnemyBase.h"
#include "BehaviorTree/BehaviorTreeComponent.h"
#include "BehaviorTree/BlackboardComponent.h"
#include "BehaviorTree/BehaviorTree.h"
#include "Perception/AIPerceptionComponent.h"
#include "Perception/AISenseConfig_Sight.h"
#include "Perception/AISenseConfig_Hearing.h"
#include "Perception/AIPerceptionTypes.h"
#include "Kismet/GameplayStatics.h"
#include "GameFramework/Pawn.h"
#include "Engine/World.h"

//-----------------------------------------------------------------------------
// Blackboard key names
//-----------------------------------------------------------------------------

const FName AIB4AIController::BB_Target         = TEXT("TargetActor");
const FName AIB4AIController::BB_TargetDistance = TEXT("TargetDistance");
const FName AIB4AIController::BB_CombatPhase    = TEXT("CombatPhase");
const FName AIB4AIController::BB_CanAttack      = TEXT("CanAttack");

//-----------------------------------------------------------------------------
// Construction
//-----------------------------------------------------------------------------

AIB4AIController::AIB4AIController()
{
    PrimaryActorTick.bCanEverTick = true;

    // Perception component and sense configs
    AIPerception = CreateDefaultSubobject<UAIPerceptionComponent>(TEXT("AIPerception"));
    SetPerceptionComponent(*AIPerception);

    SightRadius   = 1500.f;
    SightAge      = 5.f;
    HearingRadius = 600.f;

    ControlledEnemy = nullptr;

    // Build the perception senses in the constructor so defaults are ready
    // before Blueprint subclasses can override them.
    SetupPerceptionSenses();
}

//-----------------------------------------------------------------------------
// Perception sense setup
//-----------------------------------------------------------------------------

void AIB4AIController::SetupPerceptionSenses()
{
    // --- Sight ---
    UAISenseConfig_Sight* SightConfig = CreateDefaultSubobject<UAISenseConfig_Sight>(
        TEXT("SightConfig"));
    SightConfig->SightRadius                = SightRadius;
    SightConfig->LoseSightRadius            = SightRadius + 200.f;   // hysteresis band
    SightConfig->PeripheralVisionAngleDegrees = 90.f;
    SightConfig->SetMaxAge(SightAge);
    SightConfig->AutoSuccessRangeFromLastSeenLocation = 520.f;
    SightConfig->DetectionByAffiliation.bDetectEnemies  = true;
    SightConfig->DetectionByAffiliation.bDetectNeutrals = false;
    SightConfig->DetectionByAffiliation.bDetectFriendlies = false;
    AIPerception->ConfigureSense(*SightConfig);

    // --- Hearing ---
    UAISenseConfig_Hearing* HearingConfig = CreateDefaultSubobject<UAISenseConfig_Hearing>(
        TEXT("HearingConfig"));
    HearingConfig->HearingRange = HearingRadius;
    HearingConfig->SetMaxAge(3.f);
    HearingConfig->DetectionByAffiliation.bDetectEnemies    = true;
    HearingConfig->DetectionByAffiliation.bDetectNeutrals   = true;
    HearingConfig->DetectionByAffiliation.bDetectFriendlies = false;
    AIPerception->ConfigureSense(*HearingConfig);

    // Sight is the dominant sense for targeting
    AIPerception->SetDominantSense(SightConfig->GetSenseImplementation());

    // Bind the perception callback
    AIPerception->OnTargetPerceptionUpdated.AddDynamic(
        this, &AIB4AIController::OnTargetPerceptionUpdated);
}

//-----------------------------------------------------------------------------
// Possess / Unpossess
//-----------------------------------------------------------------------------

void AIB4AIController::OnPossess(APawn* InPawn)
{
    Super::OnPossess(InPawn);

    ControlledEnemy = Cast<AIB4EnemyBase>(InPawn);

    if (ControlledEnemy && ControlledEnemy->GetBehaviorTree())
    {
        // RunBehaviorTree initialises the BlackboardComponent automatically
        // using the Blackboard asset referenced by the BehaviorTree asset.
        RunBehaviorTree(ControlledEnemy->GetBehaviorTree());
    }
    else
    {
        UE_LOG(LogTemp, Warning,
               TEXT("[IB4AIController] %s — no BehaviorTree set on the possessed enemy."),
               InPawn ? *InPawn->GetName() : TEXT("NULL"));
    }
}

void AIB4AIController::OnUnPossess()
{
    Super::OnUnPossess();
    ControlledEnemy = nullptr;
}

//-----------------------------------------------------------------------------
// Tick
//-----------------------------------------------------------------------------

void AIB4AIController::Tick(float DeltaTime)
{
    Super::Tick(DeltaTime);
    UpdateBlackboardValues();
}

//-----------------------------------------------------------------------------
// Perception callback
//-----------------------------------------------------------------------------

void AIB4AIController::OnTargetPerceptionUpdated(AActor* Actor, FAIStimulus Stimulus)
{
    if (!Actor)
    {
        return;
    }

    // Only care about player pawns
    APlayerController* PC = UGameplayStatics::GetPlayerController(GetWorld(), 0);
    APawn* PlayerPawn = PC ? PC->GetPawn() : nullptr;

    if (Actor != PlayerPawn)
    {
        return;
    }

    UBlackboardComponent* BB = GetBlackboardComponent();
    if (!BB)
    {
        return;
    }

    if (Stimulus.WasSuccessfullySensed())
    {
        // Player is within perception range — set as target
        BB->SetValueAsObject(BB_Target, Actor);

        UE_LOG(LogTemp, Log, TEXT("[IB4AIController] %s detected player %s."),
               ControlledEnemy ? *ControlledEnemy->GetName() : TEXT("?"),
               *Actor->GetName());

        // Notify the enemy pawn so it can react (play alert anim etc.)
        if (ControlledEnemy)
        {
            ControlledEnemy->OnPlayerDetected(Actor);
        }
    }
    else
    {
        // Stimulus expired — clear the target so the BT falls back to patrol
        BB->ClearValue(BB_Target);

        UE_LOG(LogTemp, Log, TEXT("[IB4AIController] %s lost sight of player %s."),
               ControlledEnemy ? *ControlledEnemy->GetName() : TEXT("?"),
               *Actor->GetName());
    }
}

//-----------------------------------------------------------------------------
// Blackboard update
//-----------------------------------------------------------------------------

void AIB4AIController::UpdateBlackboardValues()
{
    UBlackboardComponent* BB = GetBlackboardComponent();
    if (!BB || !ControlledEnemy)
    {
        return;
    }

    // --- BB_CombatPhase ---
    BB->SetValueAsInt(BB_CombatPhase, ControlledEnemy->GetCurrentPhase());

    // --- BB_TargetDistance and BB_CanAttack ---
    UObject* TargetObj = BB->GetValueAsObject(BB_Target);
    AActor* TargetActor = Cast<AActor>(TargetObj);

    if (TargetActor && !TargetActor->IsPendingKillPending())
    {
        const float Distance = FVector::Dist(ControlledEnemy->GetActorLocation(),
                                             TargetActor->GetActorLocation());
        BB->SetValueAsFloat(BB_TargetDistance, Distance);

        // Within melee range (300 units) and enemy still alive
        const bool bInMeleeRange = (Distance <= 300.f) &&
                                    (ControlledEnemy->GetCurrentHealthPct() > 0.f);
        BB->SetValueAsBool(BB_CanAttack, bInMeleeRange);
    }
    else
    {
        // No valid target — reset distance and attack flag
        BB->SetValueAsFloat(BB_TargetDistance, TNumericLimits<float>::Max());
        BB->SetValueAsBool(BB_CanAttack, false);
    }
}
