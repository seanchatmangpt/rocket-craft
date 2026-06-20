// Copyright 2024 Infinity Blade 4 Project. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "UObject/NoExportTypes.h"
#include "InfinityBlade4/Core/IB4Types.h"
#include "IB4EquipmentBase.generated.h"

/**
 * Six-tier rarity system. Rarity governs gem socket count, mastery XP
 * thresholds and base combat stat ceilings.
 */
UENUM(BlueprintType)
enum class EEquipmentRarity : uint8
{
    Common      UMETA(DisplayName = "Common"),
    Uncommon    UMETA(DisplayName = "Uncommon"),
    Rare        UMETA(DisplayName = "Rare"),
    Epic        UMETA(DisplayName = "Epic"),
    Legendary   UMETA(DisplayName = "Legendary"),
    Infinity    UMETA(DisplayName = "Infinity")
};

/**
 * Gem element types. None indicates an empty socket.
 * Mirrors EMagicType but also covers the "no gem" state.
 */
UENUM(BlueprintType)
enum class EGemType : uint8
{
    None        UMETA(DisplayName = "None"),
    Fire        UMETA(DisplayName = "Fire"),
    Ice         UMETA(DisplayName = "Ice"),
    Lightning   UMETA(DisplayName = "Lightning"),
    Dark        UMETA(DisplayName = "Dark"),
    Light       UMETA(DisplayName = "Light")
};

/**
 * Represents a single gem socket on a piece of equipment.
 * BonusValue is the flat stat bonus contributed by the socketed gem (e.g.,
 * +15 fire damage). A socket with SocketedGem == None contributes nothing.
 */
USTRUCT(BlueprintType)
struct FGemSocket
{
    GENERATED_BODY()

    FGemSocket()
        : SocketedGem(EGemType::None)
        , BonusValue(0)
    {}

    /** Which gem (if any) is currently slotted. */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Gem")
    EGemType SocketedGem;

    /** Flat bonus contributed by this gem (attack/magic/defense depending on type). */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Gem")
    int32 BonusValue;
};

/**
 * Combat stats carried by every piece of equipment.
 * XPGainMultiplier stacks multiplicatively with the character's bloodline bonus.
 * GoldGainBonus is a percentage (e.g., 20 == +20%).
 */
USTRUCT(BlueprintType)
struct FEquipmentStats
{
    GENERATED_BODY()

    FEquipmentStats()
        : AttackBonus(0)
        , DefenseBonus(0)
        , MagicBonus(0)
        , XPGainMultiplier(1.0f)
        , GoldGainBonus(0)
    {}

    /** Flat bonus added to outgoing physical damage. */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Stats")
    int32 AttackBonus;

    /** Flat damage reduction applied before health deduction. */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Stats")
    int32 DefenseBonus;

    /** Flat bonus added to outgoing magic damage. */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Stats")
    int32 MagicBonus;

    /** XP gain multiplier for this item (1.0 default, 1.5+ for XP+ items). */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Stats")
    float XPGainMultiplier;

    /** Percentage gold gain bonus contributed by this item (e.g., 20 == +20%). */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Stats")
    int32 GoldGainBonus;
};

/**
 * UIB4EquipmentBase — data-only UObject that represents one piece of equipment.
 *
 * This is NOT an actor. It lives inside the inventory / loadout system and is
 * serialized alongside save-game state. Subclasses (UIB4Sword, UIB4Shield, etc.)
 * extend it with slot-specific properties.
 *
 * Mastery flow:
 *   1. AddMasteryXP() is called by UIB4EquipmentComponent after combat, passing
 *      the slot's share of battle XP (total XP / 5 slots).
 *   2. When MasteryXP reaches MasteryXPRequired the item becomes mastered:
 *      bIsMastered = true, SellValue doubles, and overflow XP is retained.
 *   3. ApplyBloodlineMultiplier() is called on NG+ to scale difficulty.
 */
UCLASS(BlueprintType, Blueprintable)
class INFINITYBLADE4_API UIB4ItemData : public UObject
{
    GENERATED_BODY()

public:
    UIB4ItemData();
};

UCLASS(BlueprintType, Blueprintable)
class INFINITYBLADE4_API UIB4EquipmentBase : public UIB4ItemData
{
    GENERATED_BODY()

public:

    UIB4EquipmentBase();

    // -------------------------------------------------------------------------
    // Identity
    // -------------------------------------------------------------------------

    /** Unique string identifier used to look up this item in data tables. */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Equipment|Identity")
    FName ItemID;

    /** Localised display name shown in menus and tooltips. */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Equipment|Identity")
    FText DisplayName;

    /** Localised flavour / lore text shown in the equipment detail panel. */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Equipment|Identity")
    FText Description;

    /** Which equipment slot this item occupies on the character. */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Equipment|Identity")
    EEquipmentSlot Slot;

