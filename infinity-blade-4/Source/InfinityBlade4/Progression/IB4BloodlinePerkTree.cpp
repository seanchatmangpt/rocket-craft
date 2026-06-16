// Copyright 2024 Infinity Blade 4 Project. All Rights Reserved.

#include "Progression/IB4BloodlinePerkTree.h"

// ---------------------------------------------------------------------------
// Constructor — populates all 15 perks
// ---------------------------------------------------------------------------

UIB4BloodlinePerkTree::UIB4BloodlinePerkTree()
{
    AllPerks.Reserve(15);

    // -------------------------------------------------------------------
    // Tier 1 — No prerequisites
    // -------------------------------------------------------------------

    // 1. BloodyResolve — Attack +10%
    AddPerk(
        FName(TEXT("BloodyResolve")),
        NSLOCTEXT("IB4Perks", "BloodyResolve_Name",        "Bloody Resolve"),
        NSLOCTEXT("IB4Perks", "BloodyResolve_Desc",
            "Centuries of bloodshed have sharpened your lineage. Gain +10% attack damage."),
        1, NAME_None,
        EPerkEffectType::AttackBonus, 0.10f
    );

    // 2. IronHide — Defense +10%
    AddPerk(
        FName(TEXT("IronHide")),
        NSLOCTEXT("IB4Perks", "IronHide_Name",             "Iron Hide"),
        NSLOCTEXT("IB4Perks", "IronHide_Desc",
            "Your bloodline has endured countless blows. Gain +10% damage reduction."),
        1, NAME_None,
        EPerkEffectType::DefenseBonus, 0.10f
    );

    // 3. SwiftStrikes — ComboWindow +0.1 s
    AddPerk(
        FName(TEXT("SwiftStrikes")),
        NSLOCTEXT("IB4Perks", "SwiftStrikes_Name",         "Swift Strikes"),
        NSLOCTEXT("IB4Perks", "SwiftStrikes_Desc",
            "Ancestral reflexes extend the combo input window by 0.1 seconds."),
        1, NAME_None,
        EPerkEffectType::ComboWindow, 0.10f
    );

    // 4. MagicSensitivity — Magic +15%
    AddPerk(
        FName(TEXT("MagicSensitivity")),
        NSLOCTEXT("IB4Perks", "MagicSensitivity_Name",     "Magic Sensitivity"),
        NSLOCTEXT("IB4Perks", "MagicSensitivity_Desc",
            "An awakened bloodline resonates with the QIP. Gain +15% magic damage."),
        1, NAME_None,
        EPerkEffectType::MagicBonus, 0.15f
    );

    // 5. Scavenger — GoldFind +20%
    AddPerk(
        FName(TEXT("Scavenger")),
        NSLOCTEXT("IB4Perks", "Scavenger_Name",            "Scavenger"),
        NSLOCTEXT("IB4Perks", "Scavenger_Desc",
            "A survivor's instinct — your lineage knows where enemies hide their gold. +20% gold found."),
        1, NAME_None,
        EPerkEffectType::GoldFind, 0.20f
    );

    // -------------------------------------------------------------------
    // Tier 2 — Each requires a specific Tier-1 perk
    // -------------------------------------------------------------------

    // 6. DeadlyPrecision — CritChance +5%, prereq BloodyResolve
    AddPerk(
        FName(TEXT("DeadlyPrecision")),
        NSLOCTEXT("IB4Perks", "DeadlyPrecision_Name",      "Deadly Precision"),
        NSLOCTEXT("IB4Perks", "DeadlyPrecision_Desc",
            "Your attacks find chinks in every defence. Gain +5% critical hit chance."),
        2, FName(TEXT("BloodyResolve")),
        EPerkEffectType::CritChance, 0.05f
    );

    // 7. FortressStance — HealthBonus +15%, prereq IronHide
    AddPerk(
        FName(TEXT("FortressStance")),
        NSLOCTEXT("IB4Perks", "FortressStance_Name",       "Fortress Stance"),
        NSLOCTEXT("IB4Perks", "FortressStance_Desc",
            "A mountain cannot be moved. Your maximum health is increased by 15%."),
        2, FName(TEXT("IronHide")),
        EPerkEffectType::HealthBonus, 0.15f
    );

    // 8. ComboMaster — ComboWindow +0.15 s + AttackBonus +5%, prereq SwiftStrikes
    AddDualPerk(
        FName(TEXT("ComboMaster")),
        NSLOCTEXT("IB4Perks", "ComboMaster_Name",          "Combo Master"),
        NSLOCTEXT("IB4Perks", "ComboMaster_Desc",
            "Your bloodline has mastered the flow of battle. "
            "Extend the combo window by an additional 0.15 seconds and gain +5% attack damage."),
        2, FName(TEXT("SwiftStrikes")),
        EPerkEffectType::ComboWindow,  0.15f,
        EPerkEffectType::AttackBonus,  0.05f
    );

    // 9. ArcaneChanneling — MagicCostReduction +20%, prereq MagicSensitivity
    AddPerk(
        FName(TEXT("ArcaneChanneling")),
        NSLOCTEXT("IB4Perks", "ArcaneChanneling_Name",     "Arcane Channeling"),
        NSLOCTEXT("IB4Perks", "ArcaneChanneling_Desc",
            "Bloodline resonance reduces the QIP energy required to cast spells by 20%."),
        2, FName(TEXT("MagicSensitivity")),
        EPerkEffectType::MagicCostReduction, 0.20f
    );

    // 10. TreasureHunter — GoldFind +30%, prereq Scavenger
    AddPerk(
        FName(TEXT("TreasureHunter")),
        NSLOCTEXT("IB4Perks", "TreasureHunter_Name",       "Treasure Hunter"),
        NSLOCTEXT("IB4Perks", "TreasureHunter_Desc",
            "Centuries of looting have honed your bloodline's nose for wealth. +30% gold found."),
        2, FName(TEXT("Scavenger")),
        EPerkEffectType::GoldFind, 0.30f
    );

    // -------------------------------------------------------------------
    // Tier 3 — Each requires a specific Tier-2 perk
    // -------------------------------------------------------------------

    // 11. AusarLegacy — AttackBonus +25% + CritChance +10%, prereq DeadlyPrecision
    AddDualPerk(
        FName(TEXT("AusarLegacy")),
        NSLOCTEXT("IB4Perks", "AusarLegacy_Name",          "Ausar's Legacy"),
        NSLOCTEXT("IB4Perks", "AusarLegacy_Desc",
            "Channel the fury of the Deathless King himself. "
            "+25% attack damage and +10% critical hit chance."),
        3, FName(TEXT("DeadlyPrecision")),
        EPerkEffectType::AttackBonus, 0.25f,
        EPerkEffectType::CritChance,  0.10f
    );

    // 12. DeathlessResilience — HealthBonus +30% + DefenseBonus +15%, prereq FortressStance
    AddDualPerk(
        FName(TEXT("DeathlessResilience")),
        NSLOCTEXT("IB4Perks", "DeathlessResilience_Name",  "Deathless Resilience"),
        NSLOCTEXT("IB4Perks", "DeathlessResilience_Desc",
            "Your bloodline defies death itself. +30% maximum health and +15% damage reduction."),
        3, FName(TEXT("FortressStance")),
        EPerkEffectType::HealthBonus,  0.30f,
        EPerkEffectType::DefenseBonus, 0.15f
    );

    // 13. QIPResonance — ParryBonus +0.05 s + ComboWindow +0.2 s, prereq ComboMaster
    AddDualPerk(
        FName(TEXT("QIPResonance")),
        NSLOCTEXT("IB4Perks", "QIPResonance_Name",         "QIP Resonance"),
        NSLOCTEXT("IB4Perks", "QIPResonance_Desc",
            "Perfect harmony with the QIP crystal slows your perception of time. "
            "Parry window +0.05 s and combo window +0.20 s."),
        3, FName(TEXT("ComboMaster")),
        EPerkEffectType::ParryBonus,  0.05f,
        EPerkEffectType::ComboWindow, 0.20f
    );

    // 14. WorkerOfSecretsGift — MagicBonus +40% + MagicCostReduction +30%, prereq ArcaneChanneling
    AddDualPerk(
        FName(TEXT("WorkerOfSecretsGift")),
        NSLOCTEXT("IB4Perks", "WorkerOfSecretsGift_Name",  "Worker of Secrets' Gift"),
        NSLOCTEXT("IB4Perks", "WorkerOfSecretsGift_Desc",
            "The Worker's ancient knowledge flows through your bloodline. "
            "+40% magic damage and −30% magic energy cost."),
        3, FName(TEXT("ArcaneChanneling")),
        EPerkEffectType::MagicBonus,          0.40f,
        EPerkEffectType::MagicCostReduction,  0.30f
    );

    // 15. InfinitySeeker — XPGain +50%, prereq TreasureHunter
    AddPerk(
        FName(TEXT("InfinitySeeker")),
        NSLOCTEXT("IB4Perks", "InfinitySeeker_Name",       "Infinity Seeker"),
        NSLOCTEXT("IB4Perks", "InfinitySeeker_Desc",
            "Your bloodline has transcended the cycle. Earn 50% more experience from all sources."),
        3, FName(TEXT("TreasureHunter")),
        EPerkEffectType::XPGain, 0.50f
    );

    check(AllPerks.Num() == 15); // Ensure all 15 perks are registered.
}

