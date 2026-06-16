// Copyright 2024 Infinity Blade 4 Project. All Rights Reserved.

#include "IB4Helmet.h"

// ============================================================================
// UIB4Helmet
// ============================================================================

UIB4Helmet::UIB4Helmet()
    : ArmorRating(5)
    , MagicResistance(0.0f)
    , SetBonusTag(NAME_None)
    , bGrantsSetBonus(false)
{
    Slot = EEquipmentSlot::Helmet;
}

// ----------------------------------------------------------------------------

float UIB4Helmet::CalculateDamageReduction(float IncomingDamage, bool bIsMagicDamage) const
{
    if (IncomingDamage <= 0.f)
    {
        return 0.f;
    }

    float ReducedDamage = IncomingDamage;

    if (bIsMagicDamage)
    {
        // Apply magic resistance as a fraction reduction.
        ReducedDamage *= (1.0f - FMath::Clamp(MagicResistance, 0.0f, 0.3f));
    }
    else
    {
        // Apply flat armour rating reduction.
        ReducedDamage -= static_cast<float>(ArmorRating);
    }

    return FMath::Max(ReducedDamage, 0.0f);
}

// ----------------------------------------------------------------------------

void UIB4Helmet::SetSetBonusActive(bool bActive)
{
    if (bGrantsSetBonus == bActive)
    {
        return;
    }

    bGrantsSetBonus = bActive;

    if (bActive && !SetBonusTag.IsNone())
    {
        UE_LOG(LogTemp, Log,
            TEXT("UIB4Helmet::SetSetBonusActive — Set bonus '%s' is now ACTIVE on '%s'."),
            *SetBonusTag.ToString(), *DisplayName.ToString());
    }
    else
    {
        UE_LOG(LogTemp, Log,
            TEXT("UIB4Helmet::SetSetBonusActive — Set bonus '%s' deactivated on '%s'."),
            *SetBonusTag.ToString(), *DisplayName.ToString());
    }
}

// ----------------------------------------------------------------------------
// Protected
// ----------------------------------------------------------------------------

void UIB4Helmet::OnMasteryAchieved()
{
    Super::OnMasteryAchieved();

    // Mastery reward: +5 flat armor rating.
    ArmorRating += 5;

    // Mastery reward: +0.02 magic resistance (cap at 0.3).
    MagicResistance = FMath::Min(MagicResistance + 0.02f, 0.3f);

    UE_LOG(LogTemp, Log,
        TEXT("UIB4Helmet::OnMasteryAchieved — '%s' mastered. ArmorRating=%d, MagicResistance=%.3f."),
        *DisplayName.ToString(), ArmorRating, MagicResistance);
}
