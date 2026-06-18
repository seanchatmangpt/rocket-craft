// Copyright 2024 Infinity Blade 4. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "Components/ActorComponent.h"
// EAttackDirection, EMagicType, and supporting types are defined centrally in IB4Types.h
#include "Core/IB4Types.h"
#include "IB4CombatComponent.generated.h"

class AIB4Character;
class AIB4WeaponBase;
class UIB4AttackChain;
class UIB4ParrySystem;

UENUM(BlueprintType)
enum class ECombatState : uint8
{
    Idle        UMETA(DisplayName = "Idle"),
    Attacking   UMETA(DisplayName = "Attacking"),
    Parrying    UMETA(DisplayName = "Parrying"),
    Dodging     UMETA(DisplayName = "Dodging"),
    Stunned     UMETA(DisplayName = "Stunned"),
    Dead        UMETA(DisplayName = "Dead")
};

/** Delegate broadcast when combat state changes */
DECLARE_DYNAMIC_MULTICAST_DELEGATE_TwoParams(FOnCombatStateChanged, ECombatState, OldState, ECombatState, NewState);

/** Delegate broadcast when a hit lands on a target */
DECLARE_DYNAMIC_MULTICAST_DELEGATE_TwoParams(FOnAttackLanded, AActor*, Target, float, DamageDealt);

/** Delegate broadcast when the player is defeated */
DECLARE_DYNAMIC_MULTICAST_DELEGATE(FOnPlayerDefeated);

/** Delegate broadcast when an enemy is defeated */
DECLARE_DYNAMIC_MULTICAST_DELEGATE_OneParam(FOnEnemyDefeatedDelegate, AActor*, DefeatedEnemy);

UCLASS(ClassGroup=(Combat), meta=(BlueprintSpawnableComponent))
class INFINITYBLADE4_API UIB4CombatComponent : public UActorComponent
{
    GENERATED_BODY()

public:
    UIB4CombatComponent();

    UFUNCTION(BlueprintCallable, Category = "Combat|Magic")
    float GetMagicCost() const;

protected:
    virtual void BeginPlay() override;
    virtual void TickComponent(float DeltaTime, ELevelTick TickType, FActorComponentTickFunction* ThisTickFunction) override;

    //-----------------------------------------------------------------------
    // State Machine
    //-----------------------------------------------------------------------

    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Combat|State")
    ECombatState CurrentState;

    /** Sets the combat state and fires OnCombatStateChanged */
    void SetCombatState(ECombatState NewState);

    //-----------------------------------------------------------------------
    // Combo Tracking
    //-----------------------------------------------------------------------

    /** Number of consecutive attacks in the current chain */
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Combat|Combo")
    int32 ComboCount;

    /** World time of the most recent attack */
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Combat|Combo")
    float LastAttackTime;

    /** Seconds without an attack before the combo resets to 1x */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Combat|Combo")
    float ComboResetTime;

    /** Timer handle used to expire the combo window */
    FTimerHandle TimerHandle_ComboReset;

    /** Whether the player pressed attack while a montage was still playing */
    bool bAttackInputBuffered;

    /** Buffered direction from a queued attack */
    EAttackDirection BufferedAttackDirection;

    /** Resets ComboCount to 0 and clears the input buffer */
    void ResetCombo();

    //-----------------------------------------------------------------------
    // Attack
    //-----------------------------------------------------------------------

    /**
     * Begin an attack in the given direction.
     * Selects the correct montage based on ComboCount and Direction,
     * transitions state to Attacking.
     */
    UFUNCTION(BlueprintCallable, Category = "Combat|Attack")
    void BeginAttack(EAttackDirection Dir);

    /** Called by an anim notify when the swing hits geometry/actors */
    UFUNCTION(BlueprintCallable, Category = "Combat|Attack")
    void OnAttackHit(AActor* Target);

    /** Called by an anim notify at the end of the attack window */
    UFUNCTION(BlueprintCallable, Category = "Combat|Attack")
    void OnAttackMontageEnded();

    /** Enable/disable the weapon hit-collision capsule (called from anim notifies) */
    UFUNCTION(BlueprintCallable, Category = "Combat|Attack")
    void SetWeaponCollisionEnabled(bool bEnabled);

    //-----------------------------------------------------------------------
    // Parry
    //-----------------------------------------------------------------------

    /**
     * Attempt to enter the parry stance.
     * Returns true if the parry was registered (state allows it).
     */
    UFUNCTION(BlueprintCallable, Category = "Combat|Parry")
    bool AttemptParry();

    /** Called internally when the parry window expires without a clash */
    void OnParryWindowExpired();

    FTimerHandle TimerHandle_ParryWindow;

    //-----------------------------------------------------------------------
    // Dodge
    //-----------------------------------------------------------------------

    /**
     * Execute a directional dodge.
     * Grants 0.3s invincibility via LaunchCharacter.
     */
    UFUNCTION(BlueprintCallable, Category = "Combat|Dodge")
    void ExecuteDodge(FVector Direction);

    /** Whether the character is currently in an invincibility window */
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Combat|Dodge")
    bool bInvincible;

    /** Duration (seconds) of dodge invincibility */
    UPROPERTY(EditDefaultsOnly, Category = "Combat|Dodge")
    float DodgeInvincibilityDuration;

