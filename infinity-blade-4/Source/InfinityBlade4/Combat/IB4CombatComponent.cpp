// Copyright 2024 Infinity Blade 4. All Rights Reserved.

#include "Combat/IB4CombatComponent.h"
#include "Combat/IB4WeaponBase.h"
#include "Combat/IB4AttackChain.h"
#include "Combat/IB4ParrySystem.h"
#include "Combat/IB4MagicProjectile.h"
#include "Character/IB4Character.h"
#include "Animation/AnimInstance.h"
#include "GameFramework/Character.h"
#include "GameFramework/CharacterMovementComponent.h"
#include "Kismet/GameplayStatics.h"
#include "TimerManager.h"
#include "Engine/World.h"

//-----------------------------------------------------------------------------
// Construction / Lifecycle
//-----------------------------------------------------------------------------

UIB4CombatComponent::UIB4CombatComponent()
{
    PrimaryComponentTick.bCanEverTick = true;

    CurrentState             = ECombatState::Idle;
    ComboCount               = 0;
    LastAttackTime           = 0.f;
    ComboResetTime           = 2.f;
    bAttackInputBuffered     = false;
    BufferedAttackDirection  = EAttackDirection::Right;
    bInvincible              = false;
    DodgeInvincibilityDuration = 0.3f;
    DodgeLaunchSpeed         = 800.f;
    MagicCost                = 25.f;

    // Create the parry logic sub-object
    ParrySystem = CreateDefaultSubobject<UIB4ParrySystem>(TEXT("ParrySystem"));
}

void UIB4CombatComponent::BeginPlay()
{
    Super::BeginPlay();

    OwnerCharacter = Cast<AIB4Character>(GetOwner());
}

void UIB4CombatComponent::TickComponent(float DeltaTime, ELevelTick TickType,
                                         FActorComponentTickFunction* ThisTickFunction)
{
    Super::TickComponent(DeltaTime, TickType, ThisTickFunction);

    // Combo expiry check is handled via timer; tick kept for potential future use.
}

//-----------------------------------------------------------------------------
// State Machine
//-----------------------------------------------------------------------------

void UIB4CombatComponent::SetCombatState(ECombatState NewState)
{
    if (CurrentState == NewState)
    {
        return;
    }

    ECombatState OldState = CurrentState;
    CurrentState = NewState;

    OnCombatStateChanged.Broadcast(OldState, NewState);
}

//-----------------------------------------------------------------------------
// Combo
//-----------------------------------------------------------------------------

void UIB4CombatComponent::ResetCombo()
{
    ComboCount              = 0;
    bAttackInputBuffered    = false;
    LastAttackTime          = 0.f;
    GetWorld()->GetTimerManager().ClearTimer(TimerHandle_ComboReset);
}

float UIB4CombatComponent::GetComboMultiplier() const
{
    if (ComboCount <= 1) return 1.0f;
    if (ComboCount == 2) return 1.5f;
    if (ComboCount == 3) return 2.0f;
    return 3.0f; // 4+ hits
}

//-----------------------------------------------------------------------------
// Attack
//-----------------------------------------------------------------------------

UAnimMontage* UIB4CombatComponent::SelectAttackMontage(EAttackDirection Dir, int32 ComboLevel) const
{
    // Layout: 4 directions × 3 combo levels = 12 slots
    // Index = DirectionIndex * 3 + clamp(ComboLevel, 0, 2)
    const int32 DirectionIndex = static_cast<int32>(Dir); // Left=0, Right=1, Up=2, Down=3
    const int32 ClampedLevel   = FMath::Clamp(ComboLevel, 0, 2);
    const int32 MontageIndex   = DirectionIndex * 3 + ClampedLevel;

    if (AttackMontages.IsValidIndex(MontageIndex))
    {
        return AttackMontages[MontageIndex];
    }

    return nullptr;
}

void UIB4CombatComponent::BeginAttack(EAttackDirection Dir)
{
    if (CurrentState == ECombatState::Dead || CurrentState == ECombatState::Stunned)
    {
        return;
    }

    // Buffer the input if we're mid-swing and still in the combo chain window
    if (CurrentState == ECombatState::Attacking)
    {
        bAttackInputBuffered   = true;
        BufferedAttackDirection = Dir;
        return;
    }

    // Refresh combo timer
    GetWorld()->GetTimerManager().ClearTimer(TimerHandle_ComboReset);
    GetWorld()->GetTimerManager().SetTimer(
        TimerHandle_ComboReset,
        this,
        &UIB4CombatComponent::ResetCombo,
        ComboResetTime,
        false
    );

    // Clamp combo for montage selection (0-based, max combo level index = 2)
    const int32 ComboLevel = FMath::Clamp(ComboCount, 0, 2);

    UAnimMontage* MontageToPlay = SelectAttackMontage(Dir, ComboLevel);

    if (MontageToPlay)
    {
        if (OwnerCharacter.IsValid())
        {
            if (UAnimInstance* AnimInst = OwnerCharacter->GetMesh()->GetAnimInstance())
            {
                AnimInst->Montage_Play(MontageToPlay, 1.0f);
            }
        }
    }

    LastAttackTime = GetWorld()->GetTimeSeconds();
    SetCombatState(ECombatState::Attacking);
}

