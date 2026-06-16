// Copyright 2024 Infinity Blade 4 Project. All Rights Reserved.

#include "Characters/IB4PlayerCharacter.h"
#include "Camera/CameraComponent.h"
#include "GameFramework/SpringArmComponent.h"
#include "Components/InputComponent.h"
#include "GameFramework/CharacterMovementComponent.h"

#include "Components/IB4EquipmentComponent.h"
#include "Combat/IB4CombatComponent.h"
#include "Components/IB4BloodlineComponent.h"

AIB4PlayerCharacter::AIB4PlayerCharacter(const FObjectInitializer& ObjectInitializer)
    : Super(ObjectInitializer)
{
    // -----------------------------------------------------------------------
    // Camera rig: 350-unit boom, -60° pitch so we look slightly down at the
    // character (classic IB feel), camera lag to smooth quick turns.
    // -----------------------------------------------------------------------
    CameraBoom = CreateDefaultSubobject<USpringArmComponent>(TEXT("CameraBoom"));
    CameraBoom->SetupAttachment(RootComponent);
    CameraBoom->TargetArmLength         = 350.f;
    CameraBoom->SocketOffset            = FVector(0.f, 0.f, 40.f);  // slight vertical lift
    CameraBoom->bUsePawnControlRotation = true;    // yaw with controller
    CameraBoom->bEnableCameraLag        = true;
    CameraBoom->CameraLagSpeed          = 10.f;
    CameraBoom->bEnableCameraRotationLag = true;
    CameraBoom->CameraRotationLagSpeed  = 8.f;

    // Set the pitch so the camera looks 60° downward toward the player
    CameraBoom->SetRelativeRotation(FRotator(-60.f, 0.f, 0.f));

    FollowCamera = CreateDefaultSubobject<UCameraComponent>(TEXT("FollowCamera"));
    FollowCamera->SetupAttachment(CameraBoom, USpringArmComponent::SocketName);
    FollowCamera->bUsePawnControlRotation = false;  // spring arm handles rotation

    // -----------------------------------------------------------------------
    // Actor components
    // -----------------------------------------------------------------------
    EquipmentComponent  = CreateDefaultSubobject<UIB4EquipmentComponent>(TEXT("EquipmentComponent"));
    CombatComponent     = CreateDefaultSubobject<UIB4CombatComponent>(TEXT("CombatComponent"));
    BloodlineComponent  = CreateDefaultSubobject<UIB4BloodlineComponent>(TEXT("BloodlineComponent"));

    // Player character rotates to face input direction; camera yaw is independent
    bUseControllerRotationYaw = false;
    GetCharacterMovement()->bOrientRotationToMovement = true;

    // -----------------------------------------------------------------------
    // Bloodline XP curve defaults
    // -----------------------------------------------------------------------
    XPBase     = 100.f;
    XPExponent = 1.5f;
}

void AIB4PlayerCharacter::BeginPlay()
{
    Super::BeginPlay();
}

void AIB4PlayerCharacter::Tick(float DeltaTime)
{
    Super::Tick(DeltaTime);
}

// ---------------------------------------------------------------------------
// Input binding
// ---------------------------------------------------------------------------

void AIB4PlayerCharacter::SetupPlayerInputComponent(UInputComponent* PlayerInputComponent)
{
    Super::SetupPlayerInputComponent(PlayerInputComponent);
    check(PlayerInputComponent);

    // --- Movement axes ---
    PlayerInputComponent->BindAxis("MoveForward", this, &AIB4PlayerCharacter::MoveForward);
    PlayerInputComponent->BindAxis("MoveRight",   this, &AIB4PlayerCharacter::MoveRight);
    PlayerInputComponent->BindAxis("LookUp",      this, &AIB4PlayerCharacter::LookUp);
    PlayerInputComponent->BindAxis("TurnRight",   this, &AIB4PlayerCharacter::TurnRight);

    // --- Jump ---
    PlayerInputComponent->BindAction("Jump", IE_Pressed,  this, &ACharacter::Jump);
    PlayerInputComponent->BindAction("Jump", IE_Released, this, &ACharacter::StopJumping);

    // --- Attack directions (keyboard / gamepad fallback; touch goes via controller) ---
    PlayerInputComponent->BindAction("AttackOverhead", IE_Pressed, this, &AIB4PlayerCharacter::OnAttackOverhead);
    PlayerInputComponent->BindAction("AttackLeft",     IE_Pressed, this, &AIB4PlayerCharacter::OnAttackLeft);
    PlayerInputComponent->BindAction("AttackRight",    IE_Pressed, this, &AIB4PlayerCharacter::OnAttackRight);

    // --- Parry & Dodge ---
    PlayerInputComponent->BindAction("Parry", IE_Pressed, this, &AIB4PlayerCharacter::OnParryInput);
    PlayerInputComponent->BindAction("Dodge", IE_Pressed, this, &AIB4PlayerCharacter::OnDodgeInput);

    // --- Magic (keyboard / gamepad fallback) ---
    PlayerInputComponent->BindAction("MagicFire",      IE_Pressed, this, &AIB4PlayerCharacter::OnMagicFire);
    PlayerInputComponent->BindAction("MagicLightning", IE_Pressed, this, &AIB4PlayerCharacter::OnMagicLightning);
    PlayerInputComponent->BindAction("MagicIce",       IE_Pressed, this, &AIB4PlayerCharacter::OnMagicIce);
}

// ---------------------------------------------------------------------------
// Movement
// ---------------------------------------------------------------------------

