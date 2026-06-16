// Copyright 2024 Infinity Blade 4 Project. All Rights Reserved.

#include "Progression/IB4XPSystem.h"
#include "Net/UnrealNetwork.h"
#include "Math/UnrealMathUtility.h"

// ---------------------------------------------------------------------------
// Static helpers
// ---------------------------------------------------------------------------

/*static*/ const TArray<FName>& UIB4XPSystem::GetValidStatNames()
{
    static const TArray<FName> ValidNames =
    {
        FName(TEXT("Health")),
        FName(TEXT("Attack")),
        FName(TEXT("Shield")),
        FName(TEXT("Magic"))
    };
    return ValidNames;
}

/*static*/ int32 UIB4XPSystem::XPForLevel(int32 Level)
{
    // 100 * Level^1.5, rounded to nearest integer.
    return FMath::RoundToInt(100.f * FMath::Pow(static_cast<float>(Level), 1.5f));
}

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

UIB4XPSystem::UIB4XPSystem()
    : CharacterLevel(1)
    , TotalXP(0)
    , XPToNextLevel(0)
    , StatPoints(0)
    , HealthStat(0)
    , AttackStat(0)
    , ShieldStat(0)
    , MagicStat(0)
    , XPInCurrentLevel(0)
{
    PrimaryComponentTick.bCanEverTick = false;
    SetIsReplicatedByDefault(true);

    // Initialise AllocatedStats map with zeroes for all valid stat names.
    for (const FName& Name : GetValidStatNames())
    {
        AllocatedStats.Add(Name, 0);
    }

    RecalculateXPThreshold();
}

// ---------------------------------------------------------------------------
// UActorComponent overrides
// ---------------------------------------------------------------------------

void UIB4XPSystem::BeginPlay()
{
    Super::BeginPlay();
    SyncReplicatedStats();
}

void UIB4XPSystem::GetLifetimeReplicatedProps(TArray<FLifetimeProperty>& OutLifetimeProps) const
{
    Super::GetLifetimeReplicatedProps(OutLifetimeProps);

    DOREPLIFETIME(UIB4XPSystem, HealthStat);
    DOREPLIFETIME(UIB4XPSystem, AttackStat);
    DOREPLIFETIME(UIB4XPSystem, ShieldStat);
    DOREPLIFETIME(UIB4XPSystem, MagicStat);
}

// ---------------------------------------------------------------------------
// AddXP
// ---------------------------------------------------------------------------

void UIB4XPSystem::AddXP(int32 Amount)
{
    if (Amount <= 0)
    {
        return;
    }

    // After reaching the level cap, XP is not accumulated here (caller routes
    // it to equipment mastery instead).
    if (CharacterLevel >= LevelCap)
    {
        return;
    }

    TotalXP += Amount;
    XPInCurrentLevel += Amount;

    // Handle multiple level-ups from a single large XP grant.
    while (XPInCurrentLevel >= XPToNextLevel && CharacterLevel < LevelCap)
    {
        XPInCurrentLevel -= XPToNextLevel;
        PerformLevelUp();
    }

    // If we just hit the cap, discard any leftover within-level XP.
    if (CharacterLevel >= LevelCap)
    {
        XPInCurrentLevel = 0;
    }
}

// ---------------------------------------------------------------------------
// AllocateStatPoint
// ---------------------------------------------------------------------------

bool UIB4XPSystem::AllocateStatPoint(FName StatName)
{
    if (StatPoints <= 0)
    {
        UE_LOG(LogTemp, Warning, TEXT("UIB4XPSystem::AllocateStatPoint — no stat points available."));
        return false;
    }

    if (!AllocatedStats.Contains(StatName))
    {
        UE_LOG(LogTemp, Warning,
               TEXT("UIB4XPSystem::AllocateStatPoint — unknown stat '%s'. Valid: Health, Attack, Shield, Magic."),
               *StatName.ToString());
        return false;
    }

    // Spend the point.
    --StatPoints;
    int32& Current = AllocatedStats[StatName];
    ++Current;

    // Keep replicated properties in sync.
    SyncReplicatedStats();

    // Notify listeners.
    OnStatAllocated.Broadcast(StatName, Current);

    return true;
}

// ---------------------------------------------------------------------------
// Stat-derived values
// ---------------------------------------------------------------------------

int32 UIB4XPSystem::GetMaxHealthForStats() const
{
    return BaseHealth + HealthStat * 60;
}

int32 UIB4XPSystem::GetAttackBonusForStats() const
{
    return AttackStat * 1;
}

int32 UIB4XPSystem::GetMagicBonusForStats() const
{
    return MagicStat * 10;
}

// ---------------------------------------------------------------------------
// Save / Load
// ---------------------------------------------------------------------------

FString UIB4XPSystem::SerialiseToJSON() const
{
    // Build a minimal JSON string without pulling in a full JSON library.
    // Format:
    // {
    //   "Level": 1,
    //   "TotalXP": 0,
    //   "XPInCurrentLevel": 0,
    //   "StatPoints": 0,
    //   "HealthStat": 0,
    //   "AttackStat": 0,
    //   "ShieldStat": 0,
    //   "MagicStat": 0
    // }
    return FString::Printf(
        TEXT("{\"Level\":%d,\"TotalXP\":%d,\"XPInCurrentLevel\":%d,"
             "\"StatPoints\":%d,\"HealthStat\":%d,\"AttackStat\":%d,"
             "\"ShieldStat\":%d,\"MagicStat\":%d}"),
        CharacterLevel,
        TotalXP,
        XPInCurrentLevel,
        StatPoints,
        HealthStat,
        AttackStat,
        ShieldStat,
        MagicStat
    );
}

