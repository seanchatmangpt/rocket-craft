// Copyright 1998-2019 Epic Games, Inc. All Rights Reserved.

#include "ShooterGame.h"
#include "Pickups/ShooterPickup_Ammo.h"
#include "Weapons/ShooterWeapon.h"
#include "OnlineSubsystemUtils.h"

AShooterPickup_Ammo::AShooterPickup_Ammo(const FObjectInitializer& ObjectInitializer) : Super(ObjectInitializer)
{
	AmmoClips = 2;
}

bool AShooterPickup_Ammo::IsForWeapon(UClass* WeaponClass)
{
	return WeaponType->IsChildOf(WeaponClass);
}

bool AShooterPickup_Ammo::CanBePickedUp(AShooterCharacter* TestPawn) const
{
	AShooterWeapon* TestWeapon = (TestPawn ? TestPawn->FindWeapon(WeaponType) : NULL);
	if (bIsActive && TestWeapon)
	{
		return TestWeapon->GetCurrentAmmo() < TestWeapon->GetMaxAmmo();
	}

	return false;
}

void AShooterPickup_Ammo::GivePickupTo(class AShooterCharacter* Pawn)
{
	AShooterWeapon* Weapon = (Pawn ? Pawn->FindWeapon(WeaponType) : NULL);
	if (Weapon)
	{
		int32 Qty = AmmoClips * Weapon->GetAmmoPerClip();
		Weapon->GiveAmmo(Qty);

		// Fire event for collected ammo
		if (Pawn)
		{
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

							Params.Add( TEXT( "ItemId" ), FVariantData( (int32)Weapon->GetAmmoType() + 1 ) ); // health is 0, ammo counts from 1
							Params.Add( TEXT( "AcquisitionMethodId" ), FVariantData( (int32)0 ) ); // unused
							Params.Add( TEXT( "LocationX" ), FVariantData( Location.X ) );
							Params.Add( TEXT( "LocationY" ), FVariantData( Location.Y ) );
							Params.Add( TEXT( "LocationZ" ), FVariantData( Location.Z ) );
							Params.Add( TEXT( "ItemQty" ), FVariantData( (int32)Qty ) );		

							Events->TriggerEvent(*UniqueID, TEXT("CollectPowerup"), Params);
						}
					}
				}
			}
		}
	}
}
