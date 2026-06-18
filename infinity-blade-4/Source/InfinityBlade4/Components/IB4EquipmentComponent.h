// Copyright 2024 Infinity Blade 4 Project. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "Components/ActorComponent.h"
#include "Core/IB4Types.h"
#include "IB4EquipmentComponent.generated.h"

/**
 * UIB4EquipmentComponent manages the five equipment slots (Weapon, Shield,
 * Helmet, Armor, Ring) and the combined stat bonuses they contribute.
 *
 * Full implementation is deferred to a follow-up changelist; this header
 * provides the type signature required by AIB4PlayerCharacter.
 */
class UIB4EquipmentBase;

UCLASS(ClassGroup = "InfinityBlade4", meta = (BlueprintSpawnableComponent))
class INFINITYBLADE4_API UIB4EquipmentComponent : public UActorComponent
{
    GENERATED_BODY()

public:

    UIB4EquipmentComponent();

    /** Equips an item to the specified slot. */
    UFUNCTION(BlueprintCallable, Category = "Equipment")
    bool EquipItem(EEquipmentSlot Slot, UIB4EquipmentBase* ItemData);

    /** Unequips the item from the specified slot. */
    UFUNCTION(BlueprintCallable, Category = "Equipment")
    void UnequipItem(EEquipmentSlot Slot);

    /** Calculates the combined stat bonuses from all equipped items. */
    UFUNCTION(BlueprintCallable, Category = "Equipment")
    FBloodlineStats GetCombinedStats() const;

    /** Returns all equipped items. */
    UFUNCTION(BlueprintCallable, Category = "Equipment")
    const TMap<EEquipmentSlot, UIB4EquipmentBase*>& GetEquippedItems() const { return EquippedItems; }

protected:

    /** Map tracking the item equipped in each slot. */
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Equipment")
    TMap<EEquipmentSlot, UIB4EquipmentBase*> EquippedItems;
};
