// Copyright 2024 Infinity Blade 4 Project. All Rights Reserved.

#include "Progression/IB4NewGamePlus.h"
#include "Progression/IB4BloodlinePerkTree.h"
#include "GameFramework/Actor.h"
#include "Engine/World.h"
#include "Engine/GameInstance.h"
#include "Kismet/GameplayStatics.h"

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

UIB4NewGamePlus::UIB4NewGamePlus()
    : CurrentBloodline(0)
    , PerkPointsAvailable(0)
    , MasteryXPMultiplier(BaseMasteryMultiplier)
    , bIsNegativeBloodline(false)
    , NegativeBloodlineLevel(0)
{
    PrimaryComponentTick.bCanEverTick = false;
}

// ---------------------------------------------------------------------------
// UActorComponent overrides
// ---------------------------------------------------------------------------

void UIB4NewGamePlus::BeginPlay()
{
    Super::BeginPlay();

    // Ensure the mastery multiplier reflects the saved bloodline count on
    // load (e.g. if the game was saved mid-run at bloodline 3, multiplier
    // should be 2^3 = 8.0).  This guard only fires when loading from JSON
    // has not been called yet.
    if (CurrentBloodline > 0 && MasteryXPMultiplier == BaseMasteryMultiplier)
    {
        MasteryXPMultiplier = FMath::Pow(2.f, static_cast<float>(CurrentBloodline));
    }
}

// ---------------------------------------------------------------------------
// TriggerRebirth
// ---------------------------------------------------------------------------

void UIB4NewGamePlus::TriggerRebirth(bool bPlayerDied)
{
    // 1. Advance bloodline counter.
    if (bIsNegativeBloodline)
    {
        NegativeBloodlineLevel = FMath::Max(NegativeBloodlineLevel - 1, NegativeBloodlineMin);

        UE_LOG(LogTemp, Log,
               TEXT("UIB4NewGamePlus: Negative bloodline rebirth. "
                    "NegativeBloodlineLevel = %d. Reason: %s"),
               NegativeBloodlineLevel,
               bPlayerDied ? TEXT("Player died") : TEXT("God King defeated"));
    }
    else
    {
        ++CurrentBloodline;

        UE_LOG(LogTemp, Log,
               TEXT("UIB4NewGamePlus: Rebirth triggered. "
                    "CurrentBloodline = %d. Reason: %s"),
               CurrentBloodline,
               bPlayerDied ? TEXT("Player died") : TEXT("God King defeated"));
    }

    // 2. Reset gold to 0.
    ResetOwnerGold();

    // 3. Clear equipment slots.
    ClearOwnerEquipment();

    // 4. Grant perk point (not in negative mode).
    if (!bIsNegativeBloodline)
    {
        ++PerkPointsAvailable;
        UE_LOG(LogTemp, Log,
               TEXT("UIB4NewGamePlus: Perk point granted. Total available: %d"),
               PerkPointsAvailable);
    }

    // 5. Double the mastery XP multiplier.
    MasteryXPMultiplier *= 2.0f;
    UE_LOG(LogTemp, Log,
           TEXT("UIB4NewGamePlus: MasteryXPMultiplier is now %.2f"),
           MasteryXPMultiplier);

    // 6. Persist level / stats / perks.
    SavePersistentData();

    // 7. Broadcast delegate.
    OnRebirth.Broadcast(CurrentBloodline, bPlayerDied);
}

// ---------------------------------------------------------------------------
// SelectPerk
// ---------------------------------------------------------------------------

