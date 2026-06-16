// Copyright 2024 Infinity Blade 4 Project. All Rights Reserved.

#include "IB4Ring.h"

// ============================================================================
// UIB4Ring
// ============================================================================

UIB4Ring::UIB4Ring()
    : PrimaryEffect(ERingEffect::None)
    , EffectValue(0.1f)
    , SecondaryEffect(ERingEffect::None)
    , SecondaryEffectValue(0.0f)
{
    Slot = EEquipmentSlot::Ring;
}

// ----------------------------------------------------------------------------

bool UIB4Ring::HasSecondaryEffect() const
{
    return SecondaryEffect != ERingEffect::None;
}

// ----------------------------------------------------------------------------

float UIB4Ring::GetEffectMagnitude(ERingEffect Effect) const
{
    if (Effect == ERingEffect::None)
    {
        return 0.0f;
    }

    float Total = 0.0f;

    if (PrimaryEffect == Effect)
    {
        Total += EffectValue;
    }

    if (SecondaryEffect == Effect)
    {
        Total += SecondaryEffectValue;
    }

    return Total;
}

// ----------------------------------------------------------------------------
// Protected
// ----------------------------------------------------------------------------

void UIB4Ring::OnMasteryAchieved()
{
    Super::OnMasteryAchieved();

    // Mastery reward: boost the primary effect value by 10% of its current
    // value (multiplicative), capped at 0.5.
    const float BoostAmount = EffectValue * 0.10f;
    EffectValue = FMath::Min(EffectValue + BoostAmount, 0.5f);

    // Also boost secondary effect if present.
    if (HasSecondaryEffect())
    {
        const float SecBoostAmount = SecondaryEffectValue * 0.10f;
        SecondaryEffectValue = FMath::Min(SecondaryEffectValue + SecBoostAmount, 0.5f);
    }

    UE_LOG(LogTemp, Log,
        TEXT("UIB4Ring::OnMasteryAchieved — '%s' mastered. PrimaryEffectValue=%.3f."),
        *DisplayName.ToString(), EffectValue);
}
