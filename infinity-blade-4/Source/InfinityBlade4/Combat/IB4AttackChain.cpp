// Copyright 2024 Infinity Blade 4. All Rights Reserved.

#include "Combat/IB4AttackChain.h"

//-----------------------------------------------------------------------------
// Construction — set up a sensible default 4-hit chain
//-----------------------------------------------------------------------------

UIB4AttackChain::UIB4AttackChain()
{
    // Default chain: Right → Left → Right → Up  (can be overridden per weapon in the editor)

    // Hit 1 — entry hit from the right
    {
        FAttackChainNode Node;
        Node.Direction        = EAttackDirection::Right;
        Node.MontageSection   = FName(TEXT("Attack_R_01"));
        Node.DamageMultiplier = 1.0f;
        Node.WindowDuration   = 0.6f;
        Node.ValidFollowUps   = { EAttackDirection::Left, EAttackDirection::Right,
                                  EAttackDirection::Up,   EAttackDirection::Down };
        Chain.Add(Node);
    }

    // Hit 2 — cross-body swing from the left
    {
        FAttackChainNode Node;
        Node.Direction        = EAttackDirection::Left;
        Node.MontageSection   = FName(TEXT("Attack_L_02"));
        Node.DamageMultiplier = 1.3f;
        Node.WindowDuration   = 0.55f;
        Node.ValidFollowUps   = { EAttackDirection::Right, EAttackDirection::Up,
                                  EAttackDirection::Down };
        Chain.Add(Node);
    }

    // Hit 3 — rising cut from the right
    {
        FAttackChainNode Node;
        Node.Direction        = EAttackDirection::Right;
        Node.MontageSection   = FName(TEXT("Attack_R_03"));
        Node.DamageMultiplier = 1.6f;
        Node.WindowDuration   = 0.5f;
        Node.ValidFollowUps   = { EAttackDirection::Up, EAttackDirection::Down };
        Chain.Add(Node);
    }

    // Hit 4 — overhead finisher
    {
        FAttackChainNode Node;
        Node.Direction        = EAttackDirection::Up;
        Node.MontageSection   = FName(TEXT("Attack_U_04"));
        Node.DamageMultiplier = 2.0f;
        Node.WindowDuration   = 0.4f;
        Node.ValidFollowUps   = {}; // Chain terminates here
        Chain.Add(Node);
    }
}

//-----------------------------------------------------------------------------
// GetNextNode
//-----------------------------------------------------------------------------

FAttackChainNode* UIB4AttackChain::GetNextNode(EAttackDirection LastDir,
                                                EAttackDirection NextDir,
                                                int32 Depth)
{
    if (Chain.Num() == 0)
    {
        return nullptr;
    }

    // First hit — any direction is valid, find the first node matching NextDir
    if (Depth == 0)
    {
        for (FAttackChainNode& Node : Chain)
        {
            if (Node.Direction == NextDir)
            {
                return &Node;
            }
        }
        // Fall back to the very first node if no directional match
        return &Chain[0];
    }

    // Subsequent hits — check that the previous node allows NextDir as a follow-up
    const int32 PrevIndex = Depth - 1;
    if (!Chain.IsValidIndex(PrevIndex))
    {
        return nullptr;
    }

    const FAttackChainNode& PrevNode = Chain[PrevIndex];

    // Ensure the requested direction is a valid follow-up
    bool bValidFollowUp = PrevNode.ValidFollowUps.Contains(NextDir);
    if (!bValidFollowUp)
    {
        return nullptr;
    }

    // Find the next node at the target depth that matches the requested direction
    if (Chain.IsValidIndex(Depth))
    {
        // Try the exact index first (chain is ordered by depth)
        if (Chain[Depth].Direction == NextDir)
        {
            return &Chain[Depth];
        }

        // If there are branching chains at the same depth, search forward
        for (int32 i = Depth; i < Chain.Num(); ++i)
        {
            if (Chain[i].Direction == NextDir)
            {
                return &Chain[i];
            }
        }
    }

    return nullptr;
}

//-----------------------------------------------------------------------------
// GetDamageMultiplierForDepth
//-----------------------------------------------------------------------------

float UIB4AttackChain::GetDamageMultiplierForDepth(int32 Depth) const
{
    if (Chain.Num() == 0)
    {
        return 1.0f;
    }

    // Clamp to the last node in the chain
    const int32 ClampedIndex = FMath::Clamp(Depth, 0, Chain.Num() - 1);
    return Chain[ClampedIndex].DamageMultiplier;
}
