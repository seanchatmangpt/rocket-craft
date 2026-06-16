# Shoot the Zombie (SurvivalGame)

> **Note:** This project is built and managed using the `./rocket` CLI tool.

## Purpose
An action game titled **Shoot the Zombie**, based on Tom Looman's C++ Survival Sample game series. The project focuses on survival mechanics, combat, and AI behavior in a zombie-infested environment.

## Target Platforms
- **Mobile**: Targeted hardware class.
- **HTML5**: Configured for Shipping builds and distribution.
- **Standard PC**: Supports Windows/Mac.

## Unique Config Settings
- **Hardware Targeting**: Set to **Mobile** with **Scalable** graphics performance.
- **Custom Collision**: Defines specific physical surfaces for detailed hit detection:
  - `PlayerBody`
  - `ZombieBody`
  - `ZombieHead`
  - `ZombieLimb`
- **Android Settings**: Targeted for Min SDK 21; uses `com.nikolalukic.shootthezombie` package name.
- **UI/UX**: 
  - Custom `SLocalPlayer` and `SGameInstance` classes.
  - UI Scale Curve optimized for mobile resolutions.
- **Maps**: 
  - Main Menu: `MainMenu_Entry`
  - Main Level: `ContainerCity_Art`
- **Rendering**: Mobile HDR is disabled to optimize for performance on mobile and web platforms.

## Gameplay C++ Audit: Resolved TODOs
The following 7 codebase TODOs relating to weapon states and damage impulses have been fully audited and resolved:
1. **Weapon Attachment Semantics (`SWeapon.h` / `SWeapon.cpp`)**: Renamed `IsAttachedToPawn` to `IsEquippedOrPending` to accurately reflect its gameplay meaning (returns true if currently equipped or in the process of being equipped).
2. **Server-Side Weapon State Verification (`SWeaponInstant.cpp`)**: Added a check in `ServerNotifyHit_Implementation` to verify `GetCurrentState() == EWeaponState::Firing`. This ensures client hits are only processed while the weapon is in the firing state.
3. **Comprehensive Rejection Logging (`SWeaponInstant.cpp`)**: Added explicit `UE_LOG` warnings for each branch where `ServerNotifyHit_Implementation` rejects a hit (e.g. invalid state, out-of-bounds client tolerance, or viewpoint dot product check failure).
4. **Impact Effect Component Recovery (`SWeaponInstant.cpp`)**: Implemented a short line trace fallback in `SpawnImpactEffects` to retrieve and recover the hit component and physical material when they are lost during replication.
5. **Point Damage Impulse (`SBaseCharacter.cpp`)**: Updated point damage death response to query the default object of the `UDamageType` class and dynamically retrieve `DamageImpulse` rather than utilizing a hardcoded force of 12000.f.
6. **Radial Damage Impulse (`SBaseCharacter.cpp`)**: Updated radial damage death response to query the default object of the `UDamageType` class and dynamically retrieve `DamageImpulse` rather than utilizing a hardcoded force of 100000.f.
7. **Weapon Switch State Gates (`SCharacter.cpp`)**: Added checks in `OnNextWeapon` and `OnPrevWeapon` to prevent weapon cycling while the active weapon is in `EWeaponState::Equipping` or `EWeaponState::Reloading`.

## Gamepad Input Configuration
Gamepad mappings in `Config/DefaultInput.ini` have been audited and verified to be conflict-free:
- **Movement Axes**:
  - `MoveForward` bound to `Gamepad_LeftY` (Scale: 1.0)
  - `MoveRight` bound to `Gamepad_LeftX` (Scale: 1.0)
- **Look Axes**:
  - `Turn` bound to `Gamepad_RightX` (Scale: 1.0)
  - `Lookup` bound to `Gamepad_RightY` (Scale: 1.0)
- **Combat Controls**:
  - `Fire` bound to `Gamepad_RightTrigger`
  - `Targeting` (Aim Down Sights) bound to `Gamepad_LeftTrigger`
- **Actions**:
  - `Jump` bound to `Gamepad_FaceButton_Bottom`
  - `Reload` bound to `Gamepad_FaceButton_Left`
  - `NextWeapon` bound to `Gamepad_FaceButton_Top`
  - `DropWeapon` bound to `Gamepad_FaceButton_Right`
- **World Interaction**:
  - `Use` bound to `Gamepad_LeftShoulder`
  - `PickupObject` bound to `Gamepad_RightShoulder`
- **Stances & Movement Modifiers**:
  - `SprintHold` bound to `Gamepad_LeftThumbstick`
  - `CrouchToggle` bound to `Gamepad_RightThumbstick`

