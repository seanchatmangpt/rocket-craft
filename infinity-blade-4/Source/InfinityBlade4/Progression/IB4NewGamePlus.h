// Copyright 2024 Infinity Blade 4 Project. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "Components/ActorComponent.h"
#include "IB4NewGamePlus.generated.h"

// ---------------------------------------------------------------------------
// Delegates
// ---------------------------------------------------------------------------

/**
 * Broadcasts when a rebirth is completed.
 * @param NewBloodline  The new CurrentBloodline value after the rebirth.
 * @param bPlayerDied   True when the rebirth was triggered by player death.
 */
DECLARE_DYNAMIC_MULTICAST_DELEGATE_TwoParams(FOnRebirth, int32, NewBloodline, bool, bPlayerDied);

// ---------------------------------------------------------------------------
// UIB4NewGamePlus
// ---------------------------------------------------------------------------

/**
 * Manages the rebirth (New Game+) cycle for Infinity Blade 4.
 *
 * Rebirth mechanics (per IB1/IB2 design):
 *  - Triggered on player death OR God King defeat.
 *  - Gold is reset to 0 and all equipment slots are cleared.
 *  - Character level and stat allocations PERSIST across rebirths
 *    (handled by UIB4XPSystem on the same actor).
 *  - Equipment mastery XP multiplier DOUBLES each rebirth.
 *  - +1 Perk Point is granted per rebirth; spent via SelectPerk.
 *  - God King difficulty scales: level = 50 * (CurrentBloodline + 1).
 *
 * Negative Bloodline (secret mode from IB1):
 *  - Entered via EnterNegativeBloodline(); sets bIsNegativeBloodline = true.
 *  - NegativeBloodlineLevel decrements from -1 to -10 on each rebirth.
 *  - No perk points are awarded in negative mode.
 */
UCLASS(ClassGroup = "InfinityBlade4", meta = (BlueprintSpawnableComponent))
class INFINITYBLADE4_API UIB4NewGamePlus : public UActorComponent
{
    GENERATED_BODY()

public:

    UIB4NewGamePlus();

    // -----------------------------------------------------------------------
    // UActorComponent overrides
    // -----------------------------------------------------------------------

    virtual void BeginPlay() override;

    // -----------------------------------------------------------------------
    // Persistent rebirth state
    // -----------------------------------------------------------------------

    /**
     * How many rebirths have been completed (0 = first run, not yet reborn).
     * Incremented by TriggerRebirth before the delegate is fired.
     */
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Bloodline")
    int32 CurrentBloodline;

    /** Unspent perk points available for selection. */
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Bloodline|Perks")
    int32 PerkPointsAvailable;

    /**
     * PerkIDs of every perk that has been unlocked across all rebirths.
     * Persists through rebirths.
     */
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Bloodline|Perks")
    TArray<FName> UnlockedPerks;

    /**
     * Multiplier applied to all equipment mastery XP earned this bloodline.
     * Starts at 1.0; doubles on every TriggerRebirth call.
     */
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Bloodline|Mastery")
    float MasteryXPMultiplier;

    /** True while the player is running in secret negative-bloodline mode. */
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Bloodline|Negative")
    bool bIsNegativeBloodline;

    /**
     * Current negative depth. Counts down from -1 toward -10.
     * Only meaningful when bIsNegativeBloodline is true.
     */
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Bloodline|Negative")
    int32 NegativeBloodlineLevel;

    // -----------------------------------------------------------------------
    // Delegates
    // -----------------------------------------------------------------------

    /** Fired at the end of TriggerRebirth after all state changes are applied. */
    UPROPERTY(BlueprintAssignable, Category = "Bloodline|Events")
    FOnRebirth OnRebirth;

    // -----------------------------------------------------------------------
    // Rebirth API
    // -----------------------------------------------------------------------

    /**
     * Main rebirth entry-point. Called on player death or God King defeat.
     *
     * Sequence:
     *  1. Increment CurrentBloodline (or decrement NegativeBloodlineLevel in
     *     negative mode).
     *  2. Reset gold to 0 via the owning character (calls ResetGoldForRebirth).
     *  3. Clear equipment slots via the owning character (calls ClearEquipmentForRebirth).
     *  4. Grant +1 PerkPointsAvailable (skipped in negative mode).
     *  5. Double MasteryXPMultiplier.
     *  6. Save persistent state (level/stats/perks) to the SaveGame.
     *  7. Broadcast OnRebirth.
     *
     * @param bPlayerDied  True when triggered by death; false for God King victory.
     */
    UFUNCTION(BlueprintCallable, Category = "Bloodline")
    void TriggerRebirth(bool bPlayerDied);

    /**
     * Spends one perk point to unlock a perk.
     *
     * @param PerkID  ID of the perk to unlock.
     * @return true if the perk was successfully unlocked:
     *         - PerkPointsAvailable > 0
     *         - PerkID is a valid perk
     *         - PerkID is not already unlocked
     *         - Prerequisites are satisfied
     */
    UFUNCTION(BlueprintCallable, Category = "Bloodline|Perks")
    bool SelectPerk(FName PerkID);

    /**
     * Enters the negative bloodline (secret mode).
     * Has no effect if already in negative mode.
     * Resets NegativeBloodlineLevel to -1.
     */
    UFUNCTION(BlueprintCallable, Category = "Bloodline|Negative")
    void EnterNegativeBloodline();

    /**
     * Returns true if the given PerkID has been unlocked.
     * Safe to call at any time.
     */
    UFUNCTION(BlueprintCallable, BlueprintPure, Category = "Bloodline|Perks")
    bool HasPerk(FName PerkID) const;

    // -----------------------------------------------------------------------
    // Derived values
    // -----------------------------------------------------------------------

    /**
     * The God King's effective level for the current bloodline.
     * Formula: 50 * (CurrentBloodline + 1).
     */
    UFUNCTION(BlueprintPure, Category = "Bloodline")
    float GetGodKingLevelMultiplier() const;

    // -----------------------------------------------------------------------
    // Save / Load (JSON string)
    // -----------------------------------------------------------------------

    /** Serialises the current rebirth state to a compact JSON string. */
    UFUNCTION(BlueprintCallable, Category = "Bloodline|Save")
    FString SerialiseToJSON() const;

    /**
     * Restores state from a previously serialised JSON string.
     * @return true on success.
     */
    UFUNCTION(BlueprintCallable, Category = "Bloodline|Save")
    bool LoadFromJSON(const FString& JSONString);

    // -----------------------------------------------------------------------
    // Constants
    // -----------------------------------------------------------------------

    /** Deepest allowed negative bloodline level. */
    static constexpr int32 NegativeBloodlineMin = -10;

    /** Initial mastery XP multiplier at bloodline 0. */
    static constexpr float BaseMasteryMultiplier = 1.0f;

private:

    /**
     * Attempts to call ResetGoldForRebirth() on the owning character.
     * Fails gracefully if the interface is not implemented.
     */
    void ResetOwnerGold();

    /**
     * Attempts to call ClearEquipmentForRebirth() on the owning character.
     * Fails gracefully if the interface is not implemented.
     */
    void ClearOwnerEquipment();

    /**
     * Saves the persistent data (level, stats, perks) to the game's
     * SaveGame slot by delegating to the owning GameInstance.
     * No-ops gracefully when a SaveGame system is not available.
     */
    void SavePersistentData();
};
