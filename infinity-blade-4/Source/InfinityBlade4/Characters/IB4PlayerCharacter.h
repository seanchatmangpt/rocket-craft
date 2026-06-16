// Copyright 2024 Infinity Blade 4 Project. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "Characters/IB4Character.h"
#include "Core/IB4Types.h"
#include "IB4PlayerCharacter.generated.h"

class USpringArmComponent;
class UCameraComponent;

// Component forward declarations
// UIB4CombatComponent has a full implementation in Combat/IB4CombatComponent.h
// UIB4EquipmentComponent and UIB4BloodlineComponent are stub headers in Components/
class UIB4EquipmentComponent;
class UIB4CombatComponent;
class UIB4BloodlineComponent;

/**
 * AIB4PlayerCharacter is the human-controlled combatant.
 *
 * Adds:
 *  - Third-person spring arm + camera rig (350 unit boom, 60° pitch, camera lag)
 *  - Equipment, Combat, and Bloodline actor components
 *  - WASD movement, attack/dodge/parry/magic input binding
 *  - Touch/swipe delegation (raw detection lives in AIB4PlayerController;
 *    this class receives resolved EAttackDirection events)
 *  - OnLevelUp BlueprintNativeEvent for bloodline progression feedback
 */
UCLASS(BlueprintType, Blueprintable)
class INFINITYBLADE4_API AIB4PlayerCharacter : public AIB4Character
{
    GENERATED_BODY()

public:

    AIB4PlayerCharacter(const FObjectInitializer& ObjectInitializer);

    virtual void BeginPlay() override;
    virtual void Tick(float DeltaTime) override;
    virtual void SetupPlayerInputComponent(UInputComponent* PlayerInputComponent) override;

    // ---------------------------------------------------------------------------
    // Camera rig
    // ---------------------------------------------------------------------------

    /** Spring arm that positions the camera behind/above the character. */
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Camera")
    USpringArmComponent* CameraBoom;

    /** Primary follow camera. */
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Camera")
    UCameraComponent* FollowCamera;

    // ---------------------------------------------------------------------------
    // Components
    // ---------------------------------------------------------------------------

    /** Manages weapon/shield/armor slots and their stat contributions. */
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Components")
    UIB4EquipmentComponent* EquipmentComponent;

    /** Manages active combo state, parry windows, dodge i-frames. */
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Components")
    UIB4CombatComponent* CombatComponent;

    /** Tracks bloodline progression, manages rebirth and inherited stats. */
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Components")
    UIB4BloodlineComponent* BloodlineComponent;

    // ---------------------------------------------------------------------------
    // Combat input handlers
    // ---------------------------------------------------------------------------

    /** Dispatched by the player controller after swipe direction is resolved. */
    UFUNCTION(BlueprintCallable, Category = "Combat")
    void OnAttackInput(EAttackDirection Direction);

    /** Called when the player triggers a parry (block + counter window). */
    UFUNCTION(BlueprintCallable, Category = "Combat")
    void OnParryInput();

    /** Called when the player triggers a dodge roll. */
    UFUNCTION(BlueprintCallable, Category = "Combat")
    void OnDodgeInput();

    /**
     * Called when the player activates a magic ability.
     * @param Type - The element to cast (Fire / Lightning / Ice).
     */
    UFUNCTION(BlueprintCallable, Category = "Combat")
    void OnMagicInput(EMagicType Type);

    // ---------------------------------------------------------------------------
    // Bloodline progression
    // ---------------------------------------------------------------------------

    /**
     * Fired when BloodlineLevel increments.
     * Override in C++ or implement in Blueprint to play level-up effects.
     */
    UFUNCTION(BlueprintNativeEvent, Category = "Bloodline")
    void OnLevelUp();
    virtual void OnLevelUp_Implementation();

    /** Add XP and check for level threshold. Server-authoritative. */
    UFUNCTION(BlueprintCallable, Category = "Bloodline")
    void AddXP(float Amount);

    /** XP required to advance to the next bloodline level (simple quadratic curve). */
    UFUNCTION(BlueprintPure, Category = "Bloodline")
    float GetXPRequiredForNextLevel() const;

    // ---------------------------------------------------------------------------
    // Death override
    // ---------------------------------------------------------------------------

    virtual void OnDeath_Implementation() override;

protected:

    // ---------------------------------------------------------------------------
    // Movement input
    // ---------------------------------------------------------------------------

    void MoveForward(float Value);
    void MoveRight(float Value);
    void LookUp(float Value);
    void TurnRight(float Value);

    // ---------------------------------------------------------------------------
    // Raw keyboard/gamepad attack bindings (touch is handled in the controller)
    // ---------------------------------------------------------------------------

    void OnAttackOverhead();
    void OnAttackLeft();
    void OnAttackRight();

    /** Magic bindings for keyboard/gamepad fallback. */
    void OnMagicFire();
    void OnMagicLightning();
    void OnMagicIce();

    // ---------------------------------------------------------------------------
    // Config
    // ---------------------------------------------------------------------------

    /** XP scale factor for the level curve: XPRequired = XPBase * (Level ^ XPExponent). */
    UPROPERTY(EditDefaultsOnly, Category = "Bloodline|Config")
    float XPBase;

    UPROPERTY(EditDefaultsOnly, Category = "Bloodline|Config")
    float XPExponent;
};
