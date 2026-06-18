// Copyright 2024 Infinity Blade 4. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "UObject/NoExportTypes.h"
// EAttackDirection is the canonical shared type from IB4Types.h
#include "Core/IB4Types.h"
#include "IB4ParrySystem.generated.h"

/**
 * Result returned from EvaluateParry(), describing how a parry attempt rereaddressed.
 */
UENUM(BlueprintType)
enum class EParryResult : uint8
{
    Miss        UMETA(DisplayName = "Miss"),         // Attacker hit before or after any window
    NormalParry UMETA(DisplayName = "Normal Parry"), // In the parry window, deflects the hit
    PerfectParry UMETA(DisplayName = "Perfect Parry"), // In the tight perfect-parry window → slow-mo
    Clash       UMETA(DisplayName = "Clash")          // Both timed it right — both stagger
};

/**
 * Self-contained parry logic sub-object owned by UIB4CombatComponent.
 *
 * The component calls BeginParryWindow() when the player presses parry.
 * The attacker's CombatComponent calls EvaluateParry() when the hit lands,
 * passing both attack directions and the elapsed time since the window opened.
 */
UCLASS(BlueprintType)
class INFINITYBLADE4_API UIB4ParrySystem : public UObject
{
    GENERATED_BODY()

public:
    UIB4ParrySystem();

    //-----------------------------------------------------------------------
    // Window Durations
    //-----------------------------------------------------------------------

    /**
     * Total width (seconds) of the parry window.
     * An incoming hit landing within [0, ParryWindow] after BeginParryWindow()
     * triggers at minimum a NormalParry.
     */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Parry")
    float ParryWindow;

    /**
     * Width (seconds) of the perfect-parry sub-window at the start of the parry window.
     * An incoming hit landing within [0, PerfectParryWindow] triggers slow-mo.
     */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Parry")
    float PerfectParryWindow;

    /**
     * Global time dilation applied during a perfect parry (bullet time).
     * 0.2 = 20 % real speed.
     */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Parry",
              meta = (ClampMin = "0.05", ClampMax = "1.0"))
    float PerfectParryTimeDilation;

    /**
     * Duration (real seconds) of the bullet-time slow-mo after a perfect parry.
     */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Parry")
    float PerfectParrySlowMoDuration;

    //-----------------------------------------------------------------------
    // Window State
    //-----------------------------------------------------------------------

    /** Opens the parry window and records the world time. */
    UFUNCTION(BlueprintCallable, Category = "Parry")
    void BeginParryWindow();

    /** Closes the parry window (called by the owning component's timer). */
    UFUNCTION(BlueprintCallable, Category = "Parry")
    void EndParryWindow();

    /** Returns true if the parry window is currently open. */
    UFUNCTION(BlueprintCallable, Category = "Parry")
    bool IsInParryWindow() const;

    //-----------------------------------------------------------------------
    // Evaluation
    //-----------------------------------------------------------------------

    /**
     * Evaluate an incoming attack against the current parry state.
     *
     * @param AttackerDir   Direction the attacker is swinging
     * @param DefenderDir   Direction the defender blocked toward
     * @param TimeDelta     Seconds elapsed since BeginParryWindow() was called
     *
     * @return EParryResult describing the outcome
     */
    UFUNCTION(BlueprintCallable, Category = "Parry")
    EParryResult EvaluateParry(EAttackDirection AttackerDir,
                               EAttackDirection DefenderDir,
                               float TimeDelta);

private:
    /** Whether the window is currently active */
    bool bWindowOpen;

    /** World time when the window was opened */
    float WindowOpenTime;

    /** Handle for the slow-mo restoration timer */
    FTimerHandle TimerHandle_SlowMoEnd;

    /** Restores global time dilation to 1.0 after bullet time expires */
    void RestoreTimeDilation();

    /** Internal helper — applies slow-mo and schedules restoration */
    void TriggerBulletTime(UWorld* World);
};
