# Exploring State Space with the Combinatorial Engine

In this tutorial, you will learn how to set up, execute, and analyze results from the Rocket-Craft Combinatorial State Space Exploration Engine (`combinatorial-engine`). 

This engine is part of the `chicago-tdd-tools` crate. It implements a formal state-traversal algorithm (called the "aimbot") to recursively explore the state space of game sessions, mapping out all possible state transitions.

By the end of this tutorial, you will have compiled the engine, executed a state exploration sweep for two distinct games (`Infinity Blade 4 MUD` and `Gundam Nexus`), and inspected the generated JSON transition report.

---

## Prerequisites

Before starting, ensure you have:
1. **Rust stable toolchain** (installed via [rustup](https://rustup.rs/))
2. Terminal access to the root directory of the workspace:
   ```bash
   cd /Users/sac/rocket-craft
   ```

---

## Step 1: Compile the Combinatorial Engine

First, compile the combinatorial engine binary to ensure all dependencies are correct.

From the root directory, compile the binary using Cargo:

```bash
cargo build -p chicago-tdd-tools --bin combinatorial-engine
```

This compiles the `combinatorial-engine` binary inside your cargo debug build path.

---

## Step 2: Run the State Space Exploration

Now, run the compiled engine with default options. The engine will:
1. Scan the workspace to discover games.
2. Initialize the state systems for both `Infinity Blade 4 MUD` and `Gundam Nexus`.
3. Traverse up to **1,000 states** using the BFS aimbot explorer.
4. Print the traversed transitions to standard output.
5. Save a detailed JSON report to `combinatorial_report.json` in your current working directory.

Run the binary using `cargo run`:

```bash
cargo run -p chicago-tdd-tools --bin combinatorial-engine
```

---

## Step 3: Inspect the Terminal Output

Look at your terminal. The output is structured in five parts:
1. **Game Discovery Logs**: Verifies that both game crates are linked:
   ```text
   Executing discover_games()...
     Discovered game: Infinity Blade 4 MUD (crate: ib4-mud)
     Discovered game: Gundam Nexus (crate: nexus-session)
   ```
2. **Exploration In-Progress Logs**: Tells you the traversal has started.
3. **Infinity Blade 4 MUD Results**: Prints the visited states count, total transitions, and the notation of moves:
   ```text
   Game: Infinity Blade 4 MUD
   Visited States Count: 147
   Transitions Count: 362
   Transitions:
     b1:Full:None:None:ep0:aNone:cF:cb0 --(explore)--> b1:Full:LT:Full:ep1:aNone:cT:cb0
     b1:Full:LT:Full:ep1:aNone:cT:cb0 --(attack:overhead)--> b1:Full:LT:Mid:ep1:aNone:cT:cb1
     ...
   ```
4. **Gundam Nexus Results**: Shows multiplayer session state transitions (Connecting -> Authenticating -> Lobby -> Match):
   ```text
   Game: Gundam Nexus
   Visited States Count: 182
   Transitions Count: 489
   Transitions:
     sC:m0:lv1:xp0:i0:g1001 --(auth:true)--> sA:m0:lv1:xp0:i0:g1001
     sA:m0:lv1:xp0:i0:g1001 --(enter_lobby)--> sL:m0:lv1:xp0:i0:g1001
     sL:m0:lv1:xp0:i0:g1001 --(enter_match:42)--> sM:m42:lv1:xp0:i0:g1001
     sM:m42:lv1:xp0:i0:g1001 --(match_complete)--> sL:m0:lv1:xp0:i0:g1001
     ...
   ```
5. **Report Generation Confirmation**:
   ```text
   Report successfully generated at: "combinatorial_report.json"
   ```

---

## Step 4: Configure a Custom Report Output Path

You can customize where the output report is written. For example, to write the report to `/tmp/combinatorial_run.json`, pass the `--output` (or `-o`) argument.

*Note: In cargo, CLI arguments targeting the binary must come after double dashes `--`.*

Run the command:

```bash
cargo run -p chicago-tdd-tools --bin combinatorial-engine -- --output ./custom_combinatorial_report.json
```

Verify that the file `./custom_combinatorial_report.json` was created.

---

## Step 5: Analyze the JSON Report Structure

Open the generated `combinatorial_report.json` file. The file maps out the full graph structure of both state spaces. 

Here is what the schema looks like:

```json
{
  "total_states_visited": 329,
  "transition_count": 851,
  "transitions": [
    {
      "game": "Infinity Blade 4 MUD",
      "source": "b1:Full:None:None:ep0:aNone:cF:cb0",
      "move": "explore",
      "target": "b1:Full:LT:Full:ep1:aNone:cT:cb0"
    },
    {
      "game": "Gundam Nexus",
      "source": "sC:m0:lv1:xp0:i0:g1001",
      "move": "auth:true",
      "target": "sA:m0:lv1:xp0:i0:g1001"
    }
  ],
  "games": {
    "Gundam Nexus": {
      "visited_states_count": 182,
      "transition_count": 489,
      "errors": []
    },
    "Infinity Blade 4 MUD": {
      "visited_states_count": 147,
      "transition_count": 362,
      "errors": []
    }
  }
}
```

### Explanation of Fields
- **`total_states_visited`**: The sum of all unique visited coordinates across both games.
- **`transitions`**: An array of objects showing how one coordinate maps to another:
  - `game`: The name of the game.
  - `source`: The chess-notation state coordinate before the move was applied.
  - `move`: The notation of the move applied.
  - `target`: The resulting chess-notation state coordinate after applying the move.
- **`games`**: A summary map containing the totals and any state transition error messages caught during BFS traversal.

---

## Next Steps
Now that you have explored the state space with the Combinatorial Engine:
- Read the **Combinatorial Maximalism Explanation** (`docs/diataxis/explanation/combinatorial_maximalism.md`) to learn the mathematical theory of chess-coordinate representations and aimbot BFS traversals.
- Check the **CLI Reference** (`docs/diataxis/reference/cli_commands.md`) for a complete option index.
