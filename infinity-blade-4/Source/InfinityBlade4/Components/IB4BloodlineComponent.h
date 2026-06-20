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
 */
UCLASS(ClassGroup = "InfinityBlade4", meta = (BlueprintSpawnableComponent))
class INFINITYBLADE4_API UIB4BloodlineComponent : public UActorComponent
{
    GENERATED_BODY()

public:

    UIB4BloodlineComponent();

    /** Returns the current bloodline rebirth cycle count. */
    UFUNCTION(BlueprintCallable, Category = "Bloodline")
    int32 GetBloodlineNumber() const { return BloodlineNumber; }

    /** Returns the list of unlocked perks. */
    UFUNCTION(BlueprintCallable, Category = "Bloodline")
    const TArray<FName>& GetUnlockedPerks() const { return UnlockedPerks; }

    /** Triggers rebirth: increments bloodline cycle, resets player level/XP, strips gear, and updates stats. */
    UFUNCTION(BlueprintCallable, Category = "Bloodline")
    void InitiateRebirth();

    /** Attempts to unlock a perk node by checking prerequisite satisfaction. */
    UFUNCTION(BlueprintCallable, Category = "Bloodline")
    bool UnlockLineageNode(FName NodeID);

    /** Computes lineage stats and updates player character's BloodlineStats. */
    UFUNCTION(BlueprintCallable, Category = "Bloodline")
    FBloodlineStats GetLineageStats();

protected:

    /** Current bloodline rebirth cycle number. */
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Bloodline")
    int32 BloodlineNumber;

    /** Array of unlocked perk IDs. */
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Bloodline")
    TArray<FName> UnlockedPerks;

    /** Current accumulated stats from lineage tree. */
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Bloodline")
    FBloodlineStats CurrentStats;
};
