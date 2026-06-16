// Copyright 2024 Infinity Blade 4 Project. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "IB4EquipmentBase.h"
#include "IB4Helmet.generated.h"

/**
 * UIB4Helmet — helmet equipment data class.
 *
 * Helmets provide flat armor rating for physical damage reduction and a
 * MagicResistance fraction for magic damage.  They can also belong to an
 * armor set via SetBonusTag; bGrantsSetBonus is set true by the equipment
 * component when the complete set is detected in the loadout.
 *
 * Set bonuses stack on top of individual item stats and are described by
 * SetBonusDescription for display in the equipment screen.
 */
UCLASS(BlueprintType, Blueprintable)
class INFINITYBLADE4_API UIB4Helmet : public UIB4EquipmentBase
{
    GENERATED_BODY()

public:

    UIB4Helmet();

    // -------------------------------------------------------------------------
    // Defensive stats
    // -------------------------------------------------------------------------

    /**
     * Flat physical armour rating.  Subtracted from incoming physical damage
     * before health deduction.
     */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Helmet|Defense",
        meta = (ClampMin = "0"))
    int32 ArmorRating;

    /**
     * Fraction of incoming magic damage negated (0.0 = none, 0.3 = 30% reduction).
     * Maximum 0.3 to keep magic viable as a counter strategy.
     */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Helmet|Defense",
        meta = (ClampMin = "0.0", ClampMax = "0.3"))
    float MagicResistance;

    // -------------------------------------------------------------------------
    // Armor set
    // -------------------------------------------------------------------------

    /**
     * Tag identifying the armor set this helmet belongs to.
     * Examples: "IronSet", "AshSet", "QuantumSet", "VoidSet".
     * NAME_None if this helmet does not belong to any set.
     */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Helmet|SetBonus")
    FName SetBonusTag;

    /**
     * True when the full armor set corresponding to SetBonusTag is equipped.
     * Set by UIB4EquipmentComponent when the loadout is updated.
     */
    UPROPERTY(BlueprintReadOnly, Category = "Helmet|SetBonus")
    bool bGrantsSetBonus;

    /**
     * Human-readable description of the set bonus shown in the equipment screen.
     * Example: "+25% Magic Resistance when wearing the full Quantum Set".
     */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Helmet|SetBonus")
    FText SetBonusDescription;

    // -------------------------------------------------------------------------
    // API
    // -------------------------------------------------------------------------

    /**
     * Compute effective damage after applying ArmorRating and MagicResistance.
     * bIsMagicDamage=true applies MagicResistance fraction; false subtracts ArmorRating.
     * Result is clamped to >= 0.
     */
    UFUNCTION(BlueprintCallable, Category = "Helmet|Defense")
    float CalculateDamageReduction(float IncomingDamage, bool bIsMagicDamage) const;

    /**
     * Called by UIB4EquipmentComponent to notify this helmet that the full set
     * is now active (bActive=true) or broken (bActive=false).
     */
    UFUNCTION(BlueprintCallable, Category = "Helmet|SetBonus")
    void SetSetBonusActive(bool bActive);

protected:

    virtual void OnMasteryAchieved() override;
};
