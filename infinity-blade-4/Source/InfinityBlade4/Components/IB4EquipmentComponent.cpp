#include "Components/IB4EquipmentComponent.h"
#include "Equipment/IB4EquipmentBase.h"

UIB4EquipmentComponent::UIB4EquipmentComponent()
{
    PrimaryComponentTick.bCanEverTick = false;
}

bool UIB4EquipmentComponent::EquipItem(EEquipmentSlot Slot, UIB4EquipmentBase* ItemData)
{
    if (!ItemData)
    {
        return false;
    }

    if (ItemData->Slot != Slot)
    {
        UE_LOG(LogTemp, Warning, TEXT("UIB4EquipmentComponent::EquipItem — Slot mismatch. Item is for slot %d, trying to equip to %d."),
            (int32)ItemData->Slot, (int32)Slot);
        return false;
    }

    UnequipItem(Slot);
    EquippedItems.Add(Slot, ItemData);
    return true;
}

void UIB4EquipmentComponent::UnequipItem(EEquipmentSlot Slot)
{
    EquippedItems.Remove(Slot);
}

FBloodlineStats UIB4EquipmentComponent::GetCombinedStats() const
{
    FBloodlineStats CombinedStats;
    CombinedStats.AttackBonus = 0.f;
    CombinedStats.DefenseBonus = 0.f;
    CombinedStats.MagicBonus = 0.f;
    CombinedStats.XPMultiplier = 1.f;

    for (const auto& Pair : EquippedItems)
    {
        if (Pair.Value)
        {
            FEquipmentStats ItemStats = Pair.Value->GetTotalStats();
            CombinedStats.AttackBonus += static_cast<float>(ItemStats.AttackBonus);
            CombinedStats.DefenseBonus += static_cast<float>(ItemStats.DefenseBonus);
            CombinedStats.MagicBonus += static_cast<float>(ItemStats.MagicBonus);
            CombinedStats.XPMultiplier *= ItemStats.XPGainMultiplier;
        }
    }

    return CombinedStats;
}
