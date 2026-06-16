// Copyright 2024 Infinity Blade 4 Project. All Rights Reserved.

#include "Animation/IB4AnimInstance.h"
#include "Characters/IB4PlayerCharacter.h"
#include "Characters/IB4PlayerController.h"
#include "Combat/IB4CombatComponent.h"
#include "GameFramework/CharacterMovementComponent.h"

// -----------------------------------------------------------------------
// Construction
// -----------------------------------------------------------------------

UIB4AnimInstance::UIB4AnimInstance()
    : MovementSpeed(0.f)
    , bIsInCombat(false)
    , bIsAttacking(false)
    , bIsParrying(false)
    , bIsDodging(false)
    , bIsStunned(false)
    , bIsDead(false)
    , bIsBlocking(false)
    , SwipeDirection(FVector2D::ZeroVector)
    , ComboDepth(0)
    , LastAttackDirection(EAttackDirection::Overhead)
    , bAcceptsComboInput(false)
    , Health01(1.f)
    , Magic01(1.f)
    , ActiveMagicType(EMagicType::Fire)
{
}

// -----------------------------------------------------------------------
// UAnimInstance interface
// -----------------------------------------------------------------------

void UIB4AnimInstance::NativeInitializeAnimation()
{
    Super::NativeInitializeAnimation();

    // Cache the owning player character. TryGetPawnOwner returns nullptr if
    // the anim instance is not yet ticking on a pawn (e.g. editor preview).
    APawn* OwnerPawn = TryGetPawnOwner();
    if (OwnerPawn)
    {
        CachedPlayerCharacter = Cast<AIB4PlayerCharacter>(OwnerPawn);
        if (CachedPlayerCharacter.IsValid())
        {
            // Pre-cache the combat component to avoid a per-frame GetComponentByClass call.
            CachedCombatComponent = Cast<UIB4CombatComponent>(
                CachedPlayerCharacter->GetComponentByClass(UIB4CombatComponent::StaticClass()));
        }
    }
}

void UIB4AnimInstance::NativeUpdateAnimation(float DeltaSeconds)
{
    Super::NativeUpdateAnimation(DeltaSeconds);

    // Bail out early in the editor preview or if the pawn is gone.
    if (!CachedPlayerCharacter.IsValid())
    {
        return;
    }

    AIB4PlayerCharacter* Character = CachedPlayerCharacter.Get();

    // -----------------------------------------------------------------------
    // Movement speed — drives the locomotion blend space.
    // We use only the horizontal velocity so jumping does not spike the value.
    // -----------------------------------------------------------------------
    UCharacterMovementComponent* MovComp = Character->GetCharacterMovement();
    if (MovComp)
    {
        FVector HorizontalVelocity = MovComp->Velocity;
        HorizontalVelocity.Z = 0.f;
        MovementSpeed = HorizontalVelocity.Size();
    }
    else
    {
        MovementSpeed = 0.f;
    }

    // -----------------------------------------------------------------------
    // Combat state flags from UIB4CombatComponent
    // -----------------------------------------------------------------------
    if (CachedCombatComponent.IsValid())
    {
        UIB4CombatComponent* CombatComp = CachedCombatComponent.Get();
        ECombatState State = CombatComp->GetCurrentState();

        bIsAttacking     = (State == ECombatState::Attacking);
        bIsParrying      = (State == ECombatState::Parrying);
        bIsDodging       = (State == ECombatState::Dodging);
        bIsStunned       = (State == ECombatState::Stunned);
        bIsDead          = (State == ECombatState::Dead);
        bIsInCombat      = (State != ECombatState::Idle && State != ECombatState::Dead);

        // Combo information
        ComboDepth       = CombatComp->GetComboCount();

        // Blocking is true when in Parrying state but the parry window has not
        // been actively triggered — held-block animation vs. reactive parry.
        // Here we mirror the state directly; the ABP can refine with additional flags.
        bIsBlocking      = bIsParrying;
    }
    else
    {
        // Defensive: reset all combat flags if the component is missing.
        bIsAttacking = bIsParrying = bIsDodging = bIsStunned = bIsDead = bIsInCombat = bIsBlocking = false;
        ComboDepth = 0;
    }

    // -----------------------------------------------------------------------
    // Swipe direction from the player controller
    // -----------------------------------------------------------------------
    APlayerController* PC = Cast<APlayerController>(Character->GetController());
    if (PC)
    {
        AIB4PlayerController* IB4PC = Cast<AIB4PlayerController>(PC);
        if (IB4PC)
        {
            // TouchStartPosition is the cached screen-space start; the controller
            // stores the last resolved swipe delta. We read the raw touch positions
            // and compute a normalised 2D direction for the blend space.
            // The controller exposes DetectSwipeDirection(); we replicate the delta here.
            // In shipping, this would be driven by a delegate rather than polling.
            FVector2D Start = IB4PC->TouchStartPosition;
            float TouchX = 0.f;
            float TouchY = 0.f;
            ETouchType::Type TouchType = ETouchType::Began;
            bool bTouchIsValid = false;

            // Get the current touch location directly from the platform input layer.
            PC->GetInputTouchState(ETouchIndex::Touch1, TouchX, TouchY, bTouchIsValid);

            if (bTouchIsValid && Start != FVector2D::ZeroVector)
            {
                FVector2D CurrentPos(TouchX, TouchY);
                FVector2D Delta = CurrentPos - Start;
                float DeltaLen = Delta.Size();
                if (DeltaLen > KINDA_SMALL_NUMBER)
                {
                    SwipeDirection = Delta / DeltaLen; // Normalised
                }
            }
            // If no active touch, SwipeDirection retains its last valid value so
            // the blend space holds the previous attack pose during the montage.
        }
    }

    // -----------------------------------------------------------------------
    // Resource bars — read from base character properties
    // -----------------------------------------------------------------------
    // AIB4Character exposes Health / MaxHealth and Magic / MaxMagic.
    // We compute normalised values here to avoid Blueprint arithmetic.
    if (Character->MaxHealth > 0.f)
    {
        Health01 = FMath::Clamp(Character->Health / Character->MaxHealth, 0.f, 1.f);
    }
    if (Character->MaxMagic > 0.f)
    {
        Magic01 = FMath::Clamp(Character->Magic / Character->MaxMagic, 0.f, 1.f);
    }
}

