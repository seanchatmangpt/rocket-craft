// Copyright 2024 Infinity Blade 4 Project. All Rights Reserved.

#include "Characters/IB4PlayerController.h"
#include "Characters/IB4PlayerCharacter.h"
#include "Kismet/GameplayStatics.h"

AIB4PlayerController::AIB4PlayerController()
{
    MinSwipeDistance = 50.f;
    bIsTouching      = false;
    TouchStartPosition = FVector2D::ZeroVector;
}

void AIB4PlayerController::BeginPlay()
{
    Super::BeginPlay();

    // Enable touch input so OnInputTouchBegin/End fire on mobile
    bShowMouseCursor = false;
}

void AIB4PlayerController::SetupInputComponent()
{
    Super::SetupInputComponent();
    check(InputComponent);

    // --- Touch gestures ---
    InputComponent->BindTouch(IE_Pressed,  this, &AIB4PlayerController::OnInputTouchBegin);
    InputComponent->BindTouch(IE_Released, this, &AIB4PlayerController::OnInputTouchEnd);

    // --- Menu ---
    InputComponent->BindAction("Pause",            IE_Pressed, this, &AIB4PlayerController::OnPauseGame);
    InputComponent->BindAction("RestartBloodline", IE_Pressed, this, &AIB4PlayerController::OnRestartBloodline);
}

// ---------------------------------------------------------------------------
// Touch / swipe
// ---------------------------------------------------------------------------

void AIB4PlayerController::OnInputTouchBegin(ETouchIndex::Type FingerIndex, FVector Location)
{
    // Only track the primary finger
    if (FingerIndex != ETouchIndex::Touch1)
    {
        return;
    }

    bIsTouching        = true;
    TouchStartPosition = FVector2D(Location.X, Location.Y);
}

void AIB4PlayerController::OnInputTouchEnd(ETouchIndex::Type FingerIndex, FVector Location)
{
    if (FingerIndex != ETouchIndex::Touch1 || !bIsTouching)
    {
        return;
    }

    bIsTouching = false;

    const FVector2D TouchEnd(Location.X, Location.Y);

    bool bIsDodge = false;
    const EAttackDirection Direction = DetectSwipeDirection(TouchStartPosition, TouchEnd, bIsDodge);

    AIB4PlayerCharacter* PlayerChar = GetIB4PlayerCharacter();
    if (!PlayerChar)
    {
        return;
    }

    if (bIsDodge)
    {
        PlayerChar->OnDodgeInput();
    }
    else
    {
        PlayerChar->OnAttackInput(Direction);
    }
}

EAttackDirection AIB4PlayerController::DetectSwipeDirection(FVector2D Start, FVector2D End, bool& bOutIsDodge) const
{
    const FVector2D Delta      = End - Start;
    const float     Distance   = Delta.Size();

    // Short tap → dodge roll
    if (Distance < MinSwipeDistance)
    {
        bOutIsDodge = true;
        // Return a default; caller must check bOutIsDodge before using the direction
        return EAttackDirection::Overhead;
    }

    bOutIsDodge = false;

    // Decompose swipe into dominant axis.
    // Screen coordinates: +X = right, +Y = down.
    const float AbsX = FMath::Abs(Delta.X);
    const float AbsY = FMath::Abs(Delta.Y);

    if (AbsY >= AbsX)
    {
        // Vertical dominant — map both up and down swipes to Overhead.
        // (A downward guard-break swipe and an upward rising slash both
        //  read as Overhead because Infinity Blade 1-3 shared this convention.)
        return EAttackDirection::Overhead;
    }
    else
    {
        // Horizontal dominant
        if (Delta.X > 0.f)
        {
            return EAttackDirection::Right;
        }
        else
        {
            return EAttackDirection::Left;
        }
    }
}

// ---------------------------------------------------------------------------
// Menu / meta actions
// ---------------------------------------------------------------------------

void AIB4PlayerController::OnPauseGame()
{
    // Toggle pause state and ask the game mode to show/hide the pause widget.
    // The actual pause-menu widget is managed in Blueprint; we just flip the flag.
    const bool bCurrentlyPaused = UGameplayStatics::IsGamePaused(GetWorld());
    UGameplayStatics::SetGamePaused(GetWorld(), !bCurrentlyPaused);

    UE_LOG(LogTemp, Log, TEXT("AIB4PlayerController::OnPauseGame — paused=%s"),
           bCurrentlyPaused ? TEXT("false") : TEXT("true"));
}

void AIB4PlayerController::OnRestartBloodline()
{
    // Reload the current level, preserving bloodline data via the GameInstance
    // (GameInstance outlives level loads).  The GameMode saves progress before
    // the level transition; we just trigger the reload here.
    UGameplayStatics::OpenLevel(GetWorld(), *GetWorld()->GetName());

    UE_LOG(LogTemp, Log, TEXT("AIB4PlayerController::OnRestartBloodline — reloading arena"));
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

AIB4PlayerCharacter* AIB4PlayerController::GetIB4PlayerCharacter() const
{
    return Cast<AIB4PlayerCharacter>(GetPawn());
}
