// Copyright 2024 Infinity Blade 4. All Rights Reserved.

#include "Combat/IB4ParrySystem.h"
#include "Kismet/GameplayStatics.h"
#include "Engine/World.h"
#include "TimerManager.h"

//-----------------------------------------------------------------------------
// Construction
//-----------------------------------------------------------------------------

UIB4ParrySystem::UIB4ParrySystem()
    : ParryWindow(0.2f)
    , PerfectParryWindow(0.05f)
    , PerfectParryTimeDilation(0.2f)
    , PerfectParrySlowMoDuration(1.5f)
    , bWindowOpen(false)
    , WindowOpenTime(0.f)
{
}

//-----------------------------------------------------------------------------
// Window Control
//-----------------------------------------------------------------------------

void UIB4ParrySystem::BeginParryWindow()
{
    bWindowOpen    = true;
    WindowOpenTime = GetWorld() ? GetWorld()->GetTimeSeconds() : 0.f;
}

void UIB4ParrySystem::EndParryWindow()
{
    bWindowOpen = false;
}

bool UIB4ParrySystem::IsInParryWindow() const
{
    if (!bWindowOpen)
    {
        return false;
    }

    UWorld* World = GetWorld();
    if (!World)
    {
        return false;
    }

    const float Elapsed = World->GetTimeSeconds() - WindowOpenTime;
    return Elapsed >= 0.f && Elapsed <= ParryWindow;
}

//-----------------------------------------------------------------------------
// EvaluateParry
//-----------------------------------------------------------------------------

EParryResult UIB4ParrySystem::EvaluateParry(EAttackDirection AttackerDir,
                                             EAttackDirection DefenderDir,
                                             float TimeDelta)
{
    // Window was not open at all — complete miss
    if (!bWindowOpen)
    {
        return EParryResult::Miss;
    }

    // Outside the parry window — miss
    if (TimeDelta < 0.f || TimeDelta > ParryWindow)
    {
        return EParryResult::Miss;
    }

    // Clash — defender and attacker both chose the same attack window timing
    // and their directions are directly opposed (e.g. both swinging Right at each other)
    // Simplified clash rule: directions match exactly AND both are in the normal window
    if (AttackerDir == DefenderDir && TimeDelta > PerfectParryWindow)
    {
        return EParryResult::Clash;
    }

    // Perfect parry — hit lands within the tight PerfectParryWindow
    if (TimeDelta <= PerfectParryWindow)
    {
        UWorld* World = GetWorld();
        if (World)
        {
            TriggerBulletTime(World);
        }
        return EParryResult::PerfectParry;
    }

    // Normal parry — hit lands in the remaining portion of the window
    return EParryResult::NormalParry;
}

//-----------------------------------------------------------------------------
// Bullet Time (Perfect Parry)
//-----------------------------------------------------------------------------

void UIB4ParrySystem::TriggerBulletTime(UWorld* World)
{
    if (!World)
    {
        return;
    }

    // Slow everything down to PerfectParryTimeDilation (0.2 = 20% speed)
    UGameplayStatics::SetGlobalTimeDilation(World, PerfectParryTimeDilation);

    // Use a real-time (undilated) timer to restore normal speed
    // TimerManager respects time dilation; we need to scale the duration accordingly.
    // Unreal scales FTimerManager by global time dilation, so divide by dilation to get
    // the correct number of "game ticks" that equal PerfectParrySlowMoDuration real seconds.
    const float ScaledDuration = PerfectParrySlowMoDuration / PerfectParryTimeDilation;

    World->GetTimerManager().SetTimer(
        TimerHandle_SlowMoEnd,
        this,
        &UIB4ParrySystem::RestoreTimeDilation,
        ScaledDuration,
        false
    );
}

void UIB4ParrySystem::RestoreTimeDilation()
{
    UWorld* World = GetWorld();
    if (World)
    {
        UGameplayStatics::SetGlobalTimeDilation(World, 1.0f);
    }
}