void AIB4PlayerCharacter::MoveForward(float Value)
{
    if (Controller && Value != 0.f)
    {
        const FRotator Rotation  = Controller->GetControlRotation();
        const FRotator YawOnly   = FRotator(0.f, Rotation.Yaw, 0.f);
        const FVector  Direction = FRotationMatrix(YawOnly).GetUnitAxis(EAxis::X);
        AddMovementInput(Direction, Value);
    }
}

void AIB4PlayerCharacter::MoveRight(float Value)
{
    if (Controller && Value != 0.f)
    {
        const FRotator Rotation  = Controller->GetControlRotation();
        const FRotator YawOnly   = FRotator(0.f, Rotation.Yaw, 0.f);
        const FVector  Direction = FRotationMatrix(YawOnly).GetUnitAxis(EAxis::Y);
        AddMovementInput(Direction, Value);
    }
}

void AIB4PlayerCharacter::LookUp(float Value)
{
    AddControllerPitchInput(Value);
}

void AIB4PlayerCharacter::TurnRight(float Value)
{
    AddControllerYawInput(Value);
}

// ---------------------------------------------------------------------------
// Attack input helpers
// ---------------------------------------------------------------------------

void AIB4PlayerCharacter::OnAttackOverhead()
{
    OnAttackInput(EAttackDirection::Overhead);
}

void AIB4PlayerCharacter::OnAttackLeft()
{
    OnAttackInput(EAttackDirection::Left);
}

void AIB4PlayerCharacter::OnAttackRight()
{
    OnAttackInput(EAttackDirection::Right);
}

void AIB4PlayerCharacter::OnAttackInput(EAttackDirection Direction)
{
    if (!IsAlive())
    {
        return;
    }

    // Delegate to CombatComponent once it is fully implemented.
    // CombatComponent->RequestAttack(Direction);

    UE_LOG(LogTemp, Verbose, TEXT("AIB4PlayerCharacter::OnAttackInput — Direction=%d"), (int32)Direction);
}

void AIB4PlayerCharacter::OnParryInput()
{
    if (!IsAlive())
    {
        return;
    }

    // CombatComponent->RequestParry();

    UE_LOG(LogTemp, Verbose, TEXT("AIB4PlayerCharacter::OnParryInput"));
}

void AIB4PlayerCharacter::OnDodgeInput()
{
    if (!IsAlive())
    {
        return;
    }

    // CombatComponent->RequestDodge();

    UE_LOG(LogTemp, Verbose, TEXT("AIB4PlayerCharacter::OnDodgeInput"));
}

// ---------------------------------------------------------------------------
// Magic input helpers
// ---------------------------------------------------------------------------

void AIB4PlayerCharacter::OnMagicFire()      { OnMagicInput(EMagicType::Fire); }
void AIB4PlayerCharacter::OnMagicLightning() { OnMagicInput(EMagicType::Lightning); }
void AIB4PlayerCharacter::OnMagicIce()       { OnMagicInput(EMagicType::Ice); }

void AIB4PlayerCharacter::OnMagicInput(EMagicType Type)
{
    if (!IsAlive())
    {
        return;
    }

    // Verify enough magic is available before consuming it
    const float SpellCost = 10.f;  // TODO: retrieve from CombatComponent / spell data table
    if (Magic < SpellCost)
    {
        UE_LOG(LogTemp, Warning, TEXT("AIB4PlayerCharacter::OnMagicInput — insufficient magic (%.1f / %.1f)"),
               Magic, SpellCost);
        return;
    }

    Magic = FMath::Clamp(Magic - SpellCost, 0.f, MaxMagic);

    // CombatComponent->CastMagic(Type);

    UE_LOG(LogTemp, Verbose, TEXT("AIB4PlayerCharacter::OnMagicInput — Type=%d"), (int32)Type);
}

// ---------------------------------------------------------------------------
// Bloodline / XP
// ---------------------------------------------------------------------------

float AIB4PlayerCharacter::GetXPRequiredForNextLevel() const
{
    // Quadratic curve: 100 * (Level+1)^1.5
    return XPBase * FMath::Pow(static_cast<float>(BloodlineLevel + 1), XPExponent);
}

void AIB4PlayerCharacter::AddXP(float Amount)
{
    if (Amount <= 0.f)
    {
        return;
    }

    // Apply bloodline XP multiplier from stats
    const float BonusXP = Amount * BloodlineStats.XPMultiplier;
    CurrentXP += BonusXP;

    // Level-up loop in case a single reward crosses multiple thresholds
    while (CurrentXP >= GetXPRequiredForNextLevel())
    {
        CurrentXP -= GetXPRequiredForNextLevel();
        BloodlineLevel++;
        OnLevelUp();
    }
}

void AIB4PlayerCharacter::OnLevelUp_Implementation()
{
    UE_LOG(LogTemp, Log, TEXT("AIB4PlayerCharacter::OnLevelUp — now Bloodline Level %d"), BloodlineLevel);
    // Blueprint override will add particle effects, sound cues, UI pop-ups, etc.
}

// ---------------------------------------------------------------------------
// Death
// ---------------------------------------------------------------------------

void AIB4PlayerCharacter::OnDeath_Implementation()
{
    // Disable player input immediately
    DisableInput(Cast<APlayerController>(GetController()));

    // Base class handles ragdoll, capsule disable, controller detach
    Super::OnDeath_Implementation();

    UE_LOG(LogTemp, Log, TEXT("AIB4PlayerCharacter — player character died at Bloodline Level %d"), BloodlineLevel);
}
