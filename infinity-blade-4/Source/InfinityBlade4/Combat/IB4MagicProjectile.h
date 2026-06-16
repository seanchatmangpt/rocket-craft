// Copyright 2024 Infinity Blade 4. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "GameFramework/Actor.h"
#include "IB4MagicProjectile.generated.h"

class UProjectileMovementComponent;
class USphereComponent;
class UParticleSystemComponent;
class USoundBase;
class UParticleSystem;

/** Status effect that can be applied by a magic projectile on hit */
UENUM(BlueprintType)
enum class EStatusEffect : uint8
{
    None    UMETA(DisplayName = "None"),
    Burn    UMETA(DisplayName = "Burn"),    // Fire: damage over time
    Freeze  UMETA(DisplayName = "Freeze"),  // Ice:  slow/root
    Stun    UMETA(DisplayName = "Stun"),    // Lightning: brief stun
    Dark    UMETA(DisplayName = "Dark")     // Dark: damage + visibility reduction
};

/** Magic type mirroring EMagicType in IB4CombatComponent */
UENUM(BlueprintType)
enum class EMagicProjectileType : uint8
{
    Fire        UMETA(DisplayName = "Fire"),
    Ice         UMETA(DisplayName = "Ice"),
    Lightning   UMETA(DisplayName = "Lightning"),
    Dark        UMETA(DisplayName = "Dark")
};

UCLASS(Blueprintable)
class INFINITYBLADE4_API AIB4MagicProjectile : public AActor
{
    GENERATED_BODY()

public:
    AIB4MagicProjectile();

protected:
    virtual void BeginPlay() override;

    //-----------------------------------------------------------------------
    // Components
    //-----------------------------------------------------------------------

    /** Sphere collision — triggers OnHit */
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Projectile")
    USphereComponent* CollisionSphere;

    /** Movement component — drives flight, homing, gravity */
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Projectile")
    UProjectileMovementComponent* ProjectileMovement;

    /** Looping particle trail attached to the projectile */
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Projectile|FX")
    UParticleSystemComponent* TrailParticle;

    //-----------------------------------------------------------------------
    // Magic Type & Stats
    //-----------------------------------------------------------------------

    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Magic")
    EMagicProjectileType MagicType;

    /** Base damage on direct impact */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Magic")
    float MagicDamage;

    /** Status effect applied to targets hit by this projectile */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Magic")
    EStatusEffect StatusEffect;

    /** Duration (seconds) of the status effect */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Magic")
    float StatusEffectDuration;

    /** Damage-per-second for DoT status effects (Burn) */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Magic")
    float DamagePerSecond;

    //-----------------------------------------------------------------------
    // Explosion / Area of Effect
    //-----------------------------------------------------------------------

    /** Whether this projectile detonates with area damage on impact */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Magic|Explosion")
    bool bExplodeOnHit;

    /** Radius of the area damage explosion */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Magic|Explosion",
              meta = (ClampMin = "0.0"))
    float ExplosionRadius;

    /** AoE damage dealt by the explosion (may differ from direct impact damage) */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Magic|Explosion")
    float ExplosionDamage;

    /** Particle effect spawned at explosion origin */
    UPROPERTY(EditDefaultsOnly, Category = "Magic|Explosion")
    UParticleSystem* ExplosionParticle;

    /** Sound played at the explosion origin */
    UPROPERTY(EditDefaultsOnly, Category = "Magic|Explosion")
    USoundBase* ExplosionSound;

    //-----------------------------------------------------------------------
    // Homing (Ice type)
    //-----------------------------------------------------------------------

    /**
     * Whether this projectile homes toward a target.
     * Typically enabled for Ice-type projectiles.
     */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Magic|Homing")
    bool bHoming;

    /** Acceleration magnitude applied when homing (cm/s²) */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Magic|Homing",
              meta = (ClampMin = "0.0"))
    float HomingAcceleration;

    /** Soft reference to the homing target */
    UPROPERTY(BlueprintReadWrite, Category = "Magic|Homing")
    TWeakObjectPtr<AActor> HomingTarget;

    //-----------------------------------------------------------------------
    // Type-Specific FX Presets
    //-----------------------------------------------------------------------

    UPROPERTY(EditDefaultsOnly, Category = "Magic|FX")
    UParticleSystem* FireTrailFX;

    UPROPERTY(EditDefaultsOnly, Category = "Magic|FX")
    UParticleSystem* IceTrailFX;

    UPROPERTY(EditDefaultsOnly, Category = "Magic|FX")
    UParticleSystem* LightningTrailFX;

    UPROPERTY(EditDefaultsOnly, Category = "Magic|FX")
    UParticleSystem* DarkTrailFX;

    //-----------------------------------------------------------------------
    // Collision Callback
    //-----------------------------------------------------------------------

    UFUNCTION()
    void OnProjectileHit(UPrimitiveComponent* HitComp, AActor* OtherActor,
                         UPrimitiveComponent* OtherComp, FVector NormalImpulse,
                         const FHitResult& Hit);

    /** Internal helpers */
    void ApplyExplosionDamage(const FVector& ExplosionOrigin);
    void ApplyStatusEffect(AActor* Target);
    void SpawnHitFX(const FVector& Location, const FVector& Normal);

public:
    //-----------------------------------------------------------------------
    // Public API (called by CombatComponent after spawn)
    //-----------------------------------------------------------------------

    /**
     * Set the magic type — overrides defaults set in CDO.
     * Also configures homing, status effect, and FX to match the type.
     */
    UFUNCTION(BlueprintCallable, Category = "Magic")
    void SetMagicType(EMagicProjectileType NewType);

    /** Assign a homing target (usually the locked-on enemy) */
    UFUNCTION(BlueprintCallable, Category = "Magic|Homing")
    void SetHomingTarget(AActor* Target);

    FORCEINLINE EMagicProjectileType GetMagicType() const { return MagicType; }
};