    /** Item rarity tier; determines socket count, mastery threshold and costs. */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Equipment|Identity")
    EEquipmentRarity Rarity;

    /** 2-D icon shown in the inventory grid and equipment slots. */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Equipment|Identity")
    UTexture2D* ItemIcon;

    // -------------------------------------------------------------------------
    // Stats
    // -------------------------------------------------------------------------

    /** Base combat stats before gem bonuses are applied. */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Equipment|Stats")
    FEquipmentStats BaseStats;

    // -------------------------------------------------------------------------
    // Gem sockets
    // -------------------------------------------------------------------------

    /**
     * Gem sockets on this item.  Maximum 3 slots; actual count is determined by
     * Rarity: Common=0, Uncommon=1, Rare=1, Epic=2, Legendary=2, Infinity=3.
     * Call InitGemSlots() after setting Rarity to populate with empty sockets.
     */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Equipment|Gems")
    TArray<FGemSocket> GemSlots;

    // -------------------------------------------------------------------------
    // Mastery
    // -------------------------------------------------------------------------

    /** Current accumulated mastery XP. Resets only on prestige/trade. */
    UPROPERTY(BlueprintReadOnly, Category = "Equipment|Mastery")
    int32 MasteryXP;

    /**
     * XP required to reach mastery. Rarity-scaled defaults:
     * Common=500, Uncommon=1000, Rare=2000, Epic=5000, Legendary=10000, Infinity=25000.
     * Doubled each NG+ cycle via ApplyBloodlineMultiplier().
     */
    UPROPERTY(BlueprintReadOnly, Category = "Equipment|Mastery")
    int32 MasteryXPRequired;

    /** True once mastery has been achieved; sell value doubles when true. */
    UPROPERTY(BlueprintReadOnly, Category = "Equipment|Mastery")
    bool bIsMastered;

    /**
     * Bloodline cycle counter used to scale mastery difficulty.
     * 0 = first playthrough, 1 = first NG+, etc.
     */
    UPROPERTY(BlueprintReadOnly, Category = "Equipment|Mastery")
    int32 BloodlineMasteryMultiplier;

    // -------------------------------------------------------------------------
    // Economy
    // -------------------------------------------------------------------------

    /** Gold cost to purchase this item from the shop. */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Equipment|Economy")
    int32 PurchaseCost;

    /**
     * Gold obtained when selling. Equals PurchaseCost/4 normally,
     * PurchaseCost/2 once mastered. Recalculated on mastery achievement.
     */
    UPROPERTY(BlueprintReadOnly, Category = "Equipment|Economy")
    int32 SellValue;

    // -------------------------------------------------------------------------
    // API
    // -------------------------------------------------------------------------

    /**
     * Accumulate mastery XP.  Called once per combat session by the equipment
     * component, passing each slot's equal share of total battle XP.
     * Triggers mastery on overflow; surplus XP is retained.
     */
    UFUNCTION(BlueprintCallable, Category = "Equipment|Mastery")
    void AddMasteryXP(int32 XP);

    /**
     * Socket a gem into the given slot index (0-based).
     * Returns false if the index is out of range, the slot is already occupied,
     * or BonusValue is non-positive.
     */
    UFUNCTION(BlueprintCallable, Category = "Equipment|Gems")
    bool SocketGem(int32 SlotIndex, EGemType GemType, int32 BonusValue);

    /**
     * Return BaseStats with all socketed gem bonuses summed in.
     * Gem type determines which stat is boosted:
     *   Fire/Lightning/Dark -> MagicBonus
     *   Ice               -> DefenseBonus (slow/chill defence)
     *   Light             -> AttackBonus (holy edge)
     */
    UFUNCTION(BlueprintCallable, Category = "Equipment|Stats")
    FEquipmentStats GetTotalStats() const;

    /**
     * Called by the bloodline system on each NG+ cycle.
     * Doubles MasteryXPRequired for every bloodline number > 0, so the
     * difficulty scales: base * 2^BloodlineNumber.
     */
    UFUNCTION(BlueprintCallable, Category = "Equipment|Mastery")
    void ApplyBloodlineMultiplier(int32 BloodlineNumber);

    /**
     * Populate GemSlots with empty sockets according to Rarity.
     * Must be called after construction or after Rarity changes.
     */
    UFUNCTION(BlueprintCallable, Category = "Equipment|Gems")
    void InitGemSlots();

protected:

    /** Called once when mastery threshold is first crossed. */
    virtual void OnMasteryAchieved();

    /** Return the mastery XP threshold for a given rarity. */
    static int32 GetBaseMasteryXPForRarity(EEquipmentRarity InRarity);

    /** Return the number of gem sockets for a given rarity. */
    static int32 GetGemSlotCountForRarity(EEquipmentRarity InRarity);
};
