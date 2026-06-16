// Copyright 2024 Infinity Blade 4 Project. All Rights Reserved.

#include "Core/IB4GameMode.h"
#include "Characters/IB4PlayerCharacter.h"
#include "Characters/IB4PlayerController.h"
#include "Kismet/GameplayStatics.h"
#include "GameFramework/SaveGame.h"

// ---------------------------------------------------------------------------
// Minimal save-game object (extend with a proper USaveGame subclass later)
// ---------------------------------------------------------------------------

/** Simple in-memory save structure. Replace with a dedicated USaveGame subclass. */
UCLASS()
class UIB4BloodlineSaveGame : public USaveGame
{
    GENERATED_BODY()

public:

    UPROPERTY()
    int32 BloodlineLevel = 0;

    UPROPERTY()
    float CurrentXP = 0.f;

    UPROPERTY()
    FBloodlineStats BloodlineStats;

    UPROPERTY()
    int32 LastArena = 0;
};

// ---------------------------------------------------------------------------

AIB4GameMode::AIB4GameMode()
{
    CurrentArena            = 0;
    ActiveTitan             = nullptr;
    BloodlineSaveSlotName   = TEXT("IB4_Bloodline");
    BloodlineSaveUserIndex  = 0;

    // Default titan sequence — mirrors the classic IB encounter order
    ArenaTitanSequence = {
        ETitanType::Knight,
        ETitanType::Warlord,
        ETitanType::Assassin,
        ETitanType::Berserker,
        ETitanType::Sorcerer,
        ETitanType::Defiler,
        ETitanType::GodKing
    };
}

void AIB4GameMode::BeginPlay()
{
    Super::BeginPlay();

    // Restore bloodline progress so returning players keep their bonuses
    LoadBloodlineProgress();
}

// ---------------------------------------------------------------------------
// Titan management
// ---------------------------------------------------------------------------

AActor* AIB4GameMode::SpawnTitan()
{
    if (ArenaTitanSequence.Num() == 0)
    {
        UE_LOG(LogTemp, Warning, TEXT("AIB4GameMode::SpawnTitan — ArenaTitanSequence is empty"));
        return nullptr;
    }

    const ETitanType TitanType = ArenaTitanSequence[CurrentArena % ArenaTitanSequence.Num()];

    TSubclassOf<AActor>* TitanClassPtr = TitanClassMap.Find(TitanType);
    if (!TitanClassPtr || !(*TitanClassPtr))
    {
        UE_LOG(LogTemp, Warning,
               TEXT("AIB4GameMode::SpawnTitan — no class mapped for TitanType %d. "
                    "Assign it in TitanClassMap inside the GameMode Blueprint."),
               (int32)TitanType);
        return nullptr;
    }

    FActorSpawnParameters SpawnParams;
    SpawnParams.SpawnCollisionHandlingOverride =
        ESpawnActorCollisionHandlingMethod::AdjustIfPossibleButAlwaysSpawn;

    ActiveTitan = GetWorld()->SpawnActor<AActor>(
        *TitanClassPtr,
        TitanSpawnTransform,
        SpawnParams
    );

    if (ActiveTitan)
    {
        UE_LOG(LogTemp, Log, TEXT("AIB4GameMode::SpawnTitan — spawned %s for arena %d"),
               *ActiveTitan->GetName(), CurrentArena);
    }

    return ActiveTitan;
}

void AIB4GameMode::OnTitanDefeated(ETitanType DefeatedTitanType, float XPReward)
{
    UE_LOG(LogTemp, Log, TEXT("AIB4GameMode::OnTitanDefeated — TitanType=%d, XP=%.1f"),
           (int32)DefeatedTitanType, XPReward);

    // Award XP to the player (AddXP applies the bloodline multiplier internally)
    AIB4PlayerCharacter* PlayerChar = GetPlayerCharacter();
    if (PlayerChar)
    {
        PlayerChar->AddXP(XPReward);
    }

    // Clean up the defeated titan reference
    if (ActiveTitan && !ActiveTitan->IsPendingKill())
    {
        ActiveTitan->Destroy();
    }
    ActiveTitan = nullptr;

    // Advance to the next arena and persist
    AdvanceArena();
    SaveBloodlineProgress();
}

