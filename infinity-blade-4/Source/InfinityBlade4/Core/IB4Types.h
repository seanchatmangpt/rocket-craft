// Copyright 2024 Infinity Blade 4 Project. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "IB4Types.generated.h"

/**
 * Direction of an attack swing, mapped from touch swipe input.
 * Overhead = downward swipe from above, Left/Right = horizontal swipes.
 */
UENUM(BlueprintType)
enum class EAttackDirection : uint8
{
    Overhead    UMETA(DisplayName = "Overhead"),
    Left        UMETA(DisplayName = "Left"),
    Right       UMETA(DisplayName = "Right")
};

/**
 * Elemental magic types available via bloodline progression.
 */
UENUM(BlueprintType)
enum class EMagicType : uint8
{
    Fire        UMETA(DisplayName = "Fire"),
    Lightning   UMETA(DisplayName = "Lightning"),
    Ice         UMETA(DisplayName = "Ice")
};

/**
 * Equipment slots available on the player character.
 */
UENUM(BlueprintType)
enum class EEquipmentSlot : uint8
{
    Weapon      UMETA(DisplayName = "Weapon"),
    Shield      UMETA(DisplayName = "Shield"),
    Helmet      UMETA(DisplayName = "Helmet"),
    Armor       UMETA(DisplayName = "Armor"),
    Ring        UMETA(DisplayName = "Ring")
};

/**
 * Titan boss archetypes encountered in each arena cycle.
 * GodKing is the final bloodline guardian.
 */
UENUM(BlueprintType)
enum class ETitanType : uint8
{
    Warlord     UMETA(DisplayName = "Warlord"),
    Knight      UMETA(DisplayName = "Knight"),
    Assassin    UMETA(DisplayName = "Assassin"),
    Berserker   UMETA(DisplayName = "Berserker"),
    Sorcerer    UMETA(DisplayName = "Sorcerer"),
    Defiler     UMETA(DisplayName = "Defiler"),
    GodKing     UMETA(DisplayName = "God King")
};

/**
 * Persistent per-bloodline stat bonuses earned through rebirth cycles.
 * All values are additive flat bonuses layered on top of base character stats.
 */
USTRUCT(BlueprintType)
struct FBloodlineStats
{
    GENERATED_BODY()

    FBloodlineStats()
        : AttackBonus(0.f)
        , DefenseBonus(0.f)
        , MagicBonus(0.f)
        , XPMultiplier(1.f)
    {}

    /** Flat bonus added to all physical damage dealt. */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Bloodline")
    float AttackBonus;

    /** Flat damage reduction applied before health deduction. */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Bloodline")
    float DefenseBonus;

    /** Flat bonus added to all magic damage dealt. */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Bloodline")
    float MagicBonus;

    /** Multiplier applied to all XP earned (1.0 = no bonus). */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Bloodline")
    float XPMultiplier;
};
