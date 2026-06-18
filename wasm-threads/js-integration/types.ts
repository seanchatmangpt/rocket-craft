/**
 * types.ts — Hand-crafted TypeScript declarations mirroring all #[wasm_bindgen]
 * and serde-serialised types from the wasm-threads Rust crates.
 *
 * JSON shapes follow serde's default enum representation:
 *   - Unit variants → bare string, e.g. "Pause"
 *   - Newtype variants → { "VariantName": <inner> }
 *   - Struct variants → { "VariantName": { field: value, ... } }
 */

// ── GameToUiMessage ──────────────────────────────────────────────────────────
// Sent FROM wasm-game-logic worker TO the UI (wasm-ui / main thread).
// Source: wasm-game-logic/src/protocol.rs

export type GameToUiMessage =
  | {
      StateUpdate: {
        tick: number;
        entity_count: number;
        /** null when no player entity exists */
        player_health: number | null;
        /** null when no player entity exists */
        player_health_max: number | null;
        player_score: number;
      };
    }
  | { GameOver: { winner_score: number; total_ticks: number } }
  | { EntityMoved: { entity_id: number; x: number; y: number } }
  | { EntityDied: { entity_id: number } };

// ── UiToGameMessage / InputCommand ───────────────────────────────────────────
// Sent FROM the UI TO the game-logic worker.
// Source: wasm-ui/src/message_bridge.rs (mirrors wasm-game-logic/src/protocol.rs)

export type InputCommand =
  | { Move: { entity: number; dx: number; dy: number } }
  | { Attack: { attacker: number; target: number } }
  | { UseItem: { entity: number; item_id: number } };

/**
 * UiToGameMessage — serde-serialised shapes:
 *   - Input(InputCommand) → { "Input": <InputCommand> }
 *   - Pause              → "Pause"
 *   - Resume             → "Resume"
 *   - Restart            → "Restart"
 *   - Ping { seq }       → { "Ping": { "seq": number } }
 */
export type UiToGameMessage =
  | { Input: InputCommand }
  | 'Pause'
  | 'Resume'
  | 'Restart'
  | { Ping: { seq: number } };

// ── FromGameMessage ──────────────────────────────────────────────────────────
// Replies from the game-logic worker that are not UI state updates.
// Source: wasm-game-logic/src/protocol.rs

export type FromGameMessage = { Pong: { seq: number } };

// ── HudData ──────────────────────────────────────────────────────────────────
// Returned by UiController.on_game_message() after processing a StateUpdate.
// Source: wasm-ui/src/hud.rs

export interface HudData {
  player_health: number;
  player_health_max: number;
  score: number;
  entity_count: number;
  game_tick: number;
  /** Frames per second — computed by the caller; the Rust side emits 0.0 */
  fps: number;
  /** Messages processed per second — computed by the caller; Rust emits 0.0 */
  messages_per_second: number;
}

// ── ThreadingApproach ────────────────────────────────────────────────────────
// Mirrors wasm-core/src/approach.rs for WasmThreadContext construction.
// The Rust side exposes a flat constructor (new_separate_modules); this type
// is used by the JS bridge layer (GameBridgeOptions) to express intent.

export type ThreadingApproach =
  | { SeparateModules: { worker_count: number } }
  | { SharedMemory: { buffer_size_bytes: number } }
  | { Hybrid: { worker_count: number; shared_buffer_size_bytes: number } };

// ── Wasm class stubs ─────────────────────────────────────────────────────────
// Type-only declarations for what wasm-pack would generate.
// In production code, import from the compiled pkg/ directory instead.

/**
 * GameLogicWorker — wasm-game-logic/src/lib.rs
 *
 * Manages the game loop inside a Web Worker. All state is held in the WASM
 * module; communication with the main thread uses JSON-encoded postMessage.
 */
export interface GameLogicWorker {
  /** Spawn the player entity and set running = true. */
  start(): void;
  /**
   * Advance the simulation by delta_ms milliseconds.
   * Returns JSON-encoded GameToUiMessage::StateUpdate, or "{}" when paused.
   */
  tick_js(delta_ms: number): string;
  /**
   * Dispatch a UiToGameMessage.
   * Returns true when the JSON was valid and dispatched, false otherwise.
   * (Rust signature: handle_input_js(&mut self, input_json: &str) -> bool)
   */
  handle_input_js(input_json: string): boolean;
  /** Total number of ticks simulated since construction. */
  tick_count(): bigint;
  is_running(): boolean;
}

export interface GameLogicWorkerConstructor {
  new(): GameLogicWorker;
}

/**
 * UiController — wasm-ui/src/lib.rs
 *
 * Runs on the UI side (main thread or a dedicated UI worker). Converts
 * incoming JSON game messages into HudData for rendering.
 */
export interface UiController {
  /**
   * Process a JSON-encoded GameToUiMessage.
   * Returns JSON-encoded HudData when the message produced renderable state,
   * or "{}" for message types that don't affect the HUD (EntityMoved, etc.).
   */
  on_game_message(json: string): string;
  /** Build a JSON UiToGameMessage::Input(Move { entity, dx, dy }). */
  send_move(entity: number, dx: number, dy: number): string;
  /** Build a JSON UiToGameMessage::Input(Attack { attacker, target }). */
  send_attack(attacker: number, target: number): string;
  /** Render-frame counter (incremented externally; Rust tracks frame field). */
  frame_count(): bigint;
  /** Number of GameToUiMessage payloads successfully processed. */
  messages_processed(): bigint;
  /** Tick value from the most-recently processed StateUpdate. */
  last_game_tick(): bigint;
}

export interface UiControllerConstructor {
  new(): UiController;
}

/**
 * WasmThreadContext — wasm-core/src/lib.rs
 *
 * Tracks the active threading approach and exposes browser-header requirements.
 * Note: the Rust WASM constructor is new_separate_modules(worker_count); other
 * approaches must be constructed server-side or converted before being passed
 * across the WASM ABI.
 */
export interface WasmThreadContext {
  worker_count(): number;
  requires_coop_coep(): boolean;
  approach_name(): string;
}

export interface WasmThreadContextConstructor {
  new_separate_modules(worker_count: number): WasmThreadContext;
}
