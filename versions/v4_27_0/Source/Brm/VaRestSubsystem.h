#pragma once

#include "CoreMinimal.h"
#include "Subsystems/GameInstanceSubsystem.h"
#include "VaRestJsonObject.h"
#include "VaRestJsonValue.h"
#include "VaRestRequestJSON.h"
#include "VaRestSubsystem.generated.h"

// Blueprint-callable VaRest subsystem. Exposes the VaRest API surface expected
// by blueprints in VehicleAdvBP — these were authored against the VaRest plugin's
// UVaRestSubsystem. Here they delegate to the inline implementations in the
// VaRestJson*/VaRestRequest headers that live in the Brm module.
UCLASS(BlueprintType)
class BRM_API UVaRestSubsystem : public UGameInstanceSubsystem
{
	GENERATED_BODY()

public:
	UVaRestSubsystem() {}

	UFUNCTION(BlueprintCallable, Category = "VaRest|Http", meta = (WorldContext = "WorldContextObject"))
	static UVaRestJsonObject* ConstructVaRestJsonObject(UObject* WorldContextObject)
	{
		return UVaRestJsonObject::ConstructVaRestJsonObject(WorldContextObject);
	}

	UFUNCTION(BlueprintCallable, Category = "VaRest|Json", meta = (WorldContext = "WorldContextObject"))
	static UVaRestJsonValue* ConstructJsonValueString(UObject* WorldContextObject, const FString& StringValue)
	{
		return UVaRestJsonValue::ConstructJsonValueString(WorldContextObject, StringValue);
	}

	UFUNCTION(BlueprintCallable, Category = "VaRest|Json", meta = (WorldContext = "WorldContextObject"))
	static UVaRestJsonValue* ConstructJsonValueNumber(UObject* WorldContextObject, float NumberValue)
	{
		return UVaRestJsonValue::ConstructJsonValueNumber(WorldContextObject, NumberValue);
	}

	UFUNCTION(BlueprintCallable, Category = "VaRest|Json", meta = (WorldContext = "WorldContextObject"))
	static UVaRestJsonValue* ConstructJsonValueBool(UObject* WorldContextObject, bool InValue)
	{
		return UVaRestJsonValue::ConstructJsonValueBool(WorldContextObject, InValue);
	}

	UFUNCTION(BlueprintCallable, Category = "VaRest|Http", meta = (WorldContext = "WorldContextObject"))
	static UVaRestRequestJSON* ConstructVaRestRequestJSON(UObject* WorldContextObject)
	{
		return UVaRestRequestJSON::ConstructVaRestRequestJSON(WorldContextObject);
	}

	UFUNCTION(BlueprintCallable, Category = "VaRest|Http")
	void CallURL(const FString& URL, EVaRestRequestVerb Verb, EVaRestRequestContentType ContentType,
	             UVaRestJsonObject* RequestObject, const FVaRestCallDelegate& Callback)
	{
		// Construct a request and fire it — delegates to VaRestRequestJSON
		UVaRestRequestJSON* Request = NewObject<UVaRestRequestJSON>(this);
		if (Request)
		{
			Request->CallURL(URL, Verb, ContentType, RequestObject, Callback);
		}
	}

	UFUNCTION(BlueprintCallable, Category = "VaRest|Json", meta = (WorldContext = "WorldContextObject"))
	static void SetField(UObject* WorldContextObject, UVaRestJsonObject* JsonObject,
	                     const FString& FieldName, UVaRestJsonValue* JsonValue)
	{
		if (JsonObject && JsonValue)
		{
			JsonObject->SetField(FieldName, JsonValue);
		}
	}
};
