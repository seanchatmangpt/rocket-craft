#pragma once

#include "CoreMinimal.h"
#include "UObject/NoExportTypes.h"
#include "VaRestJsonObject.h"
#include "VaRestRequestJSON.generated.h"

class UVaRestRequestJSON;

DECLARE_DYNAMIC_DELEGATE_OneParam(FVaRestCallDelegate, UVaRestRequestJSON*, Request);
DECLARE_DYNAMIC_MULTICAST_DELEGATE_OneParam(FVaRestCallCompleteDelegate, UVaRestRequestJSON*, Request);

UCLASS(BlueprintType, Blueprintable)
class BRM_API UVaRestRequestJSON : public UObject
{
	GENERATED_BODY()

public:
	UVaRestRequestJSON() {}

	UFUNCTION(BlueprintCallable, Category = "VaRest")
	static UVaRestRequestJSON* ConstructVaRestRequestJSON(UObject* WorldContextObject)
	{
		return NewObject<UVaRestRequestJSON>();
	}

	UPROPERTY(BlueprintAssignable, Category = "VaRest")
	FVaRestCallCompleteDelegate OnRequestComplete;

	UPROPERTY(BlueprintAssignable, Category = "VaRest")
	FVaRestCallCompleteDelegate OnRequestFail;

	UFUNCTION(BlueprintCallable, Category = "VaRest")
	void CallURL(const FString& URL, EVaRestRequestVerb Verb, EVaRestRequestContentType ContentType, UVaRestJsonObject* RequestObject, const FVaRestCallDelegate& Callback) {}

	UFUNCTION(BlueprintCallable, Category = "VaRest")
	UVaRestJsonObject* GetResponseObject() const { return nullptr; }

	UFUNCTION(BlueprintCallable, Category = "VaRest")
	void SetVerb(EVaRestRequestVerb Verb) {}

	UFUNCTION(BlueprintCallable, Category = "VaRest")
	void SetContentType(EVaRestRequestContentType ContentType) {}

	UFUNCTION(BlueprintCallable, Category = "VaRest")
	void SetRequestObject(UVaRestJsonObject* RequestObject) {}

	UFUNCTION(BlueprintCallable, Category = "VaRest")
	void ProcessURL(const FString& URL) {}
};
