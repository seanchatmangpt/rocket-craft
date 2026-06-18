// Copyright 2024 Infinity Blade 4 Project. All Rights Reserved.

#include "UI/IB4HUD.h"
#include "Characters/IB4PlayerCharacter.h"
#include "Combat/IB4CombatComponent.h"
#include "Blueprint/UserWidget.h"
#include "Engine/Canvas.h"
#include "Engine/Engine.h"

// -----------------------------------------------------------------------
// Construction
// -----------------------------------------------------------------------

AIB4HUD::AIB4HUD()
    : HUDWidgetClass(nullptr)
    , HUDWidgetInstance(nullptr)
{
}

// -----------------------------------------------------------------------
// AHUD interface
// -----------------------------------------------------------------------

void AIB4HUD::BeginPlay()
{
    Super::BeginPlay();

    // Instantiate and display the main HUD widget if a class has been assigned
    // in the class defaults (via the HUD Blueprint or Project Settings → Maps & Modes).
    if (HUDWidgetClass)
    {
        APlayerController* PC = GetOwningPlayerController();
        if (PC)
        {
            HUDWidgetInstance = CreateWidget<UUserWidget>(PC, HUDWidgetClass);
            if (HUDWidgetInstance)
            {
                // ZOrder 0 — base layer; transition overlays sit on top via separate widgets.
                HUDWidgetInstance->AddToViewport(0);
            }
            else
            {
                UE_LOG(LogHUD, Warning,
                    TEXT("AIB4HUD::BeginPlay — CreateWidget returned null for HUDWidgetClass '%s'."),
                    *HUDWidgetClass->GetName());
            }
        }
    }
    else
    {
        UE_LOG(LogHUD, Warning,
            TEXT("AIB4HUD::BeginPlay — HUDWidgetClass is not set. "
                 "Assign W_IB4HUD in the HUD class defaults."));
    }
}

void AIB4HUD::DrawHUD()
{
    Super::DrawHUD();

    // Debug overlay is compiled out entirely in shipping builds.
#if !UE_BUILD_SHIPPING
    if (!Canvas)
    {
        return;
    }

    // Obtain the owning player character and its combat component.
    APlayerController* PC = GetOwningPlayerController();
    if (!PC)
    {
        return;
    }

    APawn* Pawn = PC->GetPawn();
    AIB4PlayerCharacter* PlayerChar = Cast<AIB4PlayerCharacter>(Pawn);
    if (!PlayerChar)
    {
        return;
    }

    UIB4CombatComponent* CombatComp = Cast<UIB4CombatComponent>(
        PlayerChar->GetComponentByClass(UIB4CombatComponent::StaticClass()));

    // Layout constants — top-left corner debug panel.
    const float PanelX   = 20.f;
    float       PanelY   = 80.f;  // incremented per line
    const float LineStep = 18.f;

    // Section header
    DrawDebugLine(TEXT("=== IB4 DEBUG HUD ==="), PanelX, PanelY, FLinearColor::Yellow);
    PanelY += LineStep;

    // Health / Magic bars
    float HealthPct = (PlayerChar->MaxHealth > 0.f)
        ? (PlayerChar->Health / PlayerChar->MaxHealth) * 100.f : 0.f;
    float MagicPct = (PlayerChar->MaxMagic > 0.f)
        ? (PlayerChar->Magic / PlayerChar->MaxMagic) * 100.f : 0.f;

    DrawDebugLine(
        FString::Printf(TEXT("HP: %.0f / %.0f  (%.1f%%)"),
            PlayerChar->Health, PlayerChar->MaxHealth, HealthPct),
        PanelX, PanelY,
        HealthPct > 50.f ? FLinearColor::Green : FLinearColor::Red);
    PanelY += LineStep;

    DrawDebugLine(
        FString::Printf(TEXT("MP: %.0f / %.0f  (%.1f%%)"),
            PlayerChar->Magic, PlayerChar->MaxMagic, MagicPct),
        PanelX, PanelY, FLinearColor(0.3f, 0.5f, 1.f));
    PanelY += LineStep;

    // Bloodline level
    DrawDebugLine(
        FString::Printf(TEXT("Bloodline Level: %d"), PlayerChar->BloodlineLevel),
        PanelX, PanelY, FLinearColor::Yellow);
    PanelY += LineStep;

    // XP
    DrawDebugLine(
        FString::Printf(TEXT("XP: %.0f"), PlayerChar->CurrentXP),
        PanelX, PanelY);
    PanelY += LineStep;

    // Combat state
    if (CombatComp)
    {
        static const TMap<ECombatState, FString> StateNames =
        {
            { ECombatState::Idle,      TEXT("Idle")      },
            { ECombatState::Attacking, TEXT("Attacking") },
            { ECombatState::Parrying,  TEXT("Parrying")  },
            { ECombatState::Dodging,   TEXT("Dodging")   },
            { ECombatState::Stunned,   TEXT("Stunned")   },
            { ECombatState::Dead,      TEXT("Dead")      },
        };

        const FString* StateName = StateNames.Find(CombatComp->GetCurrentState());
        DrawDebugLine(
            FString::Printf(TEXT("Combat State: %s"),
                StateName ? **StateName : TEXT("Unknown")),
            PanelX, PanelY, FLinearColor::Cyan);
        PanelY += LineStep;

        // Combo info
        float ComboMult = CombatComp->GetComboMultiplier();
        DrawDebugLine(
            FString::Printf(TEXT("Combo: %d  (x%.1f)"),
                CombatComp->GetComboCount(), ComboMult),
            PanelX, PanelY, FLinearColor::White);
        PanelY += LineStep;
    }
#endif // !UE_BUILD_SHIPPING
}

