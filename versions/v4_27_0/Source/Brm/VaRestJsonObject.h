#pragma once

#include "CoreMinimal.h"
#include "UObject/NoExportTypes.h"
#include "VaRestJsonObject.generated.h"

class UVaRestJsonValue;

UENUM(BlueprintType)
enum class EVaRestRequestContentType : uint8
{
	x_www_form_urlencoded UMETA(DisplayName = "application/x-www-form-urlencoded"),
	json UMETA(DisplayName = "application/json"),
	binary UMETA(DisplayName = "binary"),
	custom UMETA(DisplayName = "custom")
};

UENUM(BlueprintType)
enum class EVaRestRequestVerb : uint8
{
	GET,
	POST,
	PUT,
	DEL UMETA(DisplayName = "DELETE")
};

UENUM(BlueprintType)
enum class EVaRest_JsonType : uint8
{
	None,
	Null,
	String,
	Number,
	Boolean,
	Array,
	Object
};

UCLASS(BlueprintType, Blueprintable)
class BRM_API UVaRestJsonObject : public UObject
{
	GENERATED_BODY()

public:
	UVaRestJsonObject() {}

	UFUNCTION(BlueprintCallable, Category = "VaRest")
	static UVaRestJsonObject* ConstructVaRestJsonObject(UObject* WorldContextObject)
	{
		return NewObject<UVaRestJsonObject>();
	}

	UFUNCTION(BlueprintCallable, Category = "VaRest")
	void SetStringField(const FString& FieldName, const FString& StringValue) {}

	UFUNCTION(BlueprintCallable, Category = "VaRest")
	FString GetStringField(const FString& FieldName) const { return FString(); }

	UFUNCTION(BlueprintCallable, Category = "VaRest")
	void SetNumberField(const FString& FieldName, float NumberValue) {}

	UFUNCTION(BlueprintCallable, Category = "VaRest")
	float GetNumberField(const FString& FieldName) const { return 0.0f; }

	UFUNCTION(BlueprintCallable, Category = "VaRest")
	void SetBoolField(const FString& FieldName, bool InBool) {}

	UFUNCTION(BlueprintCallable, Category = "VaRest")
	bool GetBoolField(const FString& FieldName) const { return false; }

	UFUNCTION(BlueprintCallable, Category = "VaRest")
	void SetObjectField(const FString& FieldName, UVaRestJsonObject* JsonObject) {}

	UFUNCTION(BlueprintCallable, Category = "VaRest")
	UVaRestJsonObject* GetObjectField(const FString& FieldName) const { return nullptr; }

	UFUNCTION(BlueprintCallable, Category = "VaRest")
	void SetArrayField(const FString& FieldName, const TArray<UVaRestJsonValue*>& InArray) {}

	UFUNCTION(BlueprintCallable, Category = "VaRest")
	TArray<UVaRestJsonValue*> GetArrayField(const FString& FieldName) const { return TArray<UVaRestJsonValue*>(); }

	UFUNCTION(BlueprintCallable, Category = "VaRest")
	bool HasField(const FString& FieldName) const { return false; }

	UFUNCTION(BlueprintCallable, Category = "VaRest")
	void SetField(const FString& FieldName, UVaRestJsonValue* FieldValue) {}

	UFUNCTION(BlueprintCallable, Category = "VaRest")
	UVaRestJsonValue* GetField(const FString& FieldName) const { return nullptr; }

	UFUNCTION(BlueprintCallable, Category = "VaRest")
	int32 GetIntegerField(const FString& FieldName) const { return 0; }

	UFUNCTION(BlueprintCallable, Category = "VaRest")
	bool TryGetField(const FString& FieldName, UVaRestJsonValue*& OutValue) const { return false; }
};
