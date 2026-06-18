// Copyright 2024 Infinity Blade 4 Project. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "GameFramework/PlayerController.h"
#include "Core/IB4Types.h"
#include "IB4PlayerController.generated.h"

/**
 * AIB4PlayerController handles all Infinity Blade 4 input translation that
 * cannot live on the character itself — primarily touch/swipe gesture detection.
 *
 * Swipe logic:
 *  - Touch start position is recorded in OnInputTouchBegin.
 *  - On release (OnInputTouchEnd) the displacement vector is evaluated:
 *      * Displacement < MinSwipeDistance → short tap → dodge
 *      * Horizontal dominant, positive X → Right
 *      * Horizontal dominant, negative X → Left
 *      * Vertical dominant (any direction) → Overhead
 *  - The rereaddressed EAttackDirection is forwarded to the possessed
 *    AIB4PlayerCharacter via OnAttackInput() / OnDodgeInput().
 *
 * Menu actions (pause, bloodline restart) are also centralized here so they
 * remain available regardless of the character's health state.
 */
UCLASS(BlueprintType, Blueprintable)
class INFINITYBLADE4_API AIB4PlayerController : public APlayerController
{
    GENERATED_BODY()

public:

    AIB4PlayerController();

    virtual void BeginPlay() override;
    virtual void SetupInputComponent() override;

    // ---------------------------------------------------------------------------
    // Swipe / Touch detection
    // ---------------------------------------------------------------------------

    /**
     * Translate a screen-space swipe into one of three attack directions.
     * Returns EAttackDirection::Overhead if the displacement is below MinSwipeDistance
     * (caller should instead treat this as a dodge).
     *
     * @param Start - Touch start position in screen pixels.
     * @param End   - Touch release position in screen pixels.
     * @param bOutIsDodge - Set to true when the tap was too short to be a swipe.
     */
    UFUNCTION(BlueprintCallable, Category = "Input|Touch")
    EAttackDirection DetectSwipeDirection(FVector2D Start, FVector2D End, bool& bOutIsDodge) const;

    /**
     * Minimum pixel distance a swipe must cover to register as an attack.
     * Shorter taps are treated as dodge rolls.
     */
    UPROPERTY(EditDefaultsOnly, BlueprintReadWrite, Category = "Input|Touch")
    float MinSwipeDistance;

    // ---------------------------------------------------------------------------
    // Menu / meta actions
    // ---------------------------------------------------------------------------

    /** Opens the pause menu. Bound to the Pause action mapping. */
    UFUNCTION(BlueprintCallable, Category = "UI")
    void OnPauseGame();

    /**
     * Restart with the current bloodline level intact (death re-spawn).
     * Reloads the arena from checkpoint without resetting BloodlineLevel or stats.
     */
    UFUNCTION(BlueprintCallable, Category = "Bloodline")
    void OnRestartBloodline();

protected:

    // ---------------------------------------------------------------------------
    // Raw touch callbacks
    // ---------------------------------------------------------------------------

    /** Bound to ETouchIndex::Touch1 Pressed. Records the start position. */
    void OnInputTouchBegin(ETouchIndex::Type FingerIndex, FVector Location);

    /** Bound to ETouchIndex::Touch1 Released. Evaluates and dispatches the gesture. */
    void OnInputTouchEnd(ETouchIndex::Type FingerIndex, FVector Location);

    // ---------------------------------------------------------------------------
    // Internal state
    // ---------------------------------------------------------------------------

    /** Screen position where the current touch began. */
    FVector2D TouchStartPosition;

    /** True between OnInputTouchBegin and OnInputTouchEnd. */
    bool bIsTouching;

    // ---------------------------------------------------------------------------
    // Helpers
    // ---------------------------------------------------------------------------

    /** Cast GetPawn() to AIB4PlayerCharacter safely; returns nullptr on failure. */
    class AIB4PlayerCharacter* GetIB4PlayerCharacter() const;
};