// -----------------------------------------------------------------------
// Full-screen transitions
// -----------------------------------------------------------------------

void AIB4HUD::ShowBloodlineTransition(int32 NewBloodline)
{
    if (!HUDWidgetInstance)
    {
        UE_LOG(LogHUD, Warning,
            TEXT("AIB4HUD::ShowBloodlineTransition — HUDWidgetInstance is null."));
        return;
    }

    // Call the Blueprint event on the widget.
    // The Widget Blueprint must implement the "OnBloodlineTransition" event.
    // We use the function-call pattern so the widget can run its timeline animation.
    FName FuncName(TEXT("OnBloodlineTransition"));
    if (HUDWidgetInstance->FindFunction(FuncName))
    {
        struct FBloodlineTransitionParams { int32 NewBloodlineLevel; };
        FBloodlineTransitionParams Params;
        Params.NewBloodlineLevel = NewBloodline;
        HUDWidgetInstance->ProcessEvent(
            HUDWidgetInstance->FindFunction(FuncName), &Params);
    }
    else
    {
        // Fallback: print a screen message in dev builds.
        UE_LOG(LogHUD, Log,
            TEXT("AIB4HUD::ShowBloodlineTransition — Bloodline %d awakened!"), NewBloodline);
#if !UE_BUILD_SHIPPING
        if (GEngine)
        {
            GEngine->AddOnScreenDebugMessage(-1, 5.f, FColor::Yellow,
                FString::Printf(TEXT("BLOODLINE %d AWAKENED"), NewBloodline));
        }
#endif
    }
}

void AIB4HUD::ShowDeathScreen()
{
    if (!HUDWidgetInstance)
    {
        UE_LOG(LogHUD, Warning, TEXT("AIB4HUD::ShowDeathScreen — HUDWidgetInstance is null."));
        return;
    }

    FName FuncName(TEXT("OnDeathScreen"));
    if (HUDWidgetInstance->FindFunction(FuncName))
    {
        HUDWidgetInstance->ProcessEvent(
            HUDWidgetInstance->FindFunction(FuncName), nullptr);
    }
    else
    {
        UE_LOG(LogHUD, Log, TEXT("AIB4HUD::ShowDeathScreen — player defeated."));
#if !UE_BUILD_SHIPPING
        if (GEngine)
        {
            GEngine->AddOnScreenDebugMessage(-1, 5.f, FColor::Red,
                TEXT("YOU HAVE FALLEN"));
        }
#endif
    }
}

void AIB4HUD::ShowverifiedScreen()
{
    if (!HUDWidgetInstance)
    {
        UE_LOG(LogHUD, Warning, TEXT("AIB4HUD::ShowverifiedScreen — HUDWidgetInstance is null."));
        return;
    }

    FName FuncName(TEXT("OnverifiedScreen"));
    if (HUDWidgetInstance->FindFunction(FuncName))
    {
        HUDWidgetInstance->ProcessEvent(
            HUDWidgetInstance->FindFunction(FuncName), nullptr);
    }
    else
    {
        UE_LOG(LogHUD, Log, TEXT("AIB4HUD::ShowverifiedScreen — player victorious."));
#if !UE_BUILD_SHIPPING
        if (GEngine)
        {
            GEngine->AddOnScreenDebugMessage(-1, 5.f, FColor::Green,
                TEXT("verified"));
        }
#endif
    }
}

// -----------------------------------------------------------------------
// Combo display
// -----------------------------------------------------------------------

void AIB4HUD::UpdateComboDisplay(int32 ComboCount, float Multiplier)
{
    // Push to the UMG widget.
    if (HUDWidgetInstance)
    {
        FName FuncName(TEXT("OnComboUpdate"));
        if (HUDWidgetInstance->FindFunction(FuncName))
        {
            struct FComboUpdateParams { int32 Count; float Mult; };
            FComboUpdateParams Params;
            Params.Count = ComboCount;
            Params.Mult  = Multiplier;
            HUDWidgetInstance->ProcessEvent(
                HUDWidgetInstance->FindFunction(FuncName), &Params);
        }
    }

    // In development builds, broadcast to the PWA HUD bridge via a console command.
    // The PWA side listens for window.ib4ComboUpdate events injected by the WebView.
#if !UE_BUILD_SHIPPING
    UE_LOG(LogHUD, Verbose,
        TEXT("AIB4HUD::UpdateComboDisplay — Combo=%d  Multiplier=x%.2f"),
        ComboCount, Multiplier);
#endif
}

// -----------------------------------------------------------------------
// Internal helpers
// -----------------------------------------------------------------------

#if !UE_BUILD_SHIPPING
void AIB4HUD::DrawDebugLine(const FString& Text, float X, float Y, const FLinearColor& Color)
{
    if (!Canvas)
    {
        return;
    }

    FCanvasTextItem TextItem(FVector2D(X, Y), FText::FromString(Text),
        GEngine ? GEngine->GetMediumFont() : nullptr, Color);
    TextItem.EnableShadow(FLinearColor::Black);
    Canvas->DrawItem(TextItem);
}
#endif
