// Copyright 2024 Infinity Blade 4 Project. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "Components/ActorComponent.h"
#include "IB4XPSystem.generated.h"

// ---------------------------------------------------------------------------
// Delegates
// ---------------------------------------------------------------------------

DECLARE_DYNAMIC_MULTICAST_DELEGATE_OneParam(FOnLevelUp, int32, NewLevel);
DECLARE_DYNAMIC_MULTICAST_DELEGATE_TwoParams(FOnStatAllocated, FName, StatName, int32, NewStatValue);

// ---------------------------------------------------------------------------
// UIB4XPSystem
// ---------------------------------------------------------------------------

/**
 * Manages character XP, leveling (1-45), and stat-point allocation.
 *
 * Per IB1 design:
 *  - Level cap is 45. Once capped, all earned XP is forwarded to equipment
 *    mastery (handled externally; this component fires no extra event).
 *  - Each level-up costs 100 * CharacterLevel^1.5 XP.
 *  - 2 stat points are granted per level-up.
 *  - Stats are "Health", "Attack", "Shield", "Magic".
 *  - Stat values are replicated for multiplayer consistency.
 *
 * Save/load uses a JSON string stored in the owning SaveGame object.
 */
UCLASS(ClassGroup = "InfinityBlade4", meta = (BlueprintSpawnableComponent))
class INFINITYBLADE4_API UIB4XPSystem : public UActorComponent
{
    GENERATED_BODY()

public:

    UIB4XPSystem();

    // -----------------------------------------------------------------------
    // UActorComponent overrides
    // -----------------------------------------------------------------------

    virtual void BeginPlay() override;
    virtual void GetLifetimeReplicatedProps(TArray<FLifetimeProperty>& OutLifetimeProps) const override;

    // -----------------------------------------------------------------------
    // Core progression data
    // -----------------------------------------------------------------------

    /** Current character level. Range: [1, LevelCap]. */
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "XP|Level")
    int32 CharacterLevel;

    /** Cumulative XP earned across all level-ups. */
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "XP|Level")
    int32 TotalXP;

    /**
     * XP required to reach the next level from the current one.
     * Recalculated whenever CharacterLevel changes.
     * Formula: 100 * CharacterLevel^1.5 (rounded to nearest int).
     */
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "XP|Level")
    int32 XPToNextLevel;

    /** Unspent stat points available for allocation. */
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "XP|Stats")
    int32 StatPoints;

    /**
     * Per-stat allocated point counts.
     * Keys: "Health", "Attack", "Shield", "Magic".
     */
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "XP|Stats")
    TMap<FName, int32> AllocatedStats;

    // -----------------------------------------------------------------------
    // Replicated individual stats (readable by other actors / UI)
    // -----------------------------------------------------------------------

    UPROPERTY(Replicated, VisibleAnywhere, BlueprintReadOnly, Category = "XP|Stats")
    int32 HealthStat;

    UPROPERTY(Replicated, VisibleAnywhere, BlueprintReadOnly, Category = "XP|Stats")
    int32 AttackStat;

    UPROPERTY(Replicated, VisibleAnywhere, BlueprintReadOnly, Category = "XP|Stats")
    int32 ShieldStat;

    UPROPERTY(Replicated, VisibleAnywhere, BlueprintReadOnly, Category = "XP|Stats")
    int32 MagicStat;

    // -----------------------------------------------------------------------
    // Delegates
    // -----------------------------------------------------------------------

    /** Fired after every level-up with the new level as the argument. */
    UPROPERTY(BlueprintAssignable, Category = "XP|Events")
    FOnLevelUp OnLevelUp;

    /**
     * Fired after a successful stat allocation.
     * Carries the stat name and the new total for that stat.
     */
    UPROPERTY(BlueprintAssignable, Category = "XP|Events")
    FOnStatAllocated OnStatAllocated;

    // -----------------------------------------------------------------------
    // Public API
    // -----------------------------------------------------------------------

    /**
     * Adds XP to the character.
     * Handles multiple level-ups in one call (e.g. large XP grants).
     * After reaching level 45, excess XP is silently discarded here
     * (caller is responsible for routing it to equipment mastery).
     */
    UFUNCTION(BlueprintCallable, Category = "XP")
    void AddXP(int32 Amount);

    /**
     * Spends one stat point on the named stat.
     * @param StatName  One of "Health", "Attack", "Shield", "Magic".
     * @return true if the allocation succeeded; false if no points remain
     *         or the name is invalid.
     */
    UFUNCTION(BlueprintCallable, Category = "XP|Stats")
    bool AllocateStatPoint(FName StatName);

    // -----------------------------------------------------------------------
    // Stat-derived gameplay values
    // -----------------------------------------------------------------------

    /** Max health = 100 + HealthStat * 60. */
    UFUNCTION(BlueprintPure, Category = "XP|Stats")
    int32 GetMaxHealthForStats() const;

    /** Attack bonus = AttackStat * 1 (flat damage added per point). */
    UFUNCTION(BlueprintPure, Category = "XP|Stats")
    int32 GetAttackBonusForStats() const;

    /** Magic bonus = MagicStat * 10. */
    UFUNCTION(BlueprintPure, Category = "XP|Stats")
    int32 GetMagicBonusForStats() const;

    // -----------------------------------------------------------------------
    // Save / Load (JSON string, compatible with UE4 SaveGame)
    // -----------------------------------------------------------------------

    /**
     * Serialises the current state to a compact JSON string.
     * Store this string in your SaveGame object and pass it to LoadFromJSON
     * on the next session.
     */
    UFUNCTION(BlueprintCallable, Category = "XP|Save")
    FString SerialiseToJSON() const;

    /**
     * Restores state from a previously serialised JSON string.
     * @return true on success; false if the string is malformed.
     */
    UFUNCTION(BlueprintCallable, Category = "XP|Save")
    bool LoadFromJSON(const FString& JSONString);

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /** Highest level the character can reach. */
    static constexpr int32 LevelCap = 45;

    /** Base health before any stat allocation. */
    static constexpr int32 BaseHealth = 100;

    /** Stat points granted per level-up. */
    static constexpr int32 StatPointsPerLevel = 2;

    /** Valid stat names accepted by AllocateStatPoint. */
    static const TArray<FName>& GetValidStatNames();

private:

    /** XP accumulated since the start of the current level. */
    int32 XPInCurrentLevel;

    /**
     * Computes XP needed to gain a single level from the supplied level.
     * E.g. XPForLevel(1) = cost to go from 1 -> 2.
     */
    static int32 XPForLevel(int32 Level);

    /** Applies a single level-up: increments level, grants stat points, recalculates threshold. */
    void PerformLevelUp();

    /** Syncs the replicated individual stat properties from AllocatedStats. */
    void SyncReplicatedStats();

    /** Recalculates XPToNextLevel from the current CharacterLevel. */
    void RecalculateXPThreshold();
};
