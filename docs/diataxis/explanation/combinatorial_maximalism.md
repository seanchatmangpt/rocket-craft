# Combinatorial Maximalism: State Space Exploration Theory

This document details the theoretical foundation of combinatorial state-space exploration within the Rocket-Craft ecosystem. It explains how complex, stateful game systems are abstractly mapped onto discrete coordinate structures, and how BFS traversal algorithms ("aimbots") perform rigorous state verification.

---

## 1. The Challenge of State Explosion

In modern game engines, the state of a game session is a composite of thousands of variables: player position, velocity, health, items, enemy phase, frame timings, and network sockets. Testing every possible combination of these variables is impossible due to **State Explosion** (where the number of states grows exponentially with the number of variables).

To make state-space exploration computationally feasible, we apply **Combinatorial Maximalism** combined with **State Abstraction**. Instead of traversing the raw memory state, we project it onto a coarse-grained, discrete coordinate representation resembling chess notation.

---

## 2. Chess-Coordinate Representation

A **Chess-Coordinate Representation** is a flat, human-readable, and deterministic string that collapses millions of micro-states into a single macro-coordinate bucket. 

By grouping continuous variables into discrete classes (e.g., categorizing 0-100 HP into `Dead`, `Low`, `Mid`, `Full`), we can represent the game state as a clean coordinate string. If two distinct game states map to the same coordinate string, they are treated as equivalent for the purposes of high-level flow validation.

### Coordinate Schemes in the Ecosystem

#### 1. Infinity Blade 4 MUD Coordinate Scheme
Tracks combat encounters using Siris stats:
- **Format**: `b{bloodline}:{hp_class}:{enemy_id}:{enemy_hp_class}:{enemy_phase}:{announced_attack}:{in_combat}:{combo}`

| Component | Value/Class | Mapping Logic |
|---|---|---|
| `bloodline` | Integer (e.g., `1`, `2`) | Tracks generational progression. |
| `hp_class` | `Dead`, `Low`, `Mid`, `Full` | `Dead` (<= 0%), `Low` (< 25%), `Mid` (< 100%), `Full` (>= 100%). |
| `enemy_id` | e.g. `LT`, `HT`, `DK`, `None` | Shortened Titan IDs (e.g. `LT` = LightTitan, `HT` = HeavyTitan). |
| `enemy_hp_class`| `Dead`, `Low`, `Mid`, `Full`, `None` | Categorized health ratio of the active enemy. |
| `enemy_phase` | e.g., `ep1`, `ep2` | Current phase of the boss fight mechanics. |
| `announced_attack`| `aO`, `aL`, `aR`, `aNone` | Incoming strike direction: Overhead, Left, Right, or None. |
| `in_combat` | `cT`, `cF` | Boolean representing Combat True or Combat False. |
| `combo` | e.g., `cb0`, `cb1`, `cb2` | Cumulative attack chain depth of the player. |

*Example Coordinate*: `b1:Full:LT:Full:ep1:aNone:cT:cb0`

#### 2. Gundam Nexus Coordinate Scheme
Tracks pilot session lifecycle:
- **Format**: `s{state_char}:{match_id_str}:lv{level}:xp{xp}:i{inventory_len}:g{gold}`

| Component | Code | Meaning / Range |
|---|---|---|
| `state_char` | `C`, `A`, `L`, `M`, `S`, `D` | Connecting, Authenticated, InLobby, InMatch, Spectating, Disconnected. |
| `match_id_str` | e.g. `m42`, `m0` | Active match ID (`m0` when not in match/spectating). |
| `level` | Integer | Pilot level. |
| `xp` | Integer | Pilot experience points. |
| `inventory_len` | Integer | Number of items currently equipped (0 to 5). |
| `gold` | Integer | Gold coins owned. |

*Example Coordinate*: `sL:m0:lv1:xp100:i2:g950`

---

## 3. The Aimbot State Traversal Algorithm

The **Aimbot** is a Breadth-First Search (BFS) state explorer. Unlike random play or manual testers, the Aimbot exhaustively maps all reachable states in a deterministic, systematic manner.

### Algorithmic Flow

```text
Initialize Visited HashSet
Initialize Queue containing (InitialState, InitialCoordinate)

While Queue is not empty and Visited.len() < MaxStates:
    Pop (CurrentState, CurrentCoordinate) from Queue
    Query GameCoordinateSystem for legal_moves(CurrentState)
    
    For each Move in legal_moves:
        Apply Move to CurrentState
        If Apply returns Ok(NextState):
            Compute NextCoordinate from NextState
            Record transition (CurrentCoordinate -> Move -> NextCoordinate)
            
            If NextCoordinate is not in Visited:
                Add NextCoordinate to Visited
                Enqueue (NextState, NextCoordinate)
        If Apply returns Err(Error):
            Record transition error details
```

### Why Breadth-First Search (BFS)?
1. **Shortest Path to Defect**: BFS guarantees that when an error state or illegal transition is encountered, the algorithm records the shortest path of moves from the start state. This makes debugging simple, as it provides a minimal reproducing step trace.
2. **Completeness**: BFS explores states layer-by-layer, ensuring that near states are fully cataloged before moving deep. This prevents getting stuck in infinite loops (such as continually buying and selling the same item in a shop).
3. **Deadlock Detection**: If a state is reached where `get_legal_moves` returns an empty list but the state is not a terminal state (e.g., not `Dead` or `Disconnected`), the aimbot marks it as a deadlock defect.