// ---------------------------------------------------------------------------
// Queries
// ---------------------------------------------------------------------------

TArray<FBloodlinePerk> UIB4BloodlinePerkTree::GetTierPerks(int32 Tier) const
{
    TArray<FBloodlinePerk> Result;
    for (const FBloodlinePerk& Perk : AllPerks)
    {
        if (Perk.Tier == Tier)
        {
            Result.Add(Perk);
        }
    }
    return Result;
}

bool UIB4BloodlinePerkTree::ArePrerequisitesMet(FName PerkID,
                                                 const TArray<FName>& UnlockedPerks) const
{
    for (const FBloodlinePerk& Perk : AllPerks)
    {
        if (Perk.PerkID != PerkID)
        {
            continue;
        }

        // Tier-1 perks have no prerequisite.
        if (Perk.PrerequisiteID == NAME_None)
        {
            return true;
        }

        // Check the single required prerequisite.
        return UnlockedPerks.Contains(Perk.PrerequisiteID);
    }

    // Unknown PerkID — cannot be satisfied.
    UE_LOG(LogTemp, Warning,
           TEXT("UIB4BloodlinePerkTree::ArePrerequisitesMet — unknown PerkID '%s'."),
           *PerkID.ToString());
    return false;
}

TArray<FBloodlinePerk> UIB4BloodlinePerkTree::GetAvailablePerks(
    const TArray<FName>& UnlockedPerks) const
{
    TArray<FBloodlinePerk> Result;
    for (const FBloodlinePerk& Perk : AllPerks)
    {
        // Skip already-unlocked perks.
        if (UnlockedPerks.Contains(Perk.PerkID))
        {
            continue;
        }

        // Check prerequisite.
        if (ArePrerequisitesMet(Perk.PerkID, UnlockedPerks))
        {
            Result.Add(Perk);
        }
    }
    return Result;
}

