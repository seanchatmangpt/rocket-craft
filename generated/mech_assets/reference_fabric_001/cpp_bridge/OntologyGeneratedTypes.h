#pragma once

#include "CoreMinimal.h"
#include "Engine/DataTable.h"
#include "OntologyGeneratedTypes.generated.h"

USTRUCT(BlueprintType)
struct FArmorBaseline_Tiger_I_V0 : public FTableRowBase
{
	GENERATED_BODY()

	// No SHACL properties defined
};

USTRUCT(BlueprintType)
struct FArmorZoneThicknessLaw : public FTableRowBase
{
	GENERATED_BODY()

	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="Ontology")
	float frontToSideRatio;

	// Constraints: Min=1.2, Max=1.5
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="Ontology")
	float frontToTopRatio;

	// Constraints: Min=3.0, Max=4.5
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="Ontology")
	float mantletToFrontRatio;

	// Constraints: Min=1.0, Max=2.0
};

USTRUCT(BlueprintType)
struct Fforeground_component_count : public FTableRowBase
{
	GENERATED_BODY()

	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="Ontology")
	int32 foregroundComponentCount;

	// Constraints: Min=1, Max=5
};

USTRUCT(BlueprintType)
struct Fcore_compactness_delta : public FTableRowBase
{
	GENERATED_BODY()

	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="Ontology")
	float coreCompactnessDelta;

	// Constraints: Min=0.0, Max=0.15
};

USTRUCT(BlueprintType)
struct Fblade_length_angle_delta : public FTableRowBase
{
	GENERATED_BODY()

	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="Ontology")
	float bladeLengthAngleDelta;

	// Constraints: Min=0.0, Max=15.0
};

