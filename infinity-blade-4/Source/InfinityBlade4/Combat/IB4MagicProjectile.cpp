// Copyright 2024 Infinity Blade 4. All Rights Reserved.

#include "Combat/IB4MagicProjectile.h"
#include "Combat/IB4CombatComponent.h"
#include "Components/SphereComponent.h"
#include "GameFramework/ProjectileMovementComponent.h"
#include "Particles/ParticleSystemComponent.h"
#include "Kismet/GameplayStatics.h"
#include "Engine/World.h"

//-----------------------------------------------------------------------------
// Construction
//-----------------------------------------------------------------------------

AIB4MagicProjectile::AIB4MagicProjectile()
{
    PrimaryActorTick.bCanEverTick = false;

    // --- Collision ---
    CollisionSphere = CreateDefaultSubobject<USphereComponent>(TEXT("CollisionSphere"));
    CollisionSphere->InitSphereRadius(16.f);
    CollisionSphere->SetCollisionProfileName(TEXT("Projectile"));
    CollisionSphere->SetNotifyRigidBodyCollision(true);
    SetRootComponent(CollisionSphere);

    // --- Projectile Movement ---
    ProjectileMovement = CreateDefaultSubobject<UProjectileMovementComponent>(TEXT("ProjectileMovement"));
    ProjectileMovement->UpdatedComponent        = CollisionSphere;
    ProjectileMovement->InitialSpeed            = 2000.f;
    ProjectileMovement->MaxSpeed                = 4000.f;
    ProjectileMovement->bRotationFollowsVelocity = true;
    ProjectileMovement->bShouldBounce           = false;
    ProjectileMovement->ProjectileGravityScale  = 0.f; // Magic travels flat by default

    // --- Trail Particle ---
    TrailParticle = CreateDefaultSubobject<UParticleSystemComponent>(TEXT("TrailParticle"));
    TrailParticle->SetupAttachment(RootComponent);
    TrailParticle->bAutoActivate = true;

    // --- Defaults ---
    MagicType             = EMagicProjectileType::Fire;
    MagicDamage           = 40.f;
    StatusEffect          = EStatusEffect::Burn;
    StatusEffectDuration  = 3.f;
    DamagePerSecond       = 5.f;
    bExplodeOnHit         = true;
    ExplosionRadius       = 150.f;
    ExplosionDamage       = 30.f;
    bHoming               = false;
    HomingAcceleration    = 1500.f;

    // Projectile lives for 5 seconds then auto-destroys
    InitialLifeSpan = 5.f;
}

void AIB4MagicProjectile::BeginPlay()
{
    Super::BeginPlay();

    // Bind collision callback
    CollisionSphere->OnComponentHit.AddDynamic(this, &AIB4MagicProjectile::OnProjectileHit);

    // Apply type configuration (may have been set before spawn via SetMagicType)
    SetMagicType(MagicType);
}

//-----------------------------------------------------------------------------
// SetMagicType — wire up per-type behaviour
//-----------------------------------------------------------------------------

void AIB4MagicProjectile::SetMagicType(EMagicProjectileType NewType)
{
    MagicType = NewType;

    switch (MagicType)
    {
        case EMagicProjectileType::Fire:
            StatusEffect         = EStatusEffect::Burn;
            bHoming              = false;
            bExplodeOnHit        = true;
            ProjectileMovement->ProjectileGravityScale = 0.f;
            if (FireTrailFX) { TrailParticle->SetTemplate(FireTrailFX); }
            break;

        case EMagicProjectileType::Ice:
            StatusEffect         = EStatusEffect::Freeze;
            bHoming              = true;
            bExplodeOnHit        = false; // Ice hits a single target
            ProjectileMovement->bIsHomingProjectile       = true;
            ProjectileMovement->HomingAccelerationMagnitude = HomingAcceleration;
            ProjectileMovement->ProjectileGravityScale    = 0.f;
            if (IceTrailFX) { TrailParticle->SetTemplate(IceTrailFX); }
            break;

        case EMagicProjectileType::Lightning:
            StatusEffect         = EStatusEffect::Stun;
            bHoming              = false;
            bExplodeOnHit        = false;
            ProjectileMovement->InitialSpeed = 4000.f; // Faster
            ProjectileMovement->MaxSpeed     = 6000.f;
            if (LightningTrailFX) { TrailParticle->SetTemplate(LightningTrailFX); }
            break;

        case EMagicProjectileType::Dark:
            StatusEffect         = EStatusEffect::Dark;
            bHoming              = false;
            bExplodeOnHit        = true;
            ProjectileMovement->ProjectileGravityScale = 0.3f; // Slight arc
            if (DarkTrailFX) { TrailParticle->SetTemplate(DarkTrailFX); }
            break;

        default:
            break;
    }
}

