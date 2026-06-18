# Getting Started with Unify AutoML and Developer Tools

Welcome to the getting started tutorial for the Unify ecosystem! This tutorial guides you through setting up a local development environment, starting the background server, executing component discovery scans, and running combat balance optimization using the `unify` command-line tool.

By the end of this tutorial, you will have a functional local development layout, a running background manager, and a verified game balancing stat profile.

---

## Prerequisites

Before starting, ensure you have the following installed on your machine:
1. **Rust stable toolchain** (installed via [rustup](https://rustup.rs/))
2. **Node.js 20.x** or higher (required by the background developer server)
3. **Cargo build tools** (standard with your Rust installation)

All terminal commands should be executed from the root of the `rocket-craft` repository:
```bash
cd /Users/sac/rocket-craft
```

---

## Step 1: Initialize the Developer Environment

First, we will initialize a developer environment directory containing default configuration parameters and a test component. The CLI command `dev init` scaffolds this automatically.

Run the following command to initialize the environment in a folder named `./dev_env`:

```bash
cargo run -p unify -- dev init ./dev_env
```

### What This Created
After running the command, check the `./dev_env` directory. You will find:
1. **`dev_config.json`**: The developer environment configuration:
   ```json
   {
     "env": "development",
     "port": 3000,
     "discovery_interval_sec": 5
   }
   ```
2. **`test_component.rs`**: A scaffolded Rust source file containing an auto-binding tag:
   ```rust
   // @UnifyAutoBind: TempComponent
   ```

---

## Step 2: Start the Developer Server

Next, start the background server to handle system requests, API interactions, and telemetry logging. The server uses Node.js and is defined in `genie_server.js`.

To start the server, run the `dev start` command pointing to the environment directory created in Step 1:

```bash
cargo run -p unify -- dev start ./dev_env
```

### What Happens Behind the Scenes
- The `unify` CLI spawns `node genie_server.js` as a background process.
- The process ID (PID) of the server is written to `./dev_env/server.pid`.
- A success message will be displayed in your terminal, showing the server's PID.
- The server will start listening on port `3000` (as configured in `dev_config.json`).

---

## Step 3: Run the Component Discovery Scan

With the server running and our test component in place, we can scan the codebase to discover and register components. The `automl discover` command recursively traverses directory trees, parses files matching extensions `.rs`, `.h`, `.cpp`, or `.hpp`, and registers components matching `@UnifyAutoBind` or `#[derive(AutoBind)]`.

Execute a scan on the `./dev_env` directory to discover the scaffolded component:

```bash
cargo run -p unify -- automl discover ./dev_env
```

### The Output Structure
The output will return a structured JSON response under `data` containing the `ComponentRegistry`:
```json
{
  "success": true,
  "message": "Successfully discovered components in: ./dev_env",
  "data": {
    "components": [
      {
        "name": "TempComponent",
        "file_path": "./dev_env/test_component.rs",
        "language": "Rust",
        "binding_tag": "@UnifyAutoBind: TempComponent"
      }
    ],
    "workspace_games": [
      "Infinity Blade 4 MUD (ib4-mud)",
      "Gundam Nexus (nexus-session)"
    ]
  }
}
```

Notice that `workspace_games` also includes active games discovered from the Chicago TDD Tools workspace!

---

## Step 4: Run Combat Balance Optimization

Now that we have verified our environment and tools, we will use the `automl optimize` command to find the optimal character stat allocation. The optimizer executes Monte Carlo simulations of game combat encounters based on the rules of the `InfinityBladeCoordinateSystem` (from `chicago-tdd-tools`) and a simulated character named `Siris`.

The optimizer evaluates allocations of a given pool of stat points across four attributes:
1. **Health**
2. **Attack**
3. **Defense**
4. **Magic**

We want to distribute **8 points** to get as close as possible to a **60% (0.60) player win rate**, running **20 simulations** per configuration.

Run the optimizer:

```bash
cargo run -p unify -- automl optimize --points 8 --target 0.6 --sims 20
```

### Understanding the Results
The optimizer outputs a JSON summary showing the best allocation found, the achieved win rate, the average number of turns per battle, and the average final health of the player:
```json
{
  "success": true,
  "message": "Optimization complete targeting 60.0% win rate.",
  "data": {
    "allocation": {
      "health": 5,
      "attack": 2,
      "defense": 1,
      "magic": 0
    },
    "player_win_rate": 0.6,
    "avg_turns": 14.2,
    "average_player_final_hp": 85.5
  }
}
```

This tells us that allocating 5 points to health, 2 to attack, 1 to defense, and 0 to magic gets closest to our target 60% win rate under the current combat simulator heuristic rules.

---

## Next Steps
Congratulations! You have completed the getting started tutorial. You have:
- Initialized a developer environment structure.
- Spelled up the developer background server.
- Run a component discovery scan and parsed out bindings.
- Evaluated game balancing via Monte Carlo simulations.

To explore more advanced topics:
- Proceed to the **Combinatorial Testing Tutorial** (`docs/diataxis/tutorials/combinatorial_testing.md`) to explore game state traversal.
- Read the **Auto-Binding How-To Guide** (`docs/diataxis/how_to_guides/auto_binding.md`) to learn how to add custom components to the registry.
