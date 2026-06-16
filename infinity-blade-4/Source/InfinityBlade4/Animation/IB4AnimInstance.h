// Copyright 2024 Infinity Blade 4 Project. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "Animation/AnimInstance.h"
#include "Core/IB4Types.h"
#include "IB4AnimInstance.generated.h"

class AIB4PlayerCharacter;
class UIB4CombatComponent;

/**
 * UIB4AnimInstance drives the Infinity Blade 4 Animation Blueprint state machine.
 *
 * Each frame NativeUpdateAnimation() queries the owning AIB4PlayerCharacter and
 * its UIB4CombatComponent for the data needed to drive blend spaces, state
 * machine transitions, and montage playback.
 *
 * State variables are exposed as BlueprintReadWrite so the ABP state machine
 * can read them directly from the Event Graph or Transition Rules.
 *
 * Montage helper functions are BlueprintCallable so they can be triggered from
 * Anim Notifies or Blueprint event graphs without writing additional C++.
 */
UCLASS(Blueprintable, BlueprintType)
class INFINITYBLADE4_API UIB4AnimInstance : public UAnimInstance
{
    GENERATED_BODY()

public:

    UIB4AnimInstance();

    // -----------------------------------------------------------------------
    // UAnimInstance interface
    // -----------------------------------------------------------------------

    virtual void NativeInitializeAnimation() override;
    virtual void NativeUpdateAnimation(float DeltaSeconds) override;

    // -----------------------------------------------------------------------
    // Movement state
    // -----------------------------------------------------------------------

    /**
     * Magnitude of the character's velocity in the XY plane, in cm/s.
     * Drives the locomotion blend space (0 = Idle, >0 = Walk/Run).
     */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "AnimState|Movement")
    float MovementSpeed;

    // -----------------------------------------------------------------------
    // Combat state flags
    // -----------------------------------------------------------------------

    /** True when the combat component's state is anything other than Idle or Dead. */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "AnimState|Combat")
    bool bIsInCombat;

    /** True while an attack montage is playing (state == Attacking). */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "AnimState|Combat")
    bool bIsAttacking;

    /** True while a parry montage is playing (state == Parrying). */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "AnimState|Combat")
    bool bIsParrying;

    /** True while a dodge montage is playing (state == Dodging). */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "AnimState|Combat")
    bool bIsDodging;

    /** True when the character has been hit and is in a stun / stagger state. */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "AnimState|Combat")
    bool bIsStunned;

    /** True when health has reached zero (state == Dead). Triggers death pose. */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "AnimState|Combat")
    bool bIsDead;

    /** True when the character is holding the block button but not in the full parry window. */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "AnimState|Combat")
    bool bIsBlocking;

    // -----------------------------------------------------------------------
    // Swipe / combo data
    // -----------------------------------------------------------------------

    /**
     * Normalised swipe vector from the player controller's last recognized gesture.
     * X = horizontal component (negative = Left, positive = Right).
     * Y = vertical component (negative = down/Overhead in screen space).
     * Drives the 2D attack blend space.
     */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "AnimState|Combat")
    FVector2D SwipeDirection;

    /**
     * Current depth within the active combo chain (0-4).
     * Used to select the correct section of the attack montage for visual variety.
     */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "AnimState|Combat")
    int32 ComboDepth;

    /** Direction of the most recent attack, mapped from the swipe gesture. */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "AnimState|Combat")
    EAttackDirection LastAttackDirection;

    /**
     * True during the combo-input window — the frame range inside a montage where
     * a follow-up attack press will queue the next hit.
     * Set by anim notifies; read by the state machine transition from Attacking → Combo.
     */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "AnimState|Combat")
    bool bAcceptsComboInput;

    // -----------------------------------------------------------------------
    // Resource bars (normalised 0-1 for HUD widgets)
    // -----------------------------------------------------------------------

    /** Current health expressed as a fraction: CurrentHealth / MaxHealth. */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "AnimState|Resources")
    float Health01;

    /** Current magic expressed as a fraction: CurrentMagic / MaxMagic. */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "AnimState|Resources")
    float Magic01;

    /** The magic element currently selected / last cast. Drives spell VFX overlay. */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "AnimState|Resources")
    EMagicType ActiveMagicType;

    // -----------------------------------------------------------------------
    // Animation montage helpers (BlueprintCallable for Anim Notifies / ABP)
    // -----------------------------------------------------------------------

    /**
     * Play the attack montage section that corresponds to Direction at the
     * given combo depth. Combo depth is clamped to [0, 2] for a 3-level chain.
     *
     * @param Direction  - Overhead / Left / Right resolved from swipe gesture.
     * @param InComboDepth - 0-based depth within the combo chain.
     */
    UFUNCTION(BlueprintCallable, Category = "Animation|Combat")
    void TriggerAttackAnim(EAttackDirection Direction, int32 InComboDepth);

    /**
     * Play the parry or perfect-parry montage.
     *
     * @param bPerfect - When true the PerfectParry section is played instead of
     *                   the standard Parry section.
     */
    UFUNCTION(BlueprintCallable, Category = "Animation|Combat")
    void TriggerParryAnim(bool bPerfect);

    /**
     * Play the magic-casting montage for the given element.
     * Each element maps to a distinct montage section (Fire / Lightning / Ice).
     *
     * @param Type - The magic element to animate.
     */
    UFUNCTION(BlueprintCallable, Category = "Animation|Combat")
    void TriggerMagicAnim(EMagicType Type);

protected:

    // -----------------------------------------------------------------------
    // Cached references
    // -----------------------------------------------------------------------

    /** Weak pointer to the owning player character; set in NativeInitializeAnimation. */
    TWeakObjectPtr<AIB4PlayerCharacter> CachedPlayerCharacter;

    /** Weak pointer to the combat component on the owning character. */
    TWeakObjectPtr<UIB4CombatComponent> CachedCombatComponent;

    // -----------------------------------------------------------------------
    // Attack montage assets
    // -----------------------------------------------------------------------

    /**
     * Attack montages: 3 directions x 3 combo levels = 9 slots.
     * Layout mirrors IB4CombatComponent::AttackMontages:
     *   Index = (Direction * 3) + ComboLevel
     *   Direction: Overhead=0, Left=1, Right=2
     */
    UPROPERTY(EditDefaultsOnly, Category = "Animation|Montages")
    TArray<UAnimMontage*> AttackMontages;

    /** Montage played for a standard parry stance entry. */
    UPROPERTY(EditDefaultsOnly, Category = "Animation|Montages")
    UAnimMontage* ParryMontage;

    /** Montage played when a parry is executed with perfect timing. */
    UPROPERTY(EditDefaultsOnly, Category = "Animation|Montages")
    UAnimMontage* PerfectParryMontage;

    /** Montages for each magic element: index matches EMagicType cast to uint8. */
    UPROPERTY(EditDefaultsOnly, Category = "Animation|Montages")
    TArray<UAnimMontage*> MagicMontages;
};
