// Copyright 1998-2019 Epic Games, Inc. All Rights Reserved.

#include "ShooterGame.h"
#include "Pickups/ShooterPickup_Health.h"
#include "OnlineSubsystemUtils.h"

AShooterPickup_Health::AShooterPickup_Health(const FObjectInitializer& ObjectInitializer) : Super(ObjectInitializer)
{
	Health = 50;
}

bool AShooterPickup_Health::CanBePickedUp(class AShooterCharacter* TestPawn) const
{
	return TestPawn && (TestPawn->Health < TestPawn->GetMaxHealth());
}

void AShooterPickup_Health::GivePickupTo(class AShooterCharacter* Pawn)
{
	if (Pawn)
	{
		Pawn->Health = FMath::Min(FMath::TruncToInt(Pawn->Health) + Health, Pawn->GetMaxHealth());

		// Fire event for collected health
		const UWorld* World = GetWorld();
		const IOnlineEventsPtr Events = Online::GetEventsInterface(World);
		const IOnlineIdentityPtr Identity = Online::GetIdentityInterface(World);

		if (Events.IsValid() && Identity.IsValid())
		{							
			AShooterPlayerController* PC = Cast<AShooterPlayerController>(Pawn->Controller);
			if (PC)
			{
				ULocalPlayer* LocalPlayer = Cast<ULocalPlayer>(PC->Player);

				if (LocalPlayer)
				{
					const int32 UserIndex = LocalPlayer->GetControllerId();
					TSharedPtr<const FUniqueNetId> UniqueID = Identity->GetUniquePlayerId(UserIndex);			
					if (UniqueID.IsValid())
					{
						FVector Location = Pawn->GetActorLocation();

						FOnlineEventParms Params;		

						AShooterGameState* const MyGameState = GetWorld() ? GetWorld()->GetGameState<AShooterGameState>() : nullptr;
						const int32 GameplayModeId = (MyGameState && MyGameState->NumTeams > 0) ? 1 : 0;

						Params.Add( TEXT( "SectionId" ), FVariantData( (int32)0 ) ); // unused
						Params.Add( TEXT( "GameplayModeId" ), FVariantData( GameplayModeId ) );
						Params.Add( TEXT( "DifficultyLevelId" ), FVariantData( (int32)0 ) ); // unused

						Params.Add( TEXT( "ItemId" ), FVariantData( (int32)0 ) ); // health is 0, ammo counts from 1
						Params.Add( TEXT( "AcquisitionMethodId" ), FVariantData( (int32)0 ) ); // unused
						Params.Add( TEXT( "LocationX" ), FVariantData( Location.X ) );
						Params.Add( TEXT( "LocationY" ), FVariantData( Location.Y ) );
						Params.Add( TEXT( "LocationZ" ), FVariantData( Location.Z ) );
						Params.Add( TEXT( "ItemQty" ), FVariantData( (int32)Health ) );			

						Events->TriggerEvent(*UniqueID, TEXT("CollectPowerup"), Params);
					}
				}
			}
		}
	}
}
