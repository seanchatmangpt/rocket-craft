// game-worker.ts — runs inside a Web Worker
// In production: import init, { GameLogicWorker } from './pkg/wasm_game_logic.js';

import type { UiToGameMessage, GameToUiMessage, FromGameMessage } from './types.js';

declare function postMessage(msg: GameToUiMessage | FromGameMessage): void;

interface GameWorkerState {
  tick: number;
  running: boolean;
  playerHealth: number;
  playerScore: number;
  entityCount: number;
}

// Simulates a loaded GameLogicWorker
const state: GameWorkerState = {
  tick: 0,
  running: false,
  playerHealth: 100,
  playerScore: 0,
  entityCount: 1,
};

function handleInput(msg: UiToGameMessage): string {
  if (msg === 'Pause') {
    state.running = false;
    return '';
  }
  if (msg === 'Resume') {
    state.running = true;
    return '';
  }
  if (msg === 'Restart') {
    state.tick = 0;
    state.playerHealth = 100;
    state.playerScore = 0;
    state.running = true;
    return '';
  }
  if (typeof msg === 'object' && 'Ping' in msg) {
    const pong: FromGameMessage = { Pong: { seq: msg.Ping.seq } };
    return JSON.stringify(pong);
  }
  return '';
}

self.addEventListener('message', (e: MessageEvent) => {
  const data = e.data as { type: string; payload?: unknown };
  if (data.type === 'start') {
    state.running = true;
    startTickLoop();
  } else if (data.type === 'input') {
    const reply = handleInput(data.payload as UiToGameMessage);
    if (reply) {
      postMessage(JSON.parse(reply) as FromGameMessage);
    }
  }
});

function tick(deltaMs: number): void {
  if (!state.running) return;
  state.tick += 1;
  state.playerScore = state.tick * 10;
  const msg: GameToUiMessage = {
    StateUpdate: {
      tick: state.tick,
      entity_count: state.entityCount,
      player_health: state.playerHealth,
      player_health_max: 100,
      player_score: state.playerScore,
    },
  };
  postMessage(msg);
}

let lastTime = 0;
function startTickLoop(): void {
  // In a real Worker we'd use setInterval; for simulation use a counter
  let count = 0;
  const id = setInterval(() => {
    tick(16.67);
    count++;
    if (count >= 100) clearInterval(id);
  }, 16);
}
