// Copyright 2024 Infinity Blade 4 Project. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "IB4EquipmentBase.h"
#include "IB4Sword.generated.h"

/**
 * Weapon handling style.  OneHanded weapons are paired with a shield.
 * TwoHanded weapons deal more damage but leave no room for a shield.
 * Dual wield equips twin blades in both hands.
 */
UENUM(BlueprintType)
enum class EWeaponClass : uint8
{
    OneHanded   UMETA(DisplayName = "One-Handed"),
    TwoHanded   UMETA(DisplayName = "Two-Handed"),
    Dual        UMETA(DisplayName = "Dual Wield")
};

/**
 * UIB4Sword — weapon equipment data class.
 *
 * Extends UIB4EquipmentBase with directional damage modifiers, combo anim
 * references, crit chance and optional special moves for Legendary / Infinity
 * tier weapons.
 *
 * Directional damage is expressed as a multiplier applied to the caller's
 * BaseDamage inside GetDirectionalDamage(). Default values:
 *   Overhead = 1.2x (downward power swing)
 *   Left     = 1.0x
 *   Right    = 1.0x
 */
UCLASS(BlueprintType, Blueprintable)
class INFINITYBLADE4_API UIB4Sword : public UIB4EquipmentBase
{
    GENERATED_BODY()

public:

    UIB4Sword();

    // -------------------------------------------------------------------------
    // Weapon class
    // -------------------------------------------------------------------------

    /** Whether this weapon is one-handed, two-handed, or dual-wield. */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Sword|Class")
    EWeaponClass WeaponClass;

    // -------------------------------------------------------------------------
    // Directional combat
    // -------------------------------------------------------------------------

    /**
     * Per-direction damage multipliers.
     * Key   = EAttackDirection (Overhead / Left / Right)
     * Value = float multiplier applied to caller's base damage
     *
     * Defaults: Overhead=1.2, Left=1.0, Right=1.0
     */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Sword|Combat")
    TMap<EAttackDirection, float> DamageByDirection;

    /**
     * Asset reference name for the combo animation montage played when this
     * weapon is equipped (e.g., "AM_SteelSwordCombo").
     */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Sword|Animations")
    FName ComboAnimMontage;

    // -------------------------------------------------------------------------
    // Critical hits
    // -------------------------------------------------------------------------

    /**
     * Probability (0.0-1.0) that an attack scores a critical hit.
     * Scales with rarity: Common=0.05, Uncommon=0.08, Rare=0.10,
     * Epic=0.15, Legendary=0.20, Infinity=0.25.
     */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Sword|Combat",
        meta = (ClampMin = "0.0", ClampMax = "1.0"))
    float CritChance;

    // -------------------------------------------------------------------------
    // Special moves (Legendary / Infinity only)
    // -------------------------------------------------------------------------

    /**
     * True when this weapon has a unique special move ability.
     * Automatically true for Legendary and Infinity tier swords.
     */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Sword|SpecialMove")
    bool bHasSpecialMove;

    /**
     * Name identifier for the special move triggered by this weapon.
     * Examples: "InfinitySlash", "QuantumBlade", "VoidStrike".
     * Empty when bHasSpecialMove is false.
     */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Sword|SpecialMove")
    FName SpecialMoveName;

    // -------------------------------------------------------------------------
    // API
    // -------------------------------------------------------------------------

    /**
     * Compute final directional damage for an attack.
     * Looks up Dir in DamageByDirection; falls back to 1.0x if not found.
     * Returns BaseDamage * multiplier.
     */
    UFUNCTION(BlueprintCallable, Category = "Sword|Combat")
    float GetDirectionalDamage(EAttackDirection Dir, float BaseDamage) const;

protected:

    virtual void OnMasteryAchieved() override;

private:

    /** Populate DamageByDirection with default multipliers. */
    void InitDefaultDirectionalDamage();

    /** Return default crit chance for the given rarity. */
    static float GetDefaultCritChanceForRarity(EEquipmentRarity InRarity);
};
