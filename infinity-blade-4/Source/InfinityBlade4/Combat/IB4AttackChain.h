// Copyright 2024 Infinity Blade 4. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "Engine/DataAsset.h"
// EAttackDirection is defined in IB4Types.h — the canonical shared enum source
#include "Core/IB4Types.h"
#include "IB4AttackChain.generated.h"

/**
 * A single node in an attack combo chain.
 * Defines which montage section plays, how much damage it deals relative to
 * the base, how long the player has to input the next attack, and which
 * directions can legally follow.
 */
USTRUCT(BlueprintType)
struct INFINITYBLADE4_API FAttackChainNode
{
    GENERATED_BODY()

    /** Direction this node represents */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Chain")
    EAttackDirection Direction = EAttackDirection::Right;

    /** Anim montage section name to play for this node */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Chain")
    FName MontageSection = NAME_None;

    /**
     * Damage multiplier applied at this chain depth.
     * Typical values: 1.0 → 1.3 → 1.6 → 2.0 as combo deepens.
     */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Chain",
              meta = (ClampMin = "0.1", ClampMax = "5.0"))
    float DamageMultiplier = 1.0f;

    /**
     * Seconds the player has after this swing begins to input the
     * next direction and continue the chain.
     */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Chain",
              meta = (ClampMin = "0.05", ClampMax = "3.0"))
    float WindowDuration = 0.5f;

    /**
     * Set of attack directions that can follow this node to continue the chain.
     * An empty array means the chain terminates here.
     */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Chain")
    TArray<EAttackDirection> ValidFollowUps;
};

/**
 * Data asset that holds the full attack chain sequence for a weapon type.
 *
 * Designers populate the Chain array in the editor. The runtime
 * UIB4AttackChain::GetNextNode() function is called by IB4CombatComponent
 * on each attack to advance the combo tree.
 */
UCLASS(BlueprintType)
class INFINITYBLADE4_API UIB4AttackChain : public UDataAsset
{
    GENERATED_BODY()

public:
    UIB4AttackChain();

    /**
     * Ordered list of attack chain nodes.
     * Index 0 is the first hit; subsequent indices represent deeper combo hits.
     */
    UPROPERTY(EditDefaultsOnly, BlueprintReadOnly, Category = "Chain")
    TArray<FAttackChainNode> Chain;

    /**
     * Given the last executed direction, the incoming direction, and the
     * current combo depth, returns a pointer to the matching FAttackChainNode.
     *
     * Returns nullptr if:
     *   - Depth is out of the chain's range
     *   - NextDir is not listed as a valid follow-up for the node at Depth-1
     *
     * @param LastDir   Direction of the previous attack (ignored at depth 0)
     * @param NextDir   Direction the player is trying to execute
     * @param Depth     Zero-based index: 0 for the first hit in a chain
     */
    UFUNCTION(BlueprintCallable, Category = "Attack Chain")
    FAttackChainNode* GetNextNode(EAttackDirection LastDir,
                                  EAttackDirection NextDir,
                                  int32 Depth);

    /**
     * Returns the damage multiplier for the given chain depth, clamped to
     * the last node if depth exceeds the chain length.
     */
    UFUNCTION(BlueprintCallable, Category = "Attack Chain")
    float GetDamageMultiplierForDepth(int32 Depth) const;

    /** Returns the total number of nodes in the chain */
    UFUNCTION(BlueprintCallable, Category = "Attack Chain")
    int32 GetChainLength() const { return Chain.Num(); }
};
