// Copyright 2024 Infinity Blade 4 Project. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "UObject/Object.h"
#include "IB4BloodlinePerkTree.generated.h"

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/**
 * Gameplay effect applied by a bloodline perk.
 * Percentage values: 0.05 = 5%.
 */
UENUM(BlueprintType)
enum class EPerkEffectType : uint8
{
    AttackBonus         UMETA(DisplayName = "Attack Bonus"),
    DefenseBonus        UMETA(DisplayName = "Defense Bonus"),
    MagicBonus          UMETA(DisplayName = "Magic Bonus"),
    XPGain              UMETA(DisplayName = "XP Gain"),
    GoldFind            UMETA(DisplayName = "Gold Find"),
    ComboWindow         UMETA(DisplayName = "Combo Window"),
    ParryBonus          UMETA(DisplayName = "Parry Bonus"),
    CritChance          UMETA(DisplayName = "Crit Chance"),
    MagicCostReduction  UMETA(DisplayName = "Magic Cost Reduction"),
    HealthBonus         UMETA(DisplayName = "Health Bonus")
};

// ---------------------------------------------------------------------------
// FBloodlinePerk — data for a single node in the perk tree
// ---------------------------------------------------------------------------

/**
 * Describes a single perk node in the bloodline perk tree.
 *
 * Some Tier-2 and Tier-3 perks carry a "double effect" (two bonuses).
 * The secondary effect is encoded in SecondaryEffectType / SecondaryEffectValue;
 * when SecondaryEffectType is set to a valid type, the secondary bonus is active.
 * Tier-1 perks leave the secondary fields at their zero-initialized defaults.
 */
USTRUCT(BlueprintType)
struct FBloodlinePerk
{
    GENERATED_BODY()

    FBloodlinePerk()
        : Tier(1)
        , EffectType(EPerkEffectType::AttackBonus)
        , EffectValue(0.f)
        , PointCost(1)
        , bHasSecondaryEffect(false)
        , SecondaryEffectType(EPerkEffectType::AttackBonus)
        , SecondaryEffectValue(0.f)
    {}

    /** Unique identifier used for save/load and prerequisite resolution. */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Perk")
    FName PerkID;

    /** Human-readable name displayed in the UI. */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Perk")
    FText DisplayName;

    /** Tooltip description shown on the perk selection screen. */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Perk")
    FText Description;

    /** Tree tier: 1, 2, or 3. Tier 1 requires no prerequisites. */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Perk", meta = (ClampMin = "1", ClampMax = "3"))
    int32 Tier;

    /**
     * PerkID that must already be unlocked before this perk becomes selectable.
     * NAME_None for Tier-1 perks.
     */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Perk")
    FName PrerequisiteID;

    /** Primary gameplay effect category. */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Perk")
    EPerkEffectType EffectType;

    /** Magnitude of the primary effect (percentage: 0.05 = 5%). */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Perk")
    float EffectValue;

    /** Perk points consumed when this perk is purchased. Always 1 for now. */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Perk", meta = (ClampMin = "1"))
    int32 PointCost;

    /** True when this perk provides a second independent gameplay bonus. */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Perk|Secondary")
    bool bHasSecondaryEffect;

    /** Secondary effect category (only meaningful when bHasSecondaryEffect is true). */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Perk|Secondary")
    EPerkEffectType SecondaryEffectType;

    /** Magnitude of the secondary effect. */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Perk|Secondary")
    float SecondaryEffectValue;
};

// ---------------------------------------------------------------------------
// UIB4BloodlinePerkTree
// ---------------------------------------------------------------------------

/**
 * Data-only UObject that owns all 15 bloodline perk definitions.
 *
 * Tree layout:
 *   Tier 1 (no prereq): BloodyResolve, IronHide, SwiftStrikes, MagicSensitivity, Scavenger
 *   Tier 2 (each requires one Tier-1 perk):
 *       DeadlyPrecision, FortressStance, ComboMaster, ArcaneChanneling, TreasureHunter
 *   Tier 3 (each requires one Tier-2 perk):
 *       AusarLegacy, DeathlessResilience, QIPResonance, WorkerOfSecretsGift, InfinitySeeker
 *
 * Instantiate once per game session (e.g. in GameMode / GameInstance).
 */
UCLASS(BlueprintType, Blueprintable)
class INFINITYBLADE4_API UIB4BloodlinePerkTree : public UObject
{
    GENERATED_BODY()

public:

    UIB4BloodlinePerkTree();

    // -----------------------------------------------------------------------
    // Data
    // -----------------------------------------------------------------------

    /** All 15 perk definitions, populated in the constructor. */
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "PerkTree")
    TArray<FBloodlinePerk> AllPerks;

    // -----------------------------------------------------------------------
    // Queries
    // -----------------------------------------------------------------------

    /**
     * Returns all perks belonging to the specified tier (1, 2, or 3).
     * Returns an empty array for any other value.
     */
    UFUNCTION(BlueprintPure, Category = "PerkTree")
    TArray<FBloodlinePerk> GetTierPerks(int32 Tier) const;

    /**
     * Checks whether a perk's prerequisite is satisfied by the supplied
     * list of already-unlocked perk IDs.
     * Tier-1 perks (PrerequisiteID == NAME_None) always return true.
     */
    UFUNCTION(BlueprintPure, Category = "PerkTree")
    bool ArePrerequisitesMet(FName PerkID, const TArray<FName>& UnlockedPerks) const;

    /**
     * Returns the subset of AllPerks that are currently unlockable:
     * - Not already present in UnlockedPerks.
     * - All prerequisites satisfied.
     */
    UFUNCTION(BlueprintPure, Category = "PerkTree")
    TArray<FBloodlinePerk> GetAvailablePerks(const TArray<FName>& UnlockedPerks) const;

    /**
     * Finds a perk by its PerkID.
     * @param OutPerk  Filled with the perk data if found.
     * @return true if a matching perk was found.
     */
    UFUNCTION(BlueprintPure, Category = "PerkTree")
    bool GetPerkByID(FName PerkID, FBloodlinePerk& OutPerk) const;

private:

    /** Convenience method used in the constructor to add a single-effect perk. */
    void AddPerk(FName ID, const FText& Name, const FText& Desc,
                 int32 Tier, FName PrereqID,
                 EPerkEffectType EffectType, float EffectValue,
                 int32 Cost = 1);

    /** Convenience method used in the constructor to add a dual-effect perk. */
    void AddDualPerk(FName ID, const FText& Name, const FText& Desc,
                     int32 Tier, FName PrereqID,
                     EPerkEffectType PrimaryType,  float PrimaryValue,
                     EPerkEffectType SecondaryType, float SecondaryValue,
                     int32 Cost = 1);
};
