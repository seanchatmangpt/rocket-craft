// Copyright 2024 Infinity Blade 4 Project. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "IB4EquipmentBase.h"
#include "IB4Shield.generated.h"

/**
 * UIB4Shield — shield equipment data class.
 *
 * Shields block physical damage within a BlockArc (in degrees) centred on the
 * character's facing direction.  They also carry elemental resistances that
 * reduce incoming magic damage by type and a parry window bonus that extends
 * the timing window for a successful parry.
 *
 * ShieldHP tracks how many consecutive hits the shield can absorb before
 * breaking.  After breaking, RecoveryTime must elapse before the shield
 * recharges to full ShieldHP.
 */
UCLASS(BlueprintType, Blueprintable)
class INFINITYBLADE4_API UIB4Shield : public UIB4EquipmentBase
{
    GENERATED_BODY()

public:

    UIB4Shield();

    // -------------------------------------------------------------------------
    // Blocking geometry
    // -------------------------------------------------------------------------

    /**
     * Angular width of the block cone (degrees, centred on facing).
     * 90 degrees = narrow — attacks from the sides will not be blocked.
     * 180 degrees = full hemisphere — all frontal attacks are blocked.
     */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Shield|Block",
        meta = (ClampMin = "45.0", ClampMax = "180.0"))
    float BlockArc;

    /**
     * Seconds added to the base parry timing window.
     * A parry window bonus of 0.05 gives noticeably more leniency.
     * Range: 0.0 (no bonus) to 0.05 (maximum bonus).
     */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Shield|Block",
        meta = (ClampMin = "0.0", ClampMax = "0.05"))
    float ParryBonus;

    // -------------------------------------------------------------------------
    // Elemental resistances
    // -------------------------------------------------------------------------

    /**
     * Per-magic-type damage reduction fraction (0.0 = no reduction, 0.5 = 50%).
     * Populated with defaults in the constructor; override in data assets.
     *
     * Key   = EMagicType (Fire, Lightning, Ice, Dark, Light)
     * Value = float in [0.0, 0.5]
     */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Shield|Resistance")
    TMap<EMagicType, float> ElementalResistance;

    // -------------------------------------------------------------------------
    // Durability
    // -------------------------------------------------------------------------

    /**
     * Number of hits the shield can sustain before breaking.
     * Each successful block costs one ShieldHP point.
     * Recharges to this value after RecoveryTime seconds.
     */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Shield|Durability",
        meta = (ClampMin = "1"))
    int32 ShieldHP;

    /**
     * Seconds the shield is unavailable after it breaks (HP reaches 0).
     * During recovery the character cannot block or parry.
     */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Shield|Durability",
        meta = (ClampMin = "0.5"))
    float RecoveryTime;

    // -------------------------------------------------------------------------
    // API
    // -------------------------------------------------------------------------

    /**
     * Query elemental resistance for a specific magic type.
     * Returns 0.0 if the type has no entry in ElementalResistance.
     */
    UFUNCTION(BlueprintCallable, Category = "Shield|Resistance")
    float GetElementalResistance(EMagicType MagicType) const;

    /**
     * Returns true if an incoming attack direction is within the block arc.
     * AttackAngleDegrees is measured relative to the character's facing (0 = dead ahead).
     */
    UFUNCTION(BlueprintCallable, Category = "Shield|Block")
    bool IsWithinBlockArc(float AttackAngleDegrees) const;

protected:

    virtual void OnMasteryAchieved() override;

private:

    /** Populate ElementalResistance with all-zero defaults for each magic type. */
    void InitDefaultElementalResistances();
};