    /** Impulse strength applied during dodge */
    UPROPERTY(EditDefaultsOnly, Category = "Combat|Dodge")
    float DodgeLaunchSpeed;

    FTimerHandle TimerHandle_DodgeInvincibility;

    void EndDodgeInvincibility();

    //-----------------------------------------------------------------------
    // Magic
    //-----------------------------------------------------------------------

    /**
     * Cast a magic spell of the given type.
     * Consumes MagicCost mana and fires a projectile.
     */
    UFUNCTION(BlueprintCallable, Category = "Combat|Magic")
    void CastMagic(EMagicType Type);

    UPROPERTY(EditDefaultsOnly, Category = "Combat|Magic")
    float MagicCost;

    UPROPERTY(EditDefaultsOnly, Category = "Combat|Magic")
    TSubclassOf<class AIB4MagicProjectile> MagicProjectileClass;

    //-----------------------------------------------------------------------
    // Damage Reception
    //-----------------------------------------------------------------------

    /**
     * Called when this actor receives damage.
     * Checks whether an active parry or block absorbs or redirects the hit.
     */
    UFUNCTION(BlueprintCallable, Category = "Combat|Damage")
    void OnDamageReceived(float Damage, EAttackDirection IncomingDirection);

    //-----------------------------------------------------------------------
    // Combat Utilities
    //-----------------------------------------------------------------------

    /**
     * Returns the combo damage multiplier for the current combo depth.
     * 1 hit → 1.0x, 2 hits → 1.5x, 3 hits → 2.0x, 4+ hits → 3.0x
     */
    UFUNCTION(BlueprintCallable, Category = "Combat|Combo")
    float GetComboMultiplier() const;

    /**
     * Break an enemy's guard/shield stance, causing stagger.
     * Typically called after landing a strong attack on a blocking enemy.
     */
    UFUNCTION(BlueprintCallable, Category = "Combat|Attack")
    void BreakStance(AActor* Target);

    /**
     * Called when the owner kills an enemy.
     * Resets combo, triggers loot drop, broadcasts delegate.
     */
    UFUNCTION(BlueprintCallable, Category = "Combat")
    void OnEnemyDefeated(AActor* DefeatedEnemy);

    //-----------------------------------------------------------------------
    // Animation Montage Selection
    //-----------------------------------------------------------------------

    /**
     * 3 directions × 3 combo levels = 9 unique attack montage slots.
     * Array layout: [Direction][ComboLevel] → montage asset.
     * Direction index: Overhead=0, Left=1, Right=2 (EAttackDirection enum order)
     * Total = 3 directions × 3 combo levels = 9 slots.
     */
    UPROPERTY(EditDefaultsOnly, Category = "Combat|Animation")
    TArray<UAnimMontage*> AttackMontages;

    /** Montage played when entering parry stance */
    UPROPERTY(EditDefaultsOnly, Category = "Combat|Animation")
    UAnimMontage* ParryMontage;

    /** Montage played during a dodge roll */
    UPROPERTY(EditDefaultsOnly, Category = "Combat|Animation")
    UAnimMontage* DodgeMontage;

    /** Montage played when stance is broken */
    UPROPERTY(EditDefaultsOnly, Category = "Combat|Animation")
    UAnimMontage* StunMontage;

    /** Returns the montage for the given direction and 0-based combo level (clamped to 2) */
    UAnimMontage* SelectAttackMontage(EAttackDirection Dir, int32 ComboLevel) const;

    //-----------------------------------------------------------------------
    // References
    //-----------------------------------------------------------------------

    /** Weak reference back to the owning IB4 character */
    TWeakObjectPtr<AIB4Character> OwnerCharacter;

    /** Currently equipped weapon */
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Combat")
    TWeakObjectPtr<AIB4WeaponBase> EquippedWeapon;

    /** Data asset defining the current weapon's attack chain */
    UPROPERTY(EditDefaultsOnly, Category = "Combat|Attack")
    UIB4AttackChain* AttackChainData;

    /** Parry logic sub-object */
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Combat|Parry")
    UIB4ParrySystem* ParrySystem;

    //-----------------------------------------------------------------------
    // Delegates
    //-----------------------------------------------------------------------

    UPROPERTY(BlueprintAssignable, Category = "Combat|Events")
    FOnCombatStateChanged OnCombatStateChanged;

    UPROPERTY(BlueprintAssignable, Category = "Combat|Events")
    FOnAttackLanded OnAttackLanded;

    UPROPERTY(BlueprintAssignable, Category = "Combat|Events")
    FOnPlayerDefeated OnPlayerDefeated;

    UPROPERTY(BlueprintAssignable, Category = "Combat|Events")
    FOnEnemyDefeatedDelegate OnEnemyDefeated_Delegate;

public:
    /** Set the active weapon reference (called from character equip logic) */
    void SetEquippedWeapon(AIB4WeaponBase* Weapon);

    FORCEINLINE ECombatState GetCurrentState() const { return CurrentState; }
    FORCEINLINE int32 GetComboCount() const { return ComboCount; }
    FORCEINLINE bool IsInvincible() const { return bInvincible; }

    /** Returns the magic cost of casting spells. */
    float GetMagicCost() const { return MagicCost; }
};
