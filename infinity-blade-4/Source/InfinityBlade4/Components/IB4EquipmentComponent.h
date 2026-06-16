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
UCLASS(ClassGroup = "InfinityBlade4", meta = (BlueprintSpawnableComponent))
class INFINITYBLADE4_API UIB4EquipmentComponent : public UActorComponent
{
    GENERATED_BODY()

public:

    UIB4EquipmentComponent();

    // TODO: EquipItem(EEquipmentSlot Slot, UIB4ItemData* ItemData)
    // TODO: UnequipItem(EEquipmentSlot Slot)
    // TODO: GetCombinedStats() -> FBloodlineStats
};