bool UIB4NewGamePlus::SelectPerk(FName PerkID)
{
    if (PerkPointsAvailable <= 0)
    {
        UE_LOG(LogTemp, Warning,
               TEXT("UIB4NewGamePlus::SelectPerk — no perk points available."));
        return false;
    }

    if (HasPerk(PerkID))
    {
        UE_LOG(LogTemp, Warning,
               TEXT("UIB4NewGamePlus::SelectPerk — perk '%s' is already unlocked."),
               *PerkID.ToString());
        return false;
    }

    // Validate the perk ID and prerequisites via the perk tree.
    UIB4BloodlinePerkTree* PerkTree = NewObject<UIB4BloodlinePerkTree>(this);
    if (!PerkTree)
    {
        UE_LOG(LogTemp, Error,
               TEXT("UIB4NewGamePlus::SelectPerk — could not create UIB4BloodlinePerkTree."));
        return false;
    }

    FBloodlinePerk PerkData;
    if (!PerkTree->GetPerkByID(PerkID, PerkData))
    {
        UE_LOG(LogTemp, Warning,
               TEXT("UIB4NewGamePlus::SelectPerk — unknown perk '%s'."),
               *PerkID.ToString());
        return false;
    }

    if (!PerkTree->ArePrerequisitesMet(PerkID, UnlockedPerks))
    {
        UE_LOG(LogTemp, Warning,
               TEXT("UIB4NewGamePlus::SelectPerk — prerequisite '%s' not met for perk '%s'."),
               *PerkData.PrerequisiteID.ToString(), *PerkID.ToString());
        return false;
    }

    // All checks passed — unlock the perk.
    --PerkPointsAvailable;
    UnlockedPerks.Add(PerkID);

    UE_LOG(LogTemp, Log,
           TEXT("UIB4NewGamePlus: Perk '%s' unlocked. Remaining perk points: %d"),
           *PerkID.ToString(), PerkPointsAvailable);

    // Persist after unlocking so progress is not lost on crash.
    SavePersistentData();

    return true;
}

// ---------------------------------------------------------------------------
// EnterNegativeBloodline
// ---------------------------------------------------------------------------

void UIB4NewGamePlus::EnterNegativeBloodline()
{
    if (bIsNegativeBloodline)
    {
        UE_LOG(LogTemp, Warning,
               TEXT("UIB4NewGamePlus::EnterNegativeBloodline — already in negative mode."));
        return;
    }

    bIsNegativeBloodline   = true;
    NegativeBloodlineLevel = -1;

    UE_LOG(LogTemp, Log,
           TEXT("UIB4NewGamePlus: Entered negative bloodline mode. "
                "NegativeBloodlineLevel = -1."));
}

// ---------------------------------------------------------------------------
// HasPerk
// ---------------------------------------------------------------------------

bool UIB4NewGamePlus::HasPerk(FName PerkID) const
{
    return UnlockedPerks.Contains(PerkID);
}

// ---------------------------------------------------------------------------
// GetGodKingLevelMultiplier
// ---------------------------------------------------------------------------

float UIB4NewGamePlus::GetGodKingLevelMultiplier() const
{
    // God King level = 50 * (CurrentBloodline + 1).
    return 50.f * static_cast<float>(CurrentBloodline + 1);
}

// ---------------------------------------------------------------------------
// Save / Load
// ---------------------------------------------------------------------------

FString UIB4NewGamePlus::SerialiseToJSON() const
{
    // Serialise UnlockedPerks array as a comma-separated list inside the JSON.
    FString PerksStr;
    for (int32 i = 0; i < UnlockedPerks.Num(); ++i)
    {
        if (i > 0) { PerksStr += TEXT(","); }
        PerksStr += FString::Printf(TEXT("\"%s\""), *UnlockedPerks[i].ToString());
    }

    return FString::Printf(
        TEXT("{\"CurrentBloodline\":%d,\"PerkPointsAvailable\":%d,"
             "\"MasteryXPMultiplier\":%f,\"bIsNegativeBloodline\":%s,"
             "\"NegativeBloodlineLevel\":%d,\"UnlockedPerks\":[%s]}"),
        CurrentBloodline,
        PerkPointsAvailable,
        MasteryXPMultiplier,
        bIsNegativeBloodline ? TEXT("true") : TEXT("false"),
        NegativeBloodlineLevel,
        *PerksStr
    );
}

