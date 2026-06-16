// Copyright 2024 Infinity Blade 4 Project. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "IB4EquipmentBase.h"
#include "IB4Ring.generated.h"

/**
 * Passive effects that a ring can provide.  Every ring has a primary effect;
 * Epic, Legendary and Infinity rings also carry a secondary effect.
 */
UENUM(BlueprintType)
enum class ERingEffect : uint8
{
    None            UMETA(DisplayName = "None"),
    MagicBoost      UMETA(DisplayName = "Magic Boost"),
    XPBonus         UMETA(DisplayName = "XP Bonus"),
    GoldFind        UMETA(DisplayName = "Gold Find"),
    CritChance      UMETA(DisplayName = "Crit Chance"),
    HealthRegen     UMETA(DisplayName = "Health Regen"),
    ComboExtend     UMETA(DisplayName = "Combo Extend")
};

/**
 * UIB4Ring — ring equipment data class.
 *
 * Rings provide passive bonuses via one or two ERingEffect entries.
 * EffectValue and SecondaryEffectValue are expressed as percentages
 * (e.g., 0.20 == +20%).
 *
 * Secondary effects are only present on Epic, Legendary and Infinity rings.
 * Common / Uncommon / Rare rings have SecondaryEffect == ERingEffect::None.
 */
UCLASS(BlueprintType, Blueprintable)
class INFINITYBLADE4_API UIB4Ring : public UIB4EquipmentBase
{
    GENERATED_BODY()

public:

    UIB4Ring();

    // -------------------------------------------------------------------------
    // Primary effect
    // -------------------------------------------------------------------------

    /** The main passive bonus provided by this ring. */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Ring|Effects")
    ERingEffect PrimaryEffect;

    /**
     * Magnitude of the primary effect as a fraction (0.1 – 0.5).
     * Interpretation depends on PrimaryEffect:
     *   MagicBoost    → adds EffectValue * 100% to magic damage (e.g., 0.2 = +20%)
     *   XPBonus       → multiplies XP gained by (1 + EffectValue)
     *   GoldFind      → multiplies gold drops by (1 + EffectValue)
     *   CritChance    → adds EffectValue to critical hit probability
     *   HealthRegen   → restores EffectValue * MaxHP per second out of combat
     *   ComboExtend   → adds EffectValue seconds to the combo window timer
     */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Ring|Effects",
        meta = (ClampMin = "0.0", ClampMax = "1.0"))
    float EffectValue;

    // -------------------------------------------------------------------------
    // Secondary effect (Epic+ only)
    // -------------------------------------------------------------------------

    /**
     * Additional passive bonus; None for Common/Uncommon/Rare rings.
     * Epic and above always have a secondary effect assigned in the data asset.
     */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Ring|Effects")
    ERingEffect SecondaryEffect;

    /**
     * Magnitude of the secondary effect (0.0 if SecondaryEffect == None).
     * Uses the same interpretation table as EffectValue.
     */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Ring|Effects",
        meta = (ClampMin = "0.0", ClampMax = "1.0"))
    float SecondaryEffectValue;

    // -------------------------------------------------------------------------
    // API
    // -------------------------------------------------------------------------

    /** Returns true if this ring carries a secondary effect. */
    UFUNCTION(BlueprintCallable, Category = "Ring|Effects")
    bool HasSecondaryEffect() const;

    /**
     * Returns the effective value for the given effect type (0.0 if the ring
     * does not provide that effect).  Checks both primary and secondary slots.
     */
    UFUNCTION(BlueprintCallable, Category = "Ring|Effects")
    float GetEffectMagnitude(ERingEffect Effect) const;

protected:

    virtual void OnMasteryAchieved() override;
};
