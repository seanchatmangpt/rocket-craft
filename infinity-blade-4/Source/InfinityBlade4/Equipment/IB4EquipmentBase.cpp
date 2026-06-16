// Copyright 2024 Infinity Blade 4 Project. All Rights Reserved.

#include "IB4EquipmentBase.h"

// ============================================================================
// UIB4EquipmentBase
// ============================================================================

UIB4EquipmentBase::UIB4EquipmentBase()
    : ItemID(NAME_None)
    , Slot(EEquipmentSlot::Weapon)
    , Rarity(EEquipmentRarity::Common)
    , ItemIcon(nullptr)
    , MasteryXP(0)
    , MasteryXPRequired(500)   // Default to Common threshold; overridden by InitGemSlots/SetRarity
    , bIsMastered(false)
    , BloodlineMasteryMultiplier(0)
    , PurchaseCost(100)
    , SellValue(25)            // PurchaseCost / 4
{
    // Gem slots are not pre-populated here because Rarity may still be
    // configured in the editor after construction. Call InitGemSlots() once
    // Rarity is finalised.
}

// ----------------------------------------------------------------------------

void UIB4EquipmentBase::AddMasteryXP(int32 XP)
{
    if (XP <= 0)
    {
        return;
    }

    if (bIsMastered)
    {
        // Already mastered — retain XP for display purposes but do nothing more.
        MasteryXP += XP;
        return;
    }

    MasteryXP += XP;

    if (MasteryXP >= MasteryXPRequired)
    {
        // Mastery achieved. Overflow is retained (MasteryXP stays above threshold).
        OnMasteryAchieved();
    }
}

// ----------------------------------------------------------------------------

bool UIB4EquipmentBase::SocketGem(int32 SlotIndex, EGemType GemType, int32 BonusValue)
{
    // Validate index.
    if (!GemSlots.IsValidIndex(SlotIndex))
    {
        UE_LOG(LogTemp, Warning,
            TEXT("UIB4EquipmentBase::SocketGem — SlotIndex %d is out of range for item '%s' (has %d sockets)."),
            SlotIndex, *ItemID.ToString(), GemSlots.Num());
        return false;
    }

    // Reject None gem type — callers should unequip via a dedicated function.
    if (GemType == EGemType::None)
    {
        UE_LOG(LogTemp, Warning,
            TEXT("UIB4EquipmentBase::SocketGem — Cannot socket EGemType::None. Use an unsocket function instead."));
        return false;
    }

    // Reject non-positive bonus values.
    if (BonusValue <= 0)
    {
        UE_LOG(LogTemp, Warning,
            TEXT("UIB4EquipmentBase::SocketGem — BonusValue must be > 0 (got %d)."), BonusValue);
        return false;
    }

    // Reject if slot is already occupied.
    if (GemSlots[SlotIndex].SocketedGem != EGemType::None)
    {
        UE_LOG(LogTemp, Warning,
            TEXT("UIB4EquipmentBase::SocketGem — Slot %d on item '%s' is already occupied."),
            SlotIndex, *ItemID.ToString());
        return false;
    }

    GemSlots[SlotIndex].SocketedGem = GemType;
    GemSlots[SlotIndex].BonusValue  = BonusValue;
    return true;
}

// ----------------------------------------------------------------------------

FEquipmentStats UIB4EquipmentBase::GetTotalStats() const
{
    FEquipmentStats Total = BaseStats;

    for (const FGemSocket& Socket : GemSlots)
    {
        if (Socket.SocketedGem == EGemType::None || Socket.BonusValue <= 0)
        {
            continue;
        }

        switch (Socket.SocketedGem)
        {
            case EGemType::Fire:
            case EGemType::Lightning:
            case EGemType::Dark:
                // Offensive magic gems boost magic damage.
                Total.MagicBonus += Socket.BonusValue;
                break;

            case EGemType::Ice:
                // Ice gems add defensive bonus (slow and chill effects reduce
                // incoming damage throughput).
                Total.DefenseBonus += Socket.BonusValue;
                break;

            case EGemType::Light:
                // Light / holy gems sharpen the blade for physical bonus.
                Total.AttackBonus += Socket.BonusValue;
                break;

            default:
                break;
        }
    }

    return Total;
}

// ----------------------------------------------------------------------------

void UIB4EquipmentBase::ApplyBloodlineMultiplier(int32 BloodlineNumber)
{
    if (BloodlineNumber <= 0)
    {
        // First playthrough — no scaling.
        return;
    }

    BloodlineMasteryMultiplier = BloodlineNumber;

    // Scale: base * 2^BloodlineNumber
    // Use the original rarity-based value as the baseline so repeated calls
    // don't compound unintentionally.
    const int32 BaseThreshold = GetBaseMasteryXPForRarity(Rarity);

    int32 Scaled = BaseThreshold;
    for (int32 i = 0; i < BloodlineNumber; ++i)
    {
        Scaled *= 2;
    }
    MasteryXPRequired = Scaled;
}

// ----------------------------------------------------------------------------

void UIB4EquipmentBase::InitGemSlots()
{
    const int32 SocketCount = GetGemSlotCountForRarity(Rarity);
    GemSlots.Reset();
    GemSlots.SetNum(SocketCount);   // FGemSocket default-ctor sets EGemType::None

    // Also sync MasteryXPRequired to the rarity default (ignoring bloodline here;
    // ApplyBloodlineMultiplier will be called separately on NG+ entry).
    MasteryXPRequired = GetBaseMasteryXPForRarity(Rarity);

    // Sync SellValue in case PurchaseCost was set before this call.
    SellValue = bIsMastered ? (PurchaseCost / 2) : (PurchaseCost / 4);
}

// ----------------------------------------------------------------------------
// Protected
// ----------------------------------------------------------------------------

void UIB4EquipmentBase::OnMasteryAchieved()
{
    bIsMastered = true;
    SellValue   = PurchaseCost / 2;

    UE_LOG(LogTemp, Log,
        TEXT("UIB4EquipmentBase::OnMasteryAchieved — '%s' reached mastery! Sell value is now %d gold."),
        *DisplayName.ToString(), SellValue);
}

// ----------------------------------------------------------------------------
// Static helpers
// ----------------------------------------------------------------------------

int32 UIB4EquipmentBase::GetBaseMasteryXPForRarity(EEquipmentRarity InRarity)
{
    switch (InRarity)
    {
        case EEquipmentRarity::Common:    return 500;
        case EEquipmentRarity::Uncommon:  return 1000;
        case EEquipmentRarity::Rare:      return 2000;
        case EEquipmentRarity::Epic:      return 5000;
        case EEquipmentRarity::Legendary: return 10000;
        case EEquipmentRarity::Infinity:  return 25000;
        default:                          return 500;
    }
}

int32 UIB4EquipmentBase::GetGemSlotCountForRarity(EEquipmentRarity InRarity)
{
    switch (InRarity)
    {
        case EEquipmentRarity::Common:    return 0;
        case EEquipmentRarity::Uncommon:  return 1;
        case EEquipmentRarity::Rare:      return 1;
        case EEquipmentRarity::Epic:      return 2;
        case EEquipmentRarity::Legendary: return 2;
        case EEquipmentRarity::Infinity:  return 3;
        default:                          return 0;
    }
}