bool UIB4NewGamePlus::LoadFromJSON(const FString& JSONString)
{
    if (JSONString.IsEmpty())
    {
        UE_LOG(LogTemp, Warning,
               TEXT("UIB4NewGamePlus::LoadFromJSON — empty string provided."));
        return false;
    }

    // --- Extract integer fields ---
    auto ExtractInt = [&](const FString& Key, int32& OutValue) -> bool
    {
        const FString Search = FString::Printf(TEXT("\"%s\":"), *Key);
        int32 KeyIdx = JSONString.Find(Search, ESearchCase::CaseSensitive);
        if (KeyIdx == INDEX_NONE) { return false; }
        int32 Pos = KeyIdx + Search.Len();
        while (Pos < JSONString.Len() && JSONString[Pos] == TEXT(' ')) { ++Pos; }
        FString NumStr;
        if (Pos < JSONString.Len() && JSONString[Pos] == TEXT('-'))
        {
            NumStr += TEXT("-");
            ++Pos;
        }
        while (Pos < JSONString.Len() && FChar::IsDigit(JSONString[Pos]))
        {
            NumStr += JSONString[Pos++];
        }
        if (NumStr.IsEmpty() || NumStr == TEXT("-")) { return false; }
        OutValue = FCString::Atoi(*NumStr);
        return true;
    };

    // --- Extract float fields ---
    auto ExtractFloat = [&](const FString& Key, float& OutValue) -> bool
    {
        const FString Search = FString::Printf(TEXT("\"%s\":"), *Key);
        int32 KeyIdx = JSONString.Find(Search, ESearchCase::CaseSensitive);
        if (KeyIdx == INDEX_NONE) { return false; }
        int32 Pos = KeyIdx + Search.Len();
        while (Pos < JSONString.Len() && JSONString[Pos] == TEXT(' ')) { ++Pos; }
        FString NumStr;
        if (Pos < JSONString.Len() && JSONString[Pos] == TEXT('-'))
        {
            NumStr += TEXT("-");
            ++Pos;
        }
        while (Pos < JSONString.Len() &&
               (FChar::IsDigit(JSONString[Pos]) || JSONString[Pos] == TEXT('.')))
        {
            NumStr += JSONString[Pos++];
        }
        if (NumStr.IsEmpty()) { return false; }
        OutValue = FCString::Atof(*NumStr);
        return true;
    };

    // --- Extract bool fields ---
    auto ExtractBool = [&](const FString& Key, bool& OutValue) -> bool
    {
        const FString Search = FString::Printf(TEXT("\"%s\":"), *Key);
        int32 KeyIdx = JSONString.Find(Search, ESearchCase::CaseSensitive);
        if (KeyIdx == INDEX_NONE) { return false; }
        int32 Pos = KeyIdx + Search.Len();
        while (Pos < JSONString.Len() && JSONString[Pos] == TEXT(' ')) { ++Pos; }
        if (JSONString.Mid(Pos, 4) == TEXT("true"))  { OutValue = true;  return true; }
        if (JSONString.Mid(Pos, 5) == TEXT("false")) { OutValue = false; return true; }
        return false;
    };

    int32 LoadedBloodline       = CurrentBloodline;
    int32 LoadedPerkPoints      = PerkPointsAvailable;
    float LoadedMasteryMult     = MasteryXPMultiplier;
    bool  LoadedNegMode         = bIsNegativeBloodline;
    int32 LoadedNegLevel        = NegativeBloodlineLevel;

    bool bOK = true;
    bOK &= ExtractInt(TEXT("CurrentBloodline"),       LoadedBloodline);
    bOK &= ExtractInt(TEXT("PerkPointsAvailable"),    LoadedPerkPoints);
    bOK &= ExtractFloat(TEXT("MasteryXPMultiplier"),  LoadedMasteryMult);
    bOK &= ExtractBool(TEXT("bIsNegativeBloodline"),  LoadedNegMode);
    bOK &= ExtractInt(TEXT("NegativeBloodlineLevel"), LoadedNegLevel);

    if (!bOK)
    {
        UE_LOG(LogTemp, Error,
               TEXT("UIB4NewGamePlus::LoadFromJSON — failed to parse scalar fields. JSON: %s"),
               *JSONString);
        return false;
    }

    // --- Extract UnlockedPerks array ---
    TArray<FName> LoadedPerks;
    {
        const FString ArrayKey = TEXT("\"UnlockedPerks\":[");
        int32 ArrayStart = JSONString.Find(ArrayKey, ESearchCase::CaseSensitive);
        if (ArrayStart != INDEX_NONE)
        {
            int32 Pos = ArrayStart + ArrayKey.Len();
            // Walk the array until closing bracket.
            while (Pos < JSONString.Len() && JSONString[Pos] != TEXT(']'))
            {
                if (JSONString[Pos] == TEXT('"'))
                {
                    ++Pos; // skip opening quote
                    FString PerkStr;
                    while (Pos < JSONString.Len() && JSONString[Pos] != TEXT('"'))
                    {
                        PerkStr += JSONString[Pos++];
                    }
                    if (!PerkStr.IsEmpty())
                    {
                        LoadedPerks.Add(FName(*PerkStr));
                    }
                }
                ++Pos;
            }
        }
    }

    // Apply all loaded values.
    CurrentBloodline       = FMath::Max(0, LoadedBloodline);
    PerkPointsAvailable    = FMath::Max(0, LoadedPerkPoints);
    MasteryXPMultiplier    = FMath::Max(1.0f, LoadedMasteryMult);
    bIsNegativeBloodline   = LoadedNegMode;
    NegativeBloodlineLevel = FMath::Clamp(LoadedNegLevel, NegativeBloodlineMin, 0);
    UnlockedPerks          = MoveTemp(LoadedPerks);

    UE_LOG(LogTemp, Log,
           TEXT("UIB4NewGamePlus::LoadFromJSON — loaded bloodline %d, "
                "%d perk points, %d unlocked perks."),
           CurrentBloodline, PerkPointsAvailable, UnlockedPerks.Num());

    return true;
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

void UIB4NewGamePlus::ResetOwnerGold()
{
    AActor* Owner = GetOwner();
    if (!Owner)
    {
        UE_LOG(LogTemp, Warning,
               TEXT("UIB4NewGamePlus::ResetOwnerGold — no owner actor."));
        return;
    }

    // Call ResetGoldForRebirth via UFunction if the owner implements it.
    UFunction* Func = Owner->FindFunction(FName(TEXT("ResetGoldForRebirth")));
    if (Func)
    {
        Owner->ProcessEvent(Func, nullptr);
        UE_LOG(LogTemp, Log, TEXT("UIB4NewGamePlus: Gold reset via ResetGoldForRebirth()."));
    }
    else
    {
        UE_LOG(LogTemp, Log,
               TEXT("UIB4NewGamePlus: Owner '%s' does not implement ResetGoldForRebirth — skipping."),
               *Owner->GetName());
    }
}

void UIB4NewGamePlus::ClearOwnerEquipment()
{
    AActor* Owner = GetOwner();
    if (!Owner)
    {
        UE_LOG(LogTemp, Warning,
               TEXT("UIB4NewGamePlus::ClearOwnerEquipment — no owner actor."));
        return;
    }

    UFunction* Func = Owner->FindFunction(FName(TEXT("ClearEquipmentForRebirth")));
    if (Func)
    {
        Owner->ProcessEvent(Func, nullptr);
        UE_LOG(LogTemp, Log, TEXT("UIB4NewGamePlus: Equipment cleared via ClearEquipmentForRebirth()."));
    }
    else
    {
        UE_LOG(LogTemp, Log,
               TEXT("UIB4NewGamePlus: Owner '%s' does not implement ClearEquipmentForRebirth — skipping."),
               *Owner->GetName());
    }
}

void UIB4NewGamePlus::SavePersistentData()
{
    // Persistent stat data is serialised by UIB4XPSystem (on the same actor)
    // and this component's own JSON, then written to a UE4 SaveGame slot via
    // the GameInstance. Implementation of the GameInstance save facade is
    // outside the scope of this component; we log and no-op if unavailable.
    UE_LOG(LogTemp, Log,
           TEXT("UIB4NewGamePlus::SavePersistentData — bloodline %d state marked for save. "
                "(SaveGame facade should serialise UIB4XPSystem + UIB4NewGamePlus JSON.)"),
           CurrentBloodline);
}