void UIB4CombatComponent::OnAttackHit(AActor* Target)
{
    if (!Target || !OwnerCharacter.IsValid())
    {
        return;
    }

    // Damage = base weapon damage × combo multiplier
    float BaseDamage = 0.f;
    if (AIB4WeaponBase* Weapon = EquippedWeapon.Get())
    {
        BaseDamage = Weapon->GetAttackDamage();
    }

    const float FinalDamage = BaseDamage * GetComboMultiplier();

    UGameplayStatics::ApplyDamage(
        Target,
        FinalDamage,
        OwnerCharacter->GetController(),
        OwnerCharacter.Get(),
        UDamageType::StaticClass()
    );

    ++ComboCount;

    OnAttackLanded.Broadcast(Target, FinalDamage);
}

void UIB4CombatComponent::OnAttackMontageEnded()
{
    // If there is a buffered input, chain immediately into the next attack
    if (bAttackInputBuffered)
    {
        const EAttackDirection NextDir = BufferedAttackDirection;
        bAttackInputBuffered           = false;

        SetCombatState(ECombatState::Idle);
        BeginAttack(NextDir);
    }
    else
    {
        SetCombatState(ECombatState::Idle);
    }
}

void UIB4CombatComponent::SetWeaponCollisionEnabled(bool bEnabled)
{
    if (AIB4WeaponBase* Weapon = EquippedWeapon.Get())
    {
        if (bEnabled)
        {
            Weapon->EnableHitCollision();
        }
        else
        {
            Weapon->DisableHitCollision();
        }
    }
}

//-----------------------------------------------------------------------------
// Parry
//-----------------------------------------------------------------------------

bool UIB4CombatComponent::AttemptParry()
{
    if (CurrentState == ECombatState::Dead    ||
        CurrentState == ECombatState::Stunned ||
        CurrentState == ECombatState::Dodging)
    {
        return false;
    }

    SetCombatState(ECombatState::Parrying);

    if (ParryMontage && OwnerCharacter.IsValid())
    {
        if (UAnimInstance* AnimInst = OwnerCharacter->GetMesh()->GetAnimInstance())
        {
            AnimInst->Montage_Play(ParryMontage, 1.0f);
        }
    }

    // Open the parry window — duration controlled by ParrySystem
    if (ParrySystem)
    {
        ParrySystem->BeginParryWindow();

        GetWorld()->GetTimerManager().SetTimer(
            TimerHandle_ParryWindow,
            this,
            &UIB4CombatComponent::OnParryWindowExpired,
            ParrySystem->ParryWindow,
            false
        );
    }

    return true;
}

void UIB4CombatComponent::OnParryWindowExpired()
{
    if (CurrentState == ECombatState::Parrying)
    {
        SetCombatState(ECombatState::Idle);
    }

    if (ParrySystem)
    {
        ParrySystem->EndParryWindow();
    }
}

//-----------------------------------------------------------------------------
// Dodge
//-----------------------------------------------------------------------------

void UIB4CombatComponent::ExecuteDodge(FVector Direction)
{
    if (CurrentState == ECombatState::Dead || CurrentState == ECombatState::Stunned)
    {
        return;
    }

    SetCombatState(ECombatState::Dodging);
    bInvincible = true;

    if (DodgeMontage && OwnerCharacter.IsValid())
    {
        if (UAnimInstance* AnimInst = OwnerCharacter->GetMesh()->GetAnimInstance())
        {
            AnimInst->Montage_Play(DodgeMontage, 1.0f);
        }

        // Apply a burst of movement using LaunchCharacter
        ACharacter* OwnerAsChar = Cast<ACharacter>(OwnerCharacter.Get());
        if (OwnerAsChar)
        {
            const FVector LaunchVelocity = Direction.GetSafeNormal() * DodgeLaunchSpeed;
            OwnerAsChar->LaunchCharacter(LaunchVelocity, true, true);
        }
    }

    // Schedule end of invincibility window
    GetWorld()->GetTimerManager().SetTimer(
        TimerHandle_DodgeInvincibility,
        this,
        &UIB4CombatComponent::EndDodgeInvincibility,
        DodgeInvincibilityDuration,
        false
    );
}

