# How to Configure Monte Carlo Simulations for Game Balancing

This guide provides instructions on how to use the Unify AutoML Combat Balancer to find optimal character stat distributions. The balancer uses Monte Carlo simulations over combat encounters to align player statistics with design balancing goals.

---

## How the Balancer Works

The combat balancer simulates encounters between a player character (`Siris`) and Titan enemies under the rules defined in `InfinityBladeCoordinateSystem` (from the `chicago-tdd-tools` crate).

### Heuristic Combat Logic
During each simulated battle:
1. The character's base stats are recalculated based on a specific point distribution across **Health**, **Attack**, **Defense**, and **Magic**.
2. An `Explore` action is dispatched to initiate a combat encounter.
3. The combat loop runs for up to **100 turns** or until one party's HP reaches zero.
4. **Move Heuristic**:
   - If the enemy has **announced an incoming attack** (overhead, left, or right) and `Parry` is a legal action, the simulator dispatches `Command::Parry` to block.
   - If no attack is announced, the simulator prioritizes a standard strike (`Command::Attack`).
   - If neither condition applies, it falls back to the first available legal move.
5. If the player's health remains above `0.0` at the end of the combat loop, it is counted as a player victory.

---

## Step-by-Step Configuration

To balance the game combat, you will define three parameters:
1. **Stat Points Pool (`--points` / `-p`)**: The total sum of stats to distribute.
2. **Target Win Rate (`--target` / `-t`)**: The desired win rate decimal (e.g., `0.55` for 55%).
3. **Simulations count (`--sims` / `-s`)**: The number of independent battles to run for each stat combination.

### Step 1: Run a Standard Balancing Search
For a quick balance optimization using a pool of **10 points** targeting a **55% player win rate** across **30 simulations** per combination, execute:

```bash
cargo run -p unify -- automl optimize --points 10 --target 0.55 --sims 30
```

### Step 2: Analyze the Recommendations
Inspect the returned JSON. The optimizer iterates through all valid point combinations (nested partitions of the points budget) and returns the configuration that got closest to the target:

```json
{
  "success": true,
  "message": "Optimization complete targeting 55.0% win rate.",
  "data": {
    "allocation": {
      "health": 6,
      "attack": 2,
      "defense": 2,
      "magic": 0
    },
    "player_win_rate": 0.5666666666666667,
    "avg_turns": 16.4,
    "average_player_final_hp": 74.2
  }
}
```

In this scenario:
- The closest win rate found was `56.67%` (diff of `1.67%` from target).
- The recommended stat profile is: Health `6`, Attack `2`, Defense `2`, Magic `0`.
- Battles take an average of `16.4` turns.

### Step 3: Increase Simulation Count for Stable Invariants
Because Monte Carlo simulations rely on random choices, lower simulation runs (like 20 or 30) can produce noisy, non-deterministic recommendations. 

To stabilize the optimization results, increase the simulation count to **100 runs** per configuration:

```bash
cargo run -p unify -- automl optimize --points 10 --target 0.55 --sims 100
```

*Note: Increasing simulations improves statistical confidence but increases optimization time.*

---

## Simulating a Specific Stat Configuration Programmatically

If you are developing a custom balance pipeline, you can call the Rust API directly. Import `unify_automl::balancer` into your Rust test suite or binary:

```rust
use unify_automl::balancer::{simulate_battles, StatAllocation};

fn verify_class_balance() {
    // Define a custom stat allocation
    let tank_profile = StatAllocation {
        health: 8,
        attack: 1,
        defense: 1,
        magic: 0,
    };

    // Run 500 battles to get a highly accurate win rate assessment
    let result = simulate_battles(&tank_profile, 500);

    println!("Tank Win Rate: {:.2}%", result.player_win_rate * 100.0);
    println!("Average turns to complete: {}", result.avg_turns);
    
    assert!(result.player_win_rate > 0.5, "Tank class is underpowered!");
}
```
