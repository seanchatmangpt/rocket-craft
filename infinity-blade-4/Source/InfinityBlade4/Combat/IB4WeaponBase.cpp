// Copyright 2024 Infinity Blade 4. All Rights Reserved.

#include "Combat/IB4WeaponBase.h"
#include "Combat/IB4CombatComponent.h"
#include "Character/IB4Character.h"
#include "Components/CapsuleComponent.h"
#include "Components/SkeletalMeshComponent.h"
#include "Kismet/GameplayStatics.h"
#include "Particles/ParticleSystemComponent.h"
#include "Math/UnrealMathUtility.h"
#include "Engine/World.h"

//-----------------------------------------------------------------------------
// Construction
//-----------------------------------------------------------------------------

AIB4WeaponBase::AIB4WeaponBase()
{
    PrimaryActorTick.bCanEverTick = false;

    // Weapon mesh — primary visible component
    WeaponMesh = CreateDefaultSubobject<USkeletalMeshComponent>(TEXT("WeaponMesh"));
    SetRootComponent(WeaponMesh);

    // Hit capsule — positioned along the blade / fist / staff tip
    HitCollision = CreateDefaultSubobject<UCapsuleComponent>(TEXT("HitCollision"));
    HitCollision->SetupAttachment(WeaponMesh);
    HitCollision->SetCapsuleHalfHeight(40.f);
    HitCollision->SetCapsuleRadius(8.f);
    HitCollision->SetCollisionProfileName(TEXT("OverlapAllDynamic"));
    HitCollision->SetGenerateOverlapEvents(true);

    // Collision starts disabled; enabled only during active attack window
    HitCollision->SetCollisionEnabled(ECollisionEnabled::NoCollision);

    SocketName   = TEXT("WeaponSocket");
    WeaponType   = EWeaponType::Sword;
}

void AIB4WeaponBase::BeginPlay()
{
    Super::BeginPlay();

    // Bind overlap — used for melee swing detection
    HitCollision->OnComponentBeginOverlap.AddDynamic(this, &AIB4WeaponBase::OnOverlapBegin);
    HitCollision->OnComponentHit.AddDynamic(this, &AIB4WeaponBase::OnHit);
}

//-----------------------------------------------------------------------------
// Equip / Unequip
//-----------------------------------------------------------------------------

void AIB4WeaponBase::Equip(AIB4Character* NewOwner)
{
    if (!NewOwner)
    {
        return;
    }

    OwnerCharacter = NewOwner;
    SetOwner(NewOwner);

    // Attach to the character's hand bone socket
    USkeletalMeshComponent* CharMesh = NewOwner->GetMesh();
    if (CharMesh)
    {
        AttachToComponent(
            CharMesh,
            FAttachmentTransformRules::SnapToTargetNotIncludingScale,
            SocketName
        );
    }

    // Notify the combat component that a weapon is now active
    if (UIB4CombatComponent* CombatComp = NewOwner->FindComponentByClass<UIB4CombatComponent>())
    {
        CombatComp->SetEquippedWeapon(this);
    }
}

void AIB4WeaponBase::Unequip()
{
    // Detach from character
    DetachFromActor(FDetachmentTransformRules::KeepWorldTransform);

    // Clear the combat component reference
    if (OwnerCharacter.IsValid())
    {
        if (UIB4CombatComponent* CombatComp =
                OwnerCharacter->FindComponentByClass<UIB4CombatComponent>())
        {
            CombatComp->SetEquippedWeapon(nullptr);
        }
    }

    OwnerCharacter = nullptr;
    SetOwner(nullptr);
}

//-----------------------------------------------------------------------------
// Collision Control
//-----------------------------------------------------------------------------

void AIB4WeaponBase::EnableHitCollision()
{
    HitActorsThisSwing.Empty();
    HitCollision->SetCollisionEnabled(ECollisionEnabled::QueryOnly);
}

void AIB4WeaponBase::DisableHitCollision()
{
    HitCollision->SetCollisionEnabled(ECollisionEnabled::NoCollision);
    HitActorsThisSwing.Empty();
}

