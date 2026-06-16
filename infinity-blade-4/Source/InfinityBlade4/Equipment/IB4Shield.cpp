// Copyright 2024 Infinity Blade 4 Project. All Rights Reserved.

#include "IB4Shield.h"

// ============================================================================
// UIB4Shield
// ============================================================================

UIB4Shield::UIB4Shield()
    : BlockArc(120.0f)      // Default: wider than narrow but not full hemisphere
    , ParryBonus(0.0f)
    , ShieldHP(3)
    , RecoveryTime(2.0f)
{
    Slot = EEquipmentSlot::Shield;

    InitDefaultElementalResistances();
}

// ----------------------------------------------------------------------------

float UIB4Shield::GetElementalResistance(EMagicType MagicType) const
{
    const float* ResistPtr = ElementalResistance.Find(MagicType);
    return ResistPtr ? *ResistPtr : 0.0f;
}

// ----------------------------------------------------------------------------

bool UIB4Shield::IsWithinBlockArc(float AttackAngleDegrees) const
{
    // Normalise to [-180, 180] so the half-arc comparison works correctly.
    const float HalfArc       = BlockArc * 0.5f;
    const float AbsAngle      = FMath::Abs(FMath::UnwindDegrees(AttackAngleDegrees));
    return AbsAngle <= HalfArc;
}

// ----------------------------------------------------------------------------
// Protected
// ----------------------------------------------------------------------------

void UIB4Shield::OnMasteryAchieved()
{
    Super::OnMasteryAchieved();

    // Mastery reward: increase block arc by 10 degrees (cap at 180).
    BlockArc = FMath::Min(BlockArc + 10.0f, 180.0f);

    // Mastery reward: add a small parry bonus if not already at cap.
    ParryBonus = FMath::Min(ParryBonus + 0.01f, 0.05f);

    // Mastery reward: one extra shield hit point.
    ShieldHP += 1;

    UE_LOG(LogTemp, Log,
        TEXT("UIB4Shield::OnMasteryAchieved — '%s' mastered. BlockArc=%.1f, ParryBonus=%.3f, ShieldHP=%d."),
        *DisplayName.ToString(), BlockArc, ParryBonus, ShieldHP);
}

// ----------------------------------------------------------------------------
// Private
// ----------------------------------------------------------------------------

void UIB4Shield::InitDefaultElementalResistances()
{
    ElementalResistance.Reset();
    ElementalResistance.Add(EMagicType::Fire,      0.0f);
    ElementalResistance.Add(EMagicType::Lightning, 0.0f);
    ElementalResistance.Add(EMagicType::Ice,       0.0f);
    ElementalResistance.Add(EMagicType::Dark,      0.0f);
    ElementalResistance.Add(EMagicType::Light,     0.0f);
}