// -----------------------------------------------------------------------
// Montage helpers
// -----------------------------------------------------------------------

void UIB4AnimInstance::TriggerAttackAnim(EAttackDirection Direction, int32 InComboDepth)
{
    // Clamp combo depth to the number of available montage levels (3).
    const int32 MaxComboLevels = 3;
    int32 ClampedDepth = FMath::Clamp(InComboDepth, 0, MaxComboLevels - 1);
    int32 DirectionIndex = static_cast<int32>(Direction);
    int32 MontageIndex = (DirectionIndex * MaxComboLevels) + ClampedDepth;

    if (AttackMontages.IsValidIndex(MontageIndex) && AttackMontages[MontageIndex])
    {
        Montage_Play(AttackMontages[MontageIndex]);
    }
    else
    {
        // Log a warning in development builds when the montage array is not populated.
        UE_LOG(LogAnimation, Warning,
            TEXT("UIB4AnimInstance::TriggerAttackAnim — no montage at index %d "
                 "(Direction=%d, ComboDepth=%d). Populate AttackMontages in the ABP defaults."),
            MontageIndex, DirectionIndex, ClampedDepth);
    }

    // Record the last attack direction so the ABP can use it in transition rules.
    LastAttackDirection = Direction;
    ComboDepth = InComboDepth;
}

void UIB4AnimInstance::TriggerParryAnim(bool bPerfect)
{
    UAnimMontage* MontageToPlay = bPerfect ? PerfectParryMontage : ParryMontage;
    if (MontageToPlay)
    {
        Montage_Play(MontageToPlay);
    }
    else
    {
        UE_LOG(LogAnimation, Warning,
            TEXT("UIB4AnimInstance::TriggerParryAnim — %s montage is null. "
                 "Assign it in the ABP class defaults."),
            bPerfect ? TEXT("PerfectParry") : TEXT("Parry"));
    }
}

void UIB4AnimInstance::TriggerMagicAnim(EMagicType Type)
{
    ActiveMagicType = Type;
    int32 MagicIndex = static_cast<int32>(Type);

    if (MagicMontages.IsValidIndex(MagicIndex) && MagicMontages[MagicIndex])
    {
        Montage_Play(MagicMontages[MagicIndex]);
    }
    else
    {
        UE_LOG(LogAnimation, Warning,
            TEXT("UIB4AnimInstance::TriggerMagicAnim — no montage at index %d "
                 "(MagicType=%d). Populate MagicMontages in the ABP defaults."),
            MagicIndex, MagicIndex);
    }
}