bool UIB4BloodlinePerkTree::GetPerkByID(FName PerkID, FBloodlinePerk& OutPerk) const
{
    for (const FBloodlinePerk& Perk : AllPerks)
    {
        if (Perk.PerkID == PerkID)
        {
            OutPerk = Perk;
            return true;
        }
    }
    return false;
}

// ---------------------------------------------------------------------------
// Private builder helpers
// ---------------------------------------------------------------------------

void UIB4BloodlinePerkTree::AddPerk(FName ID, const FText& Name, const FText& Desc,
                                    int32 Tier, FName PrereqID,
                                    EPerkEffectType EffectType, float EffectValue,
                                    int32 Cost)
{
    FBloodlinePerk Perk;
    Perk.PerkID            = ID;
    Perk.DisplayName       = Name;
    Perk.Description       = Desc;
    Perk.Tier              = Tier;
    Perk.PrerequisiteID    = PrereqID;
    Perk.EffectType        = EffectType;
    Perk.EffectValue       = EffectValue;
    Perk.PointCost         = Cost;
    Perk.bHasSecondaryEffect = false;
    AllPerks.Add(MoveTemp(Perk));
}

void UIB4BloodlinePerkTree::AddDualPerk(FName ID, const FText& Name, const FText& Desc,
                                        int32 Tier, FName PrereqID,
                                        EPerkEffectType PrimaryType,   float PrimaryValue,
                                        EPerkEffectType SecondaryType, float SecondaryValue,
                                        int32 Cost)
{
    FBloodlinePerk Perk;
    Perk.PerkID               = ID;
    Perk.DisplayName          = Name;
    Perk.Description          = Desc;
    Perk.Tier                 = Tier;
    Perk.PrerequisiteID       = PrereqID;
    Perk.EffectType           = PrimaryType;
    Perk.EffectValue          = PrimaryValue;
    Perk.PointCost            = Cost;
    Perk.bHasSecondaryEffect  = true;
    Perk.SecondaryEffectType  = SecondaryType;
    Perk.SecondaryEffectValue = SecondaryValue;
    AllPerks.Add(MoveTemp(Perk));
}
