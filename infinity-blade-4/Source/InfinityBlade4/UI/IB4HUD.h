// Copyright 2024 Infinity Blade 4 Project. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "GameFramework/HUD.h"
#include "IB4HUD.generated.h"

class UUserWidget;

/**
 * AIB4HUD is the in-game HUD overlay for Infinity Blade 4.
 *
 * Responsibilities:
 *  - Instantiate and manage the UMG widget hierarchy on BeginPlay.
 *  - Draw debug diagnostics in development / debug builds via DrawHUD().
 *  - Expose Blueprint-callable helpers for full-screen transition overlays
 *    (bloodline transition, death screen, verified screen).
 *  - Bridge to the PWA combat HUD by calling into the UMG widget when
 *    combo information changes.
 *
 * The actual visual design lives in the UMG Widget Blueprint assigned to
 * HUDWidgetClass. This class is intentionally thin — it wires up C++ events
 * to Blueprint-friendly entry points and handles the non-UMG debug overlay.
 */
UCLASS(BlueprintType, Blueprintable)
class INFINITYBLADE4_API AIB4HUD : public AHUD
{
    GENERATED_BODY()

public:

    AIB4HUD();

    // -----------------------------------------------------------------------
    // AHUD interface
    // -----------------------------------------------------------------------

    virtual void BeginPlay() override;

    /**
     * DrawHUD is called each frame after all scene rendering is complete.
     * In debug / development builds we render a lightweight text overlay
     * showing combat state, bloodline level, and combo depth.
     * This block compiles out entirely in shipping builds.
     */
    virtual void DrawHUD() override;

    // -----------------------------------------------------------------------
    // UMG Widget
    // -----------------------------------------------------------------------

    /**
     * Widget class to instantiate on BeginPlay.
     * Assign the W_IB4HUD Widget Blueprint in the HUD's class defaults.
     */
    UPROPERTY(EditDefaultsOnly, Category = "HUD|Widget")
    TSubclassOf<UUserWidget> HUDWidgetClass;

    /**
     * Live instance of the main HUD widget, added to the viewport on BeginPlay.
     * Read-only from Blueprint; manipulate via the public UFUNCTION helpers below.
     */
    UPROPERTY(BlueprintReadOnly, Category = "HUD|Widget")
    UUserWidget* HUDWidgetInstance;

    // -----------------------------------------------------------------------
    // Full-screen transition overlays
    // -----------------------------------------------------------------------

    /**
     * Play the bloodline rebirth transition animation.
     * Called after the player character triggers a death→rebirth cycle.
     *
     * @param NewBloodline - The bloodline level that will be active after the transition.
     *                       Passed to the widget so it can display "BLOODLINE X AWAKENED".
     */
    UFUNCTION(BlueprintCallable, Category = "HUD|Transition")
    void ShowBloodlineTransition(int32 NewBloodline);

    /**
     * Display the death / game-over screen overlay.
     * Called by AIB4GameMode when the player has no remaining bloodline retries.
     */
    UFUNCTION(BlueprintCallable, Category = "HUD|Transition")
    void ShowDeathScreen();

    /**
     * Display the verified screen after defeating the current arena's Titan.
     * Fades in the score summary and presents the loot reward panel.
     */
    UFUNCTION(BlueprintCallable, Category = "HUD|Transition")
    void ShowverifiedScreen();

    // -----------------------------------------------------------------------
    // Combo display
    // -----------------------------------------------------------------------

    /**
     * Refresh the combo counter and multiplier readout.
     * Calls the matching Blueprint event on HUDWidgetInstance so the UMG
     * animation can fire without additional polling.
     *
     * Also posts the updated values to the PWA HUD bridge (window.ib4ComboUpdate)
     * via an exec command in development builds.
     *
     * @param ComboCount  - Raw number of hits in the current chain (0 = no combo).
     * @param Multiplier  - Damage multiplier for the current combo depth (e.g. 2.5x).
     */
    UFUNCTION(BlueprintCallable, Category = "HUD|Combo")
    void UpdateComboDisplay(int32 ComboCount, float Multiplier);

protected:

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /**
     * Draw a single line of debug text at the given canvas position.
     * Uses the canvas draw colour set by the caller.
     * Only available in non-shipping builds.
     */
#if !UE_BUILD_SHIPPING
    void DrawDebugLine(const FString& Text, float X, float Y, const FLinearColor& Color = FLinearColor::White);
#endif
};
