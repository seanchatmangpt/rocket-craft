// Copyright 2024 Infinity Blade 4 Project. All Rights Reserved.

#include "Components/IB4BloodlineComponent.h"
#include "Components/IB4EquipmentComponent.h"
#include "Progression/IB4BloodlinePerkTree.h"
#include "Characters/IB4Character.h"
#include "GameFramework/Actor.h"

UIB4BloodlineComponent::UIB4BloodlineComponent()
    : BloodlineNumber(0)
{
    PrimaryComponentTick.bCanEverTick = false;
    CurrentStats.AttackBonus = 0.f;
    CurrentStats.DefenseBonus = 0.f;
    CurrentStats.MagicBonus = 0.f;
    CurrentStats.XPMultiplier = 1.f;
}

void UIB4BloodlineComponent::InitiateRebirth()
{
    BloodlineNumber++;

    AIB4Character* Char = Cast<AIB4Character>(GetOwner());
    if (Char)
    {
        Char->BloodlineLevel = 0;
        Char->CurrentXP = 0.f;

        if (UIB4EquipmentComponent* EquipComp = Char->FindComponentByClass<UIB4EquipmentComponent>())
        {
            EquipComp->UnequipItem(EEquipmentSlot::Weapon);
            EquipComp->UnequipItem(EEquipmentSlot::Shield);
            EquipComp->UnequipItem(EEquipmentSlot::Helmet);
            EquipComp->UnequipItem(EEquipmentSlot::Armor);
            EquipComp->UnequipItem(EEquipmentSlot::Ring);
        }
    }

    GetLineageStats();

    UE_LOG(LogTemp, Log, TEXT("UIB4BloodlineComponent::InitiateRebirth — Rebirth initiated. Gear stripped, bloodline bonuses preserved."));
}

bool UIB4BloodlineComponent::UnlockLineageNode(FName NodeID)
{
    if (NodeID.IsNone())
    {
        return false;
    }

    if (UnlockedPerks.Contains(NodeID))
    {
        return false;
    }

    // Check if the node exists in the perk tree
    UIB4BloodlinePerkTree* PerkTree = NewObject<UIB4BloodlinePerkTree>(this);
    if (PerkTree)
    {
        FBloodlinePerk Perk;
        if (!PerkTree->GetPerkByID(NodeID, Perk))
        {
            return false;
        }

        // Check prerequisites
        if (!PerkTree->ArePrerequisitesMet(NodeID, UnlockedPerks))
        {
            return false;
        }
    }

    UnlockedPerks.Add(NodeID);
    GetLineageStats(); // Recalculates stats and updates character
    return true;
}

FBloodlineStats UIB4BloodlineComponent::GetLineageStats()
{
    FBloodlineStats NewStats;
    NewStats.AttackBonus = 0.f;
    NewStats.DefenseBonus = 0.f;
    NewStats.MagicBonus = 0.f;
    NewStats.XPMultiplier = 1.f;

    UIB4BloodlinePerkTree* PerkTree = NewObject<UIB4BloodlinePerkTree>(this);
    if (PerkTree)
    {
        auto ProcessEffect = [&](EPerkEffectType EffectType, float EffectValue)
        {
            switch (EffectType)
            {
                case EPerkEffectType::AttackBonus:
                    NewStats.AttackBonus += EffectValue;
                    break;
                case EPerkEffectType::DefenseBonus:
                    NewStats.DefenseBonus += EffectValue;
                    break;
                case EPerkEffectType::MagicBonus:
                    NewStats.MagicBonus += EffectValue;
                    break;
                case EPerkEffectType::XPGain:
                    NewStats.XPMultiplier += EffectValue;
                    break;
                default:
                    break;
            }
        };

        for (const FName& NodeID : UnlockedPerks)
        {
            FBloodlinePerk Perk;
            if (PerkTree->GetPerkByID(NodeID, Perk))
            {
                ProcessEffect(Perk.EffectType, Perk.EffectValue);
                if (Perk.bHasSecondaryEffect)
                {
                    ProcessEffect(Perk.SecondaryEffectType, Perk.SecondaryEffectValue);
                }
            }
        }
    }

    CurrentStats = NewStats;

    AIB4Character* OwnerCharacter = Cast<AIB4Character>(GetOwner());
    if (OwnerCharacter)
    {
        OwnerCharacter->BloodlineStats = CurrentStats;
    }

    return CurrentStats;
}