//-----------------------------------------------------------------------------
// Hit / Overlap Handlers
//-----------------------------------------------------------------------------

void AIB4WeaponBase::OnOverlapBegin(UPrimitiveComponent* OverlappedComp, AActor* OtherActor,
                                     UPrimitiveComponent* OtherComp, int32 OtherBodyIndex,
                                     bool bFromSweep, const FHitResult& SweepResult)
{
    if (!OtherActor || OtherActor == GetOwner())
    {
        return;
    }

    // Prevent the same actor from being hit more than once per swing
    for (const TWeakObjectPtr<AActor>& AlreadyHit : HitActorsThisSwing)
    {
        if (AlreadyHit.Get() == OtherActor)
        {
            return;
        }
    }

    HitActorsThisSwing.Add(OtherActor);

    // Crit roll
    const float RollResult = FMath::FRand();
    const float EffectiveCrit = GetCritChance();
    const bool bIsCrit = RollResult <= EffectiveCrit;
    const float CritMultiplier = bIsCrit ? 2.0f : 1.0f;

    // Combo multiplier from combat component
    float ComboMultiplier = 1.0f;
    if (OwnerCharacter.IsValid())
    {
        if (UIB4CombatComponent* CombatComp =
                OwnerCharacter->FindComponentByClass<UIB4CombatComponent>())
        {
            ComboMultiplier = CombatComp->GetComboMultiplier();
        }
    }

    const float FinalDamage = GetAttackDamage() * CritMultiplier * ComboMultiplier;

    // Apply damage via the gameplay statics pipeline
    AController* InstigatorController = OwnerCharacter.IsValid()
                                            ? OwnerCharacter->GetController()
                                            : nullptr;

    UGameplayStatics::ApplyDamage(
        OtherActor,
        FinalDamage,
        InstigatorController,
        this,
        UDamageType::StaticClass()
    );

    // --- FX ---
    const FVector HitLocation = SweepResult.bBlockingHit
                                    ? SweepResult.ImpactPoint
                                    : OtherActor->GetActorLocation();

    if (ImpactParticle)
    {
        UGameplayStatics::SpawnEmitterAtLocation(
            GetWorld(),
            ImpactParticle,
            HitLocation,
            FRotator::ZeroRotator,
            FVector(1.f)
        );
    }

    if (ImpactSound)
    {
        UGameplayStatics::PlaySoundAtLocation(GetWorld(), ImpactSound, HitLocation);
    }

    // Notify the combat component that a hit landed (for combo progression)
    if (OwnerCharacter.IsValid())
    {
        if (UIB4CombatComponent* CombatComp =
                OwnerCharacter->FindComponentByClass<UIB4CombatComponent>())
        {
            CombatComp->OnAttackHit(OtherActor);
        }
    }
}

void AIB4WeaponBase::OnHit(UPrimitiveComponent* HitComp, AActor* OtherActor,
                            UPrimitiveComponent* OtherComp, FVector NormalImpulse,
                            const FHitResult& Hit)
{
    // Physics-based hit — delegate to overlap handler for damage/FX consistency
    OnOverlapBegin(HitComp, OtherActor, OtherComp, INDEX_NONE, true, Hit);
}

//-----------------------------------------------------------------------------
// Stat Accessors
//-----------------------------------------------------------------------------

float AIB4WeaponBase::GetAttackDamage() const
{
    float Total = BaseStats.AttackDamage;

    for (const FGemBonus& Gem : EquippedGems)
    {
        Total += Gem.AttackBonus;
    }

    return Total;
}

float AIB4WeaponBase::GetMagicDamage() const
{
    float Total = BaseStats.MagicDamage;

    for (const FGemBonus& Gem : EquippedGems)
    {
        Total += Gem.MagicBonus;
    }

    return Total;
}

float AIB4WeaponBase::GetCritChance() const
{
    float Total = BaseStats.CritChance;

    for (const FGemBonus& Gem : EquippedGems)
    {
        Total += Gem.CritBonus;
    }

    return FMath::Clamp(Total, 0.f, 1.f);
}
