#pragma once

#include "CoreMinimal.h"
#include "MechDensity.generated.h"

USTRUCT(BlueprintType)
struct FMechMeshDensity
{
    GENERATED_BODY()

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="Mech Density")
    int32 Subframes;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="Mech Density")
    int32 FeatherBlades;
    
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="Mech Density")
    int32 ArmorPanels;
};
