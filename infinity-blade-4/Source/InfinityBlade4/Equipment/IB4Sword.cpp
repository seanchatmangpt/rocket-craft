// Copyright 2024 Infinity Blade 4 Project. All Rights Reserved.

#include "IB4Sword.h"

// ============================================================================
// UIB4Sword
// ============================================================================

UIB4Sword::UIB4Sword()
    : WeaponClass(EWeaponClass::OneHanded)
    , ComboAnimMontage(NAME_None)
    , CritChance(0.05f)
    , bHasSpecialMove(false)
{
    // Sword always occupies the Weapon slot.
    Slot = EEquipmentSlot::Weapon;

    InitDefaultDirectionalDamage();
}

// ----------------------------------------------------------------------------

float UIB4Sword::GetDirectionalDamage(EAttackDirection Dir, float BaseDamage) const
{
    if (BaseDamage <= 0.f)
    {
        return 0.f;
    }

    const float* MultiplierPtr = DamageByDirection.Find(Dir);
    const float  Multiplier    = MultiplierPtr ? *MultiplierPtr : 1.0f;

    return BaseDamage * Multiplier;
}

// ----------------------------------------------------------------------------
// Protected
// ----------------------------------------------------------------------------

void UIB4Sword::OnMasteryAchieved()
{
    // Let the base class handle SellValue doubling and logging.
    Super::OnMasteryAchieved();

    // On mastery, Legendary and Infinity swords unlock their special move if
    // it was not already flagged in the data asset.
    if (Rarity == EEquipmentRarity::Legendary || Rarity == EEquipmentRarity::Infinity)
    {
        bHasSpecialMove = true;
    }

    UE_LOG(LogTemp, Log,
        TEXT("UIB4Sword::OnMasteryAchieved — '%s' mastered. CritChance now boosted by 0.05."),
        *DisplayName.ToString());

    // Small crit bonus for mastering a sword (cap at 0.25).
    CritChance = FMath::Min(CritChance + 0.05f, 0.25f);
}

// ----------------------------------------------------------------------------
// Private
// ----------------------------------------------------------------------------

void UIB4Sword::InitDefaultDirectionalDamage()
{
    DamageByDirection.Reset();
    DamageByDirection.Add(EAttackDirection::Overhead, 1.2f);
    DamageByDirection.Add(EAttackDirection::Left,     1.0f);
    DamageByDirection.Add(EAttackDirection::Right,    1.0f);
}

float UIB4Sword::GetDefaultCritChanceForRarity(EEquipmentRarity InRarity)
{
    switch (InRarity)
    {
        case EEquipmentRarity::Common:    return 0.05f;
        case EEquipmentRarity::Uncommon:  return 0.08f;
        case EEquipmentRarity::Rare:      return 0.10f;
        case EEquipmentRarity::Epic:      return 0.15f;
        case EEquipmentRarity::Legendary: return 0.20f;
        case EEquipmentRarity::Infinity:  return 0.25f;
        default:                          return 0.05f;
    }
}
