// Copyright 2024 Infinity Blade 4 Project. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "Components/ActorComponent.h"
#include "Core/IB4Types.h"
#include "IB4BloodlineComponent.generated.h"

/**
 * UIB4BloodlineComponent owns the persistent bloodline progression data and
 * exposes hooks for rebirth (death-reset with carried stats) and lineage tree
 * unlocks.
 *
 * Full implementation is deferred to a follow-up changelist; this header
 * provides the type signature required by AIB4PlayerCharacter.
 */
UCLASS(ClassGroup = "InfinityBlade4", meta = (BlueprintSpawnableComponent))
class INFINITYBLADE4_API UIB4BloodlineComponent : public UActorComponent
{
    GENERATED_BODY()

public:

    UIB4BloodlineComponent();

    // TODO: InitiateRebirth()   — strip gear, keep bloodline bonuses
    // TODO: UnlockLineageNode(FName NodeID)
    // TODO: GetLineageStats() -> FBloodlineStats
};