bool UIB4XPSystem::LoadFromJSON(const FString& JSONString)
{
    if (JSONString.IsEmpty())
    {
        UE_LOG(LogTemp, Warning, TEXT("UIB4XPSystem::LoadFromJSON — empty string provided."));
        return false;
    }

    // Simple key-value extraction without a full JSON parser dependency.
    auto ExtractInt = [&](const FString& Key, int32& OutValue) -> bool
    {
        const FString Search = FString::Printf(TEXT("\"%s\":"), *Key);
        int32 KeyIdx = JSONString.Find(Search, ESearchCase::CaseSensitive);
        if (KeyIdx == INDEX_NONE)
        {
            return false;
        }
        int32 ValueStart = KeyIdx + Search.Len();
        // Skip whitespace
        while (ValueStart < JSONString.Len() && JSONString[ValueStart] == TEXT(' '))
        {
            ++ValueStart;
        }
        // Read digits (and optional leading minus)
        FString NumStr;
        if (ValueStart < JSONString.Len() && JSONString[ValueStart] == TEXT('-'))
        {
            NumStr += TEXT("-");
            ++ValueStart;
        }
        while (ValueStart < JSONString.Len() && FChar::IsDigit(JSONString[ValueStart]))
        {
            NumStr += JSONString[ValueStart];
            ++ValueStart;
        }
        if (NumStr.IsEmpty() || NumStr == TEXT("-"))
        {
            return false;
        }
        OutValue = FCString::Atoi(*NumStr);
        return true;
    };

    int32 LoadedLevel        = CharacterLevel;
    int32 LoadedTotalXP      = TotalXP;
    int32 LoadedXPCurrent    = XPInCurrentLevel;
    int32 LoadedStatPoints   = StatPoints;
    int32 LoadedHealthStat   = HealthStat;
    int32 LoadedAttackStat   = AttackStat;
    int32 LoadedShieldStat   = ShieldStat;
    int32 LoadedMagicStat    = MagicStat;

    bool bOK = true;
    bOK &= ExtractInt(TEXT("Level"),            LoadedLevel);
    bOK &= ExtractInt(TEXT("TotalXP"),          LoadedTotalXP);
    bOK &= ExtractInt(TEXT("XPInCurrentLevel"), LoadedXPCurrent);
    bOK &= ExtractInt(TEXT("StatPoints"),       LoadedStatPoints);
    bOK &= ExtractInt(TEXT("HealthStat"),       LoadedHealthStat);
    bOK &= ExtractInt(TEXT("AttackStat"),       LoadedAttackStat);
    bOK &= ExtractInt(TEXT("ShieldStat"),       LoadedShieldStat);
    bOK &= ExtractInt(TEXT("MagicStat"),        LoadedMagicStat);

    if (!bOK)
    {
        UE_LOG(LogTemp, Error,
               TEXT("UIB4XPSystem::LoadFromJSON — failed to parse one or more fields. JSON: %s"),
               *JSONString);
        return false;
    }

    // Clamp to valid ranges before applying.
    CharacterLevel    = FMath::Clamp(LoadedLevel,     1, LevelCap);
    TotalXP           = FMath::Max(0, LoadedTotalXP);
    XPInCurrentLevel  = FMath::Max(0, LoadedXPCurrent);
    StatPoints        = FMath::Max(0, LoadedStatPoints);
    HealthStat        = FMath::Max(0, LoadedHealthStat);
    AttackStat        = FMath::Max(0, LoadedAttackStat);
    ShieldStat        = FMath::Max(0, LoadedShieldStat);
    MagicStat         = FMath::Max(0, LoadedMagicStat);

    // Rebuild AllocatedStats map from the loaded individual values.
    AllocatedStats[FName(TEXT("Health"))] = HealthStat;
    AllocatedStats[FName(TEXT("Attack"))] = AttackStat;
    AllocatedStats[FName(TEXT("Shield"))] = ShieldStat;
    AllocatedStats[FName(TEXT("Magic"))]  = MagicStat;

    RecalculateXPThreshold();

    return true;
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

void UIB4XPSystem::PerformLevelUp()
{
    ++CharacterLevel;
    StatPoints += StatPointsPerLevel;
    RecalculateXPThreshold();

    UE_LOG(LogTemp, Log, TEXT("UIB4XPSystem: Level up! Now level %d. StatPoints available: %d"),
           CharacterLevel, StatPoints);

    OnLevelUp.Broadcast(CharacterLevel);
}

void UIB4XPSystem::SyncReplicatedStats()
{
    HealthStat = AllocatedStats.FindRef(FName(TEXT("Health")));
    AttackStat = AllocatedStats.FindRef(FName(TEXT("Attack")));
    ShieldStat = AllocatedStats.FindRef(FName(TEXT("Shield")));
    MagicStat  = AllocatedStats.FindRef(FName(TEXT("Magic")));
}

void UIB4XPSystem::RecalculateXPThreshold()
{
    if (CharacterLevel >= LevelCap)
    {
        XPToNextLevel = 0; // No further leveling.
    }
    else
    {
        XPToNextLevel = XPForLevel(CharacterLevel);
    }
}