//-----------------------------------------------------------------------------
// Homing Target
//-----------------------------------------------------------------------------

void AIB4MagicProjectile::SetHomingTarget(AActor* Target)
{
    HomingTarget = Target;

    if (bHoming && Target)
    {
        // UE4 homing uses the UpdatedComponent's root as the target's pivot
        ProjectileMovement->HomingTargetComponent = Target->GetRootComponent();
    }
}

//-----------------------------------------------------------------------------
// Hit Handler
//-----------------------------------------------------------------------------

void AIB4MagicProjectile::OnProjectileHit(UPrimitiveComponent* HitComp, AActor* OtherActor,
                                           UPrimitiveComponent* OtherComp, FVector NormalImpulse,
                                           const FHitResult& Hit)
{
    if (!OtherActor || OtherActor == GetInstigator())
    {
        return;
    }

    const FVector ImpactLocation = Hit.bBlockingHit ? Hit.ImpactPoint : GetActorLocation();
    const FVector ImpactNormal   = Hit.bBlockingHit ? Hit.ImpactNormal : FVector::UpVector;

    // Direct hit damage on the first target
    UGameplayStatics::ApplyDamage(
        OtherActor,
        MagicDamage,
        GetInstigatorController(),
        this,
        UDamageType::StaticClass()
    );

    // Apply status effect to the primary target
    ApplyStatusEffect(OtherActor);

    // Area damage if explosive
    if (bExplodeOnHit)
    {
        ApplyExplosionDamage(ImpactLocation);
    }

    // Visual / audio FX
    SpawnHitFX(ImpactLocation, ImpactNormal);

    // Destroy after impact
    Destroy();
}

//-----------------------------------------------------------------------------
// Area Damage
//-----------------------------------------------------------------------------

void AIB4MagicProjectile::ApplyExplosionDamage(const FVector& ExplosionOrigin)
{
    UGameplayStatics::ApplyRadialDamage(
        GetWorld(),
        ExplosionDamage,
        ExplosionOrigin,
        ExplosionRadius,
        UDamageType::StaticClass(),
        TArray<AActor*>(), // Ignore list (empty — hits everything in radius)
        this,
        GetInstigatorController(),
        true // Full damage falloff disabled — dealt to all actors equally in radius
    );
}

//-----------------------------------------------------------------------------
// Status Effect
//-----------------------------------------------------------------------------

void AIB4MagicProjectile::ApplyStatusEffect(AActor* Target)
{
    if (!Target || StatusEffect == EStatusEffect::None)
    {
        return;
    }

    // Each status effect is implemented as a gameplay event on the target.
    // In a full implementation these would route through a UStatusEffectComponent.
    // Here we broadcast a generic event with the effect type encoded in the payload.
    // Downstream blueprints or character components respond to this event.

    switch (StatusEffect)
    {
        case EStatusEffect::Burn:
            // Burn: apply DamagePerSecond × StatusEffectDuration spread as ticking damage
            // Simplified: apply the full DoT upfront scaled by duration
            UGameplayStatics::ApplyDamage(
                Target,
                DamagePerSecond * StatusEffectDuration,
                GetInstigatorController(),
                this,
                UDamageType::StaticClass()
            );
            break;

        case EStatusEffect::Freeze:
            // Freeze is handled by the target's character movement; notify via an event.
            // UE4 note: in practice you'd call a UStatusEffectComponent::AddFreeze() here.
            break;

        case EStatusEffect::Stun:
            // Stun: put the target's CombatComponent into the Stunned state
            if (UIB4CombatComponent* CombatComp = Target->FindComponentByClass<UIB4CombatComponent>())
            {
                CombatComp->SetCombatState(ECombatState::Stunned);
            }
            break;

        case EStatusEffect::Dark:
            // Dark: additional damage + intended post-process darkness handled by game code
            UGameplayStatics::ApplyDamage(
                Target,
                MagicDamage * 0.5f, // Extra dark damage
                GetInstigatorController(),
                this,
                UDamageType::StaticClass()
            );
            break;

        default:
            break;
    }
}

//-----------------------------------------------------------------------------
// FX
//-----------------------------------------------------------------------------

void AIB4MagicProjectile::SpawnHitFX(const FVector& Location, const FVector& Normal)
{
    UWorld* World = GetWorld();
    if (!World)
    {
        return;
    }

    // Explosion particle (if assigned)
    if (ExplosionParticle)
    {
        UGameplayStatics::SpawnEmitterAtLocation(
            World,
            ExplosionParticle,
            Location,
            Normal.Rotation(),
            FVector(1.f)
        );
    }

    // Impact sound
    if (ExplosionSound)
    {
        UGameplayStatics::PlaySoundAtLocation(World, ExplosionSound, Location);
    }
}