void UIB4CombatComponent::EndDodgeInvincibility()
{
    bInvincible = false;

    if (CurrentState == ECombatState::Dodging)
    {
        SetCombatState(ECombatState::Idle);
    }
}

//-----------------------------------------------------------------------------
// Magic
//-----------------------------------------------------------------------------

void UIB4CombatComponent::CastMagic(EMagicType Type)
{
    if (CurrentState == ECombatState::Dead || CurrentState == ECombatState::Stunned)
    {
        return;
    }

    if (!OwnerCharacter.IsValid() || !MagicProjectileClass)
    {
        return;
    }

    // Drain mana — the character is responsible for tracking the pool
    OwnerCharacter->ConsumeMana(MagicCost);

    // Spawn projectile at character's hand socket, aimed forward
    const FVector SpawnLocation  = OwnerCharacter->GetMesh()
                                        ->GetSocketLocation(TEXT("RightHandSocket"));
    const FRotator SpawnRotation = OwnerCharacter->GetActorRotation();

    FActorSpawnParameters SpawnParams;
    SpawnParams.Owner   = OwnerCharacter.Get();
    SpawnParams.Instigator = Cast<APawn>(OwnerCharacter.Get());

    if (AIB4MagicProjectile* Proj = GetWorld()->SpawnActor<AIB4MagicProjectile>(
            MagicProjectileClass, SpawnLocation, SpawnRotation, SpawnParams))
    {
        Proj->SetMagicType(Type);
    }
}

//-----------------------------------------------------------------------------
// Damage Reception
//-----------------------------------------------------------------------------

void UIB4CombatComponent::OnDamageReceived(float Damage, EAttackDirection IncomingDirection)
{
    if (CurrentState == ECombatState::Dead)
    {
        return;
    }

    // Invincibility from dodge absorbs all damage
    if (bInvincible)
    {
        return;
    }

    // Check active parry
    if (CurrentState == ECombatState::Parrying && ParrySystem)
    {
        // Let the parry system evaluate the result
        // (Direction comparison is handled externally between attacker/defender)
        if (ParrySystem->IsInParryWindow())
        {
            // Successful parry — cancel the incoming damage
            return;
        }
    }

    // Apply the damage to the character's health pool
    if (OwnerCharacter.IsValid())
    {
        OwnerCharacter->ReceiveCombatDamage(Damage);

        if (OwnerCharacter->IsDead())
        {
            SetCombatState(ECombatState::Dead);
            OnPlayerDefeated.Broadcast();
        }
        else
        {
            // Interrupt current action and apply stagger
            SetCombatState(ECombatState::Stunned);

            if (StunMontage)
            {
                if (UAnimInstance* AnimInst = OwnerCharacter->GetMesh()->GetAnimInstance())
                {
                    AnimInst->Montage_Play(StunMontage, 1.0f);
                }
            }
        }
    }
}

//-----------------------------------------------------------------------------
// Utilities
//-----------------------------------------------------------------------------

void UIB4CombatComponent::BreakStance(AActor* Target)
{
    if (!Target)
    {
        return;
    }

    // Apply a bonus "break" damage and attempt to put the target into Stunned state
    UGameplayStatics::ApplyDamage(
        Target,
        0.f, // Stance break deals no raw HP damage by default
        OwnerCharacter.IsValid() ? OwnerCharacter->GetController() : nullptr,
        OwnerCharacter.Get(),
        UDamageType::StaticClass()
    );

    // Notify the target's combat component (if present) that their guard was broken
    if (UIB4CombatComponent* TargetCombat = Target->FindComponentByClass<UIB4CombatComponent>())
    {
        TargetCombat->SetCombatState(ECombatState::Stunned);
    }
}

void UIB4CombatComponent::OnEnemyDefeated(AActor* DefeatedEnemy)
{
    // Reset the combo counter and clear timers
    GetWorld()->GetTimerManager().ClearTimer(TimerHandle_ComboReset);
    ResetCombo();

    SetCombatState(ECombatState::Idle);

    // Trigger loot on the defeated enemy
    if (DefeatedEnemy)
    {
        DefeatedEnemy->OnActorEndPlay.AddDynamic(this, &UIB4CombatComponent::ResetCombo);
        // Notify any interested listeners (UI, sound, etc.)
        OnEnemyDefeated_Delegate.Broadcast(DefeatedEnemy);
    }
}

//-----------------------------------------------------------------------------
// Weapon Reference
//-----------------------------------------------------------------------------

void UIB4CombatComponent::SetEquippedWeapon(AIB4WeaponBase* Weapon)
{
    EquippedWeapon = Weapon;
}
