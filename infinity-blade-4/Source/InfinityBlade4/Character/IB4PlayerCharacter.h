// Copyright 2024 Infinity Blade 4. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "GameFramework/Character.h"
#include "IB4PlayerCharacter.generated.h"

/**
 * AIB4PlayerCharacter
 *
 * The player-controlled character in Infinity Blade 4.
 * Carries the bloodline progression system and can enter
 * a "rebirth" state when killed (health regenerated, XP preserved).
 *
 * This stub exposes the interface consumed by AI classes (IB4GodKingAI).
 * Full implementation lives in IB4PlayerCharacter.cpp.
 */
UCLASS(Blueprintable)
class INFINITYBLADE4_API AIB4PlayerCharacter : public ACharacter
{
    GENERATED_BODY()

public:
    AIB4PlayerCharacter();

    /**
     * Triggers the IB4 rebirth mechanic:
     * - preserves bloodline XP and equipped items
     * - re-initialises health to MaxHealth
     * - plays the rebirth cinematic/particle sequence
     * - respawns the player at the last safe checkpoint
     */
    UFUNCTION(BlueprintCallable, Category = "Player|Rebirth")
    void TriggerRebirth();

    /** Current debuff stacks from the God King's QIP Scar ability */
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Player|Debuffs")
    int32 QIPScarStacks;
};
