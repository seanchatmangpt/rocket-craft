// Copyright 2024 Infinity Blade 4 Project. All Rights Reserved.

#include "Characters/IB4Character.h"
#include "Net/UnrealNetwork.h"
#include "Components/CapsuleComponent.h"
#include "GameFramework/CharacterMovementComponent.h"

AIB4Character::AIB4Character(const FObjectInitializer& ObjectInitializer)
    : Super(ObjectInitializer)
{
    // Replication
    bReplicates = true;
    bAlwaysRelevant = false;

    // Default resource values
    Health          = 100.f;
    MaxHealth       = 100.f;
    Magic           = 50.f;
    MaxMagic        = 50.f;
    BloodlineLevel  = 0;
    CurrentXP       = 0.f;
    bIsDying        = false;

    // Capsule — standard humanoid combat dimensions
    GetCapsuleComponent()->InitCapsuleSize(42.f, 96.f);

    // Rotate toward movement direction; controller yaw drives camera only
    bUseControllerRotationPitch = false;
    bUseControllerRotationYaw   = false;
    bUseControllerRotationRoll  = false;

    GetCharacterMovement()->bOrientRotationToMovement = true;
    GetCharacterMovement()->RotationRate               = FRotator(0.f, 540.f, 0.f);
    GetCharacterMovement()->JumpZVelocity              = 600.f;
    GetCharacterMovement()->AirControl                 = 0.2f;

    // Mesh — the actual skeletal mesh asset is set in the Blueprint subclass
    GetMesh()->SetRelativeLocation(FVector(0.f, 0.f, -97.f));
    GetMesh()->SetRelativeRotation(FRotator(0.f, -90.f, 0.f));
}

void AIB4Character::BeginPlay()
{
    Super::BeginPlay();

    // Clamp to valid range on begin play in case BP defaults were misconfigured
    Health = FMath::Clamp(Health, 0.f, MaxHealth);
    Magic  = FMath::Clamp(Magic,  0.f, MaxMagic);
}

void AIB4Character::GetLifetimeReplicatedProps(TArray<FLifetimeProperty>& OutLifetimeProps) const
{
    Super::GetLifetimeReplicatedProps(OutLifetimeProps);

    DOREPLIFETIME(AIB4Character, Health);
    DOREPLIFETIME(AIB4Character, MaxHealth);
    DOREPLIFETIME(AIB4Character, Magic);
    DOREPLIFETIME(AIB4Character, MaxMagic);
    DOREPLIFETIME(AIB4Character, BloodlineLevel);
    DOREPLIFETIME(AIB4Character, CurrentXP);
}

// ---------------------------------------------------------------------------
// Health
// ---------------------------------------------------------------------------

float AIB4Character::GetHealthPercent() const
{
    if (MaxHealth <= 0.f)
    {
        return 0.f;
    }
    return FMath::Clamp(Health / MaxHealth, 0.f, 1.f);
}

bool AIB4Character::IsAlive() const
{
    return Health > 0.f && !bIsDying;
}

void AIB4Character::Heal(float HealthAmount)
{
    if (!IsAlive() || HealthAmount <= 0.f)
    {
        return;
    }

    Health = FMath::Clamp(Health + HealthAmount, 0.f, MaxHealth);
}

void AIB4Character::OnRep_Health()
{
    // Clients: trigger UI refresh or hit reactions when health changes.
    // Concrete BP subclasses handle visual feedback via this hook.
    if (!IsAlive() && !bIsDying)
    {
        OnDeath();
    }
}

// ---------------------------------------------------------------------------
// Magic
// ---------------------------------------------------------------------------

float AIB4Character::GetMagicPercent() const
{
    if (MaxMagic <= 0.f)
    {
        return 0.f;
    }
    return FMath::Clamp(Magic / MaxMagic, 0.f, 1.f);
}

void AIB4Character::RestoreMagic(float MagicAmount)
{
    if (!IsAlive() || MagicAmount <= 0.f)
    {
        return;
    }

    Magic = FMath::Clamp(Magic + MagicAmount, 0.f, MaxMagic);
}

void AIB4Character::OnRep_Magic()
{
    // Clients: notify UI that magic bar changed.
}

// ---------------------------------------------------------------------------
// Damage & Death
// ---------------------------------------------------------------------------

float AIB4Character::TakeDamage(float DamageAmount, struct FDamageEvent const& DamageEvent,
                                 class AController* EventInstigator, AActor* DamageCauser)
{
    if (bIsDying || !IsAlive())
    {
        return 0.f;
    }

    // Apply bloodline defense reduction — minimum 1 point of damage always lands
    const float EffectiveDamage = FMath::Max(1.f, DamageAmount - BloodlineStats.DefenseBonus);

    const float ActualDamage = Super::TakeDamage(EffectiveDamage, DamageEvent, EventInstigator, DamageCauser);

    Health = FMath::Clamp(Health - ActualDamage, 0.f, MaxHealth);

    if (Health <= 0.f && !bIsDying)
    {
        OnDeath();
    }

    return ActualDamage;
}

void AIB4Character::OnDeath_Implementation()
{
    bIsDying = true;

    // Disable further input and collision responses
    GetCapsuleComponent()->SetCollisionEnabled(ECollisionEnabled::NoCollision);
    GetCapsuleComponent()->SetCollisionResponseToAllChannels(ECR_Ignore);

    // Detach from controller so it can possess a new pawn
    DetachFromControllerPendingDestroy();

    SetRagdollPhysics();
}

// ---------------------------------------------------------------------------
// Combat component interface
// ---------------------------------------------------------------------------

void AIB4Character::ReceiveCombatDamage(float DamageAmount)
{
    if (bIsDying || DamageAmount <= 0.f)
    {
        return;
    }

    Health = FMath::Clamp(Health - DamageAmount, 0.f, MaxHealth);

    if (Health <= 0.f && !bIsDying)
    {
        OnDeath();
    }
}

float AIB4Character::ConsumeMana(float ManaAmount)
{
    if (ManaAmount <= 0.f)
    {
        return 0.f;
    }

    const float Consumed = FMath::Min(ManaAmount, Magic);
    Magic = FMath::Clamp(Magic - Consumed, 0.f, MaxMagic);
    return Consumed;
}

bool AIB4Character::IsDead() const
{
    return !IsAlive();
}

void AIB4Character::SetRagdollPhysics()
{
    USkeletalMeshComponent* Mesh = GetMesh();
    if (!Mesh)
    {
        return;
    }

    // Stop animation and let physics drive the bones
    Mesh->SetAllBodiesSimulatePhysics(true);
    Mesh->SetSimulatePhysics(true);
    Mesh->WakeAllRigidBodies();
    Mesh->bBlendPhysics = true;

    // Disable movement component so the ragdoll doesn't fight physics
    if (GetCharacterMovement())
    {
        GetCharacterMovement()->StopMovementImmediately();
        GetCharacterMovement()->DisableMovement();
        GetCharacterMovement()->SetComponentTickEnabled(false);
    }
}