void AIB4GameMode::AdvanceArena()
{
    if (ArenaTitanSequence.Num() == 0)
    {
        return;
    }

    CurrentArena = (CurrentArena + 1) % ArenaTitanSequence.Num();

    UE_LOG(LogTemp, Log, TEXT("AIB4GameMode::AdvanceArena — next arena index = %d"), CurrentArena);
}

// ---------------------------------------------------------------------------
// Persistence
// ---------------------------------------------------------------------------

void AIB4GameMode::SaveBloodlineProgress()
{
    AIB4PlayerCharacter* PlayerChar = GetPlayerCharacter();
    if (!PlayerChar)
    {
        UE_LOG(LogTemp, Warning, TEXT("AIB4GameMode::SaveBloodlineProgress — no player character found"));
        return;
    }

    UIB4BloodlineSaveGame* SaveData =
        Cast<UIB4BloodlineSaveGame>(
            UGameplayStatics::CreateSaveGameObject(UIB4BloodlineSaveGame::StaticClass())
        );

    if (!SaveData)
    {
        UE_LOG(LogTemp, Error, TEXT("AIB4GameMode::SaveBloodlineProgress — failed to create SaveGame object"));
        return;
    }

    SaveData->BloodlineLevel = PlayerChar->BloodlineLevel;
    SaveData->CurrentXP      = PlayerChar->CurrentXP;
    SaveData->BloodlineStats = PlayerChar->BloodlineStats;
    SaveData->LastArena      = CurrentArena;

    const bool bSuccess = UGameplayStatics::SaveGameToSlot(
        SaveData, BloodlineSaveSlotName, BloodlineSaveUserIndex);

    if (bSuccess)
    {
        UE_LOG(LogTemp, Log,
               TEXT("AIB4GameMode::SaveBloodlineProgress — saved BloodlineLevel=%d, XP=%.1f, Arena=%d"),
               SaveData->BloodlineLevel, SaveData->CurrentXP, SaveData->LastArena);
    }
    else
    {
        UE_LOG(LogTemp, Error, TEXT("AIB4GameMode::SaveBloodlineProgress — save failed for slot '%s'"),
               *BloodlineSaveSlotName);
    }
}

void AIB4GameMode::LoadBloodlineProgress()
{
    if (!UGameplayStatics::DoesSaveGameExist(BloodlineSaveSlotName, BloodlineSaveUserIndex))
    {
        UE_LOG(LogTemp, Log, TEXT("AIB4GameMode::LoadBloodlineProgress — no save found, starting fresh"));
        return;
    }

    UIB4BloodlineSaveGame* SaveData =
        Cast<UIB4BloodlineSaveGame>(
            UGameplayStatics::LoadGameFromSlot(BloodlineSaveSlotName, BloodlineSaveUserIndex)
        );

    if (!SaveData)
    {
        UE_LOG(LogTemp, Error, TEXT("AIB4GameMode::LoadBloodlineProgress — failed to load SaveGame object"));
        return;
    }

    // Restore arena position
    CurrentArena = FMath::Clamp(SaveData->LastArena, 0,
                                FMath::Max(0, ArenaTitanSequence.Num() - 1));

    // Apply stats to the player character — PostLogin hasn't necessarily run yet,
    // so we defer via a brief timer if the pawn isn't possessed yet.
    AIB4PlayerCharacter* PlayerChar = GetPlayerCharacter();
    if (PlayerChar)
    {
        PlayerChar->BloodlineLevel = SaveData->BloodlineLevel;
        PlayerChar->CurrentXP      = SaveData->CurrentXP;
        PlayerChar->BloodlineStats = SaveData->BloodlineStats;

        UE_LOG(LogTemp, Log,
               TEXT("AIB4GameMode::LoadBloodlineProgress — restored BloodlineLevel=%d, XP=%.1f, Arena=%d"),
               PlayerChar->BloodlineLevel, PlayerChar->CurrentXP, CurrentArena);
    }
    else
    {
        UE_LOG(LogTemp, Warning,
               TEXT("AIB4GameMode::LoadBloodlineProgress — player character not yet possessed; "
                    "bloodline stats will need to be applied on PostLogin"));
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

AIB4PlayerCharacter* AIB4GameMode::GetPlayerCharacter() const
{
    APlayerController* PC = UGameplayStatics::GetPlayerController(GetWorld(), 0);
    if (!PC)
    {
        return nullptr;
    }
    return Cast<AIB4PlayerCharacter>(PC->GetPawn());
}
