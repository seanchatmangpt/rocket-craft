#pragma once

#include "CoreMinimal.h"
#include "UObject/NoExportTypes.h"
#include "VaRestJsonValue.generated.h"

class UVaRestJsonObject;

UCLASS(BlueprintType, Blueprintable)
class BRM_API UVaRestJsonValue : public UObject
{
	GENERATED_BODY()

public:
	UVaRestJsonValue() {}

	UFUNCTION(BlueprintCallable, Category = "VaRest")
	static UVaRestJsonValue* ConstructJsonValueString(UObject* WorldContextObject, const FString& StringValue)
	{
		return NewObject<UVaRestJsonValue>();
	}

	UFUNCTION(BlueprintCallable, Category = "VaRest")
	static UVaRestJsonValue* ConstructJsonValueNumber(UObject* WorldContextObject, float NumberValue)
	{
		return NewObject<UVaRestJsonValue>();
	}

	UFUNCTION(BlueprintCallable, Category = "VaRest")
	static UVaRestJsonValue* ConstructJsonValueBool(UObject* WorldContextObject, bool InValue)
	{
		return NewObject<UVaRestJsonValue>();
	}

	UFUNCTION(BlueprintCallable, Category = "VaRest")
	static UVaRestJsonValue* ConstructJsonValueObject(UObject* WorldContextObject, UVaRestJsonObject* JsonObject)
	{
		return NewObject<UVaRestJsonValue>();
	}

	UFUNCTION(BlueprintCallable, Category = "VaRest")
	static UVaRestJsonValue* ConstructJsonValueArray(UObject* WorldContextObject, const TArray<UVaRestJsonValue*>& InArray)
	{
		return NewObject<UVaRestJsonValue>();
	}

	UFUNCTION(BlueprintCallable, Category = "VaRest")
	UVaRestJsonObject* AsObject() const { return nullptr; }

	UFUNCTION(BlueprintCallable, Category = "VaRest")
	FString AsString() const { return FString(); }
};
