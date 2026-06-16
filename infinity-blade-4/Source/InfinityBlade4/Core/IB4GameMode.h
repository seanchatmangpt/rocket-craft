// Copyright 2024 Infinity Blade 4 Project. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "GameFramework/GameModeBase.h"
#include "Core/IB4Types.h"
#include "IB4GameMode.generated.h"

/**
 * AIB4GameMode governs the arena loop for Infinity Blade 4.
 *
 * Responsibilities:
 *  - Track which arena the player is currently fighting in (CurrentArena)
 *  - Spawn titan bosses of the appropriate type for the arena
 *  - React to titan defeat: award XP, advance arena, trigger save
 *  - Persist and restore bloodline progress via SaveGame slots
 */
UCLASS(BlueprintType, Blueprintable)
class INFINITYBLADE4_API AIB4GameMode : public AGameModeBase
{
    GENERATED_BODY()

public:

    AIB4GameMode();

    virtual void BeginPlay() override;

    // ---------------------------------------------------------------------------
    // Arena state
    // ---------------------------------------------------------------------------

    /** Index of the arena the player is currently fighting in (0-based). */
    UPROPERTY(BlueprintReadOnly, Category = "Arena")
    int32 CurrentArena;

    /**
     * Sequence of titan types that guard each arena.
     * The titan at index CurrentArena is spawned when SpawnTitan() is called.
     * Designers set this in the Blueprint subclass defaults.
     */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Arena")
    TArray<ETitanType> ArenaTitanSequence;

    /**
     * Transform used as the spawn origin for the titan in the current arena.
     * Set at BeginPlay from a tag-searched Actor or via designer override.
     */
    UPROPERTY(EditDefaultsOnly, BlueprintReadWrite, Category = "Arena")
    FTransform TitanSpawnTransform;

    // ---------------------------------------------------------------------------
    // Titan management
    // ---------------------------------------------------------------------------

    /**
     * Spawn the titan appropriate for CurrentArena.
     * Returns the newly spawned titan pawn, or nullptr if none could be spawned.
     */
    UFUNCTION(BlueprintCallable, Category = "Arena")
    AActor* SpawnTitan();

    /**
     * Blueprint-callable class map: maps each ETitanType to the BP class to spawn.
     * Designers populate this in the GameMode Blueprint defaults.
     */
    UPROPERTY(EditDefaultsOnly, BlueprintReadWrite, Category = "Arena")
    TMap<ETitanType, TSubclassOf<AActor>> TitanClassMap;

    /**
     * Called by the titan (or its AI controller) when it is defeated.
     * Awards XP to the player, advances the arena index, and saves progress.
     *
     * @param DefeatedTitanType - Titan archetype that was just killed.
     * @param XPReward - Raw XP to grant before player's XP multiplier is applied.
     */
    UFUNCTION(BlueprintCallable, Category = "Arena")
    void OnTitanDefeated(ETitanType DefeatedTitanType, float XPReward);

    // ---------------------------------------------------------------------------
    // Persistence
    // ---------------------------------------------------------------------------

    /**
     * Write current bloodline data (level, XP, stats) to the designated save slot.
     * Called automatically after each titan defeat and on graceful exit.
     */
    UFUNCTION(BlueprintCallable, Category = "Persistence")
    void SaveBloodlineProgress();

    /**
     * Read bloodline data from the save slot and apply it to the player character.
     * Called at BeginPlay so a restarted run inherits prior progression.
     */
    UFUNCTION(BlueprintCallable, Category = "Persistence")
    void LoadBloodlineProgress();

    /** Save slot name used for all bloodline persistence operations. */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Persistence")
    FString BloodlineSaveSlotName;

    /** User index for the save game (typically 0 for single-player). */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Persistence")
    int32 BloodlineSaveUserIndex;

protected:

    /** Reference to the last titan spawned. Cleared on titan defeat. */
    UPROPERTY()
    AActor* ActiveTitan;

    // ---------------------------------------------------------------------------
    // Internal helpers
    // ---------------------------------------------------------------------------

    /** Advance CurrentArena; wraps if the sequence is exhausted (GodKing rerun). */
    void AdvanceArena();

    /** Retrieve the player's AIB4PlayerCharacter from the first player controller. */
    class AIB4PlayerCharacter* GetPlayerCharacter() const;
};
