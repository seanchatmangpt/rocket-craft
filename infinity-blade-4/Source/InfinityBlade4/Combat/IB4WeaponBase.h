// Copyright 2024 Infinity Blade 4. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "GameFramework/Actor.h"
#include "IB4WeaponBase.generated.h"

class AIB4Character;
class UCapsuleComponent;
class USkeletalMeshComponent;
class UParticleSystem;
class USoundBase;

/** Weapon category — drives which animation blueprint set is selected */
UENUM(BlueprintType)
enum class EWeaponType : uint8
{
    Sword       UMETA(DisplayName = "Sword"),
    HeavyBlade  UMETA(DisplayName = "Heavy Blade"),
    Fist        UMETA(DisplayName = "Fist"),
    Staff       UMETA(DisplayName = "Staff")
};

/** Gem socket bonus contributed to a weapon stat */
USTRUCT(BlueprintType)
struct FGemBonus
{
    GENERATED_BODY()

    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Gem")
    float AttackBonus = 0.f;

    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Gem")
    float CritBonus = 0.f;

    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Gem")
    float MagicBonus = 0.f;
};

/** Intrinsic stats of a weapon before gem bonuses */
USTRUCT(BlueprintType)
struct FWeaponStats
{
    GENERATED_BODY()

    /** Base physical damage per hit */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Stats")
    float AttackDamage = 50.f;

    /** Probability [0..1] that a hit becomes a critical */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Stats")
    float CritChance = 0.1f;

    /** Base magic damage contributed to spells/enchantments */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Stats")
    float MagicDamage = 0.f;

    /** Number of gem slots available on this weapon */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Stats")
    int32 GemSlots = 0;
};

UCLASS(Abstract, Blueprintable)
class INFINITYBLADE4_API AIB4WeaponBase : public AActor
{
    GENERATED_BODY()

public:
    AIB4WeaponBase();

protected:
    virtual void BeginPlay() override;

    //-----------------------------------------------------------------------
    // Components
    //-----------------------------------------------------------------------

    /** Visible weapon mesh (1st/3rd person) */
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Weapon|Mesh")
    USkeletalMeshComponent* WeaponMesh;

    /**
     * Capsule used for melee hit detection.
     * Enabled only during the active attack window via anim notifies.
     */
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Weapon|Collision")
    UCapsuleComponent* HitCollision;

    //-----------------------------------------------------------------------
    // Stats & Gems
    //-----------------------------------------------------------------------

    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Weapon|Stats")
    FWeaponStats BaseStats;

    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Weapon|Gems")
    TArray<FGemBonus> EquippedGems;

    //-----------------------------------------------------------------------
    // Type & Socket
    //-----------------------------------------------------------------------

    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Weapon")
    EWeaponType WeaponType;

    /** Name of the character's hand bone/socket to attach to when equipped */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Weapon")
    FName SocketName;

    //-----------------------------------------------------------------------
    // Owner reference
    //-----------------------------------------------------------------------

    UPROPERTY(Transient)
    TWeakObjectPtr<AIB4Character> OwnerCharacter;

    //-----------------------------------------------------------------------
    // FX
    //-----------------------------------------------------------------------

    /** Particle effect spawned at the hit location */
    UPROPERTY(EditDefaultsOnly, Category = "Weapon|FX")
    UParticleSystem* ImpactParticle;

    /** Sound played on a successful hit */
    UPROPERTY(EditDefaultsOnly, Category = "Weapon|FX")
    USoundBase* ImpactSound;

    //-----------------------------------------------------------------------
    // Overlap / hit callbacks
    //-----------------------------------------------------------------------

    UFUNCTION()
    void OnHit(UPrimitiveComponent* HitComp, AActor* OtherActor,
               UPrimitiveComponent* OtherComp, FVector NormalImpulse,
               const FHitResult& Hit);

    UFUNCTION()
    void OnOverlapBegin(UPrimitiveComponent* OverlappedComp, AActor* OtherActor,
                        UPrimitiveComponent* OtherComp, int32 OtherBodyIndex,
                        bool bFromSweep, const FHitResult& SweepResult);

    /** Actors already hit in the current swing (cleared when collision disabled) */
    TArray<TWeakObjectPtr<AActor>> HitActorsThisSwing;

public:
    //-----------------------------------------------------------------------
    // Equip / Unequip
    //-----------------------------------------------------------------------

    UFUNCTION(BlueprintCallable, Category = "Weapon")
    void Equip(AIB4Character* NewOwner);

    UFUNCTION(BlueprintCallable, Category = "Weapon")
    void Unequip();

    //-----------------------------------------------------------------------
    // Collision control (called from CombatComponent via anim notifies)
    //-----------------------------------------------------------------------

    UFUNCTION(BlueprintCallable, Category = "Weapon")
    void EnableHitCollision();

    UFUNCTION(BlueprintCallable, Category = "Weapon")
    void DisableHitCollision();

    //-----------------------------------------------------------------------
    // Damage calculation
    //-----------------------------------------------------------------------

    /**
     * Returns AttackDamage + gem bonuses.
     * Critical roll is evaluated inside OnHit — this returns the raw base value.
     */
    UFUNCTION(BlueprintCallable, Category = "Weapon|Stats")
    float GetAttackDamage() const;

    UFUNCTION(BlueprintCallable, Category = "Weapon|Stats")
    float GetMagicDamage() const;

    UFUNCTION(BlueprintCallable, Category = "Weapon|Stats")
    float GetCritChance() const;

    //-----------------------------------------------------------------------
    // Accessors
    //-----------------------------------------------------------------------

    FORCEINLINE EWeaponType GetWeaponType() const { return WeaponType; }
    FORCEINLINE FName GetSocketName() const { return SocketName; }
    FORCEINLINE USkeletalMeshComponent* GetWeaponMesh() const { return WeaponMesh; }
};
