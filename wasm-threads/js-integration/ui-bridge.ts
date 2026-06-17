// ui-bridge.ts — runs on the main thread
import type {
  GameToUiMessage,
  UiToGameMessage,
  HudData,
  InputCommand,
  ThreadingApproach,
} from './types.js';

export interface GameBridgeOptions {
  workerUrl: string;
  approach: ThreadingApproach;
  onHudUpdate?: (hud: HudData) => void;
  onGameOver?: (winnerScore: number, totalTicks: number) => void;
}

export class GameBridge {
  private worker: Worker | null = null;
  private frameCount = 0;
  private options: GameBridgeOptions;
  private hudState: HudData = {
    player_health: 100,
    player_health_max: 100,
    score: 0,
    entity_count: 0,
    game_tick: 0,
    fps: 0,
    messages_per_second: 0,
  };

  constructor(options: GameBridgeOptions) {
    this.options = options;
  }

  start(): void {
    this.worker = new Worker(this.options.workerUrl, { type: 'module' });
    this.worker.addEventListener('message', (e: MessageEvent) => {
      this.onWorkerMessage(e.data as GameToUiMessage);
    });
    this.worker.postMessage({ type: 'start' });
  }

  private onWorkerMessage(msg: GameToUiMessage): void {
    this.frameCount += 1;
    if ('StateUpdate' in msg) {
      const u = msg.StateUpdate;
      this.hudState = {
        player_health: u.player_health ?? 0,
        player_health_max: u.player_health_max ?? 100,
        score: u.player_score,
        entity_count: u.entity_count,
        game_tick: u.tick,
        fps: 0,
        messages_per_second: 0,
      };
      this.options.onHudUpdate?.(this.hudState);
    } else if ('GameOver' in msg) {
      this.options.onGameOver?.(msg.GameOver.winner_score, msg.GameOver.total_ticks);
    }
  }

  sendMove(entity: number, dx: number, dy: number): void {
    const cmd: InputCommand = { Move: { entity, dx, dy } };
    const msg: UiToGameMessage = { Input: cmd };
    this.worker?.postMessage({ type: 'input', payload: msg });
  }

  sendAttack(attacker: number, target: number): void {
    const cmd: InputCommand = { Attack: { attacker, target } };
    const msg: UiToGameMessage = { Input: cmd };
    this.worker?.postMessage({ type: 'input', payload: msg });
  }

  pause(): void {
    this.worker?.postMessage({ type: 'input', payload: 'Pause' as UiToGameMessage });
  }

  resume(): void {
    this.worker?.postMessage({ type: 'input', payload: 'Resume' as UiToGameMessage });
  }

  sendPing(seq: number): void {
    const msg: UiToGameMessage = { Ping: { seq } };
    this.worker?.postMessage({ type: 'input', payload: msg });
  }

  getHudState(): HudData {
    return { ...this.hudState };
  }

  getFrameCount(): number {
    return this.frameCount;
  }

  stop(): void {
    this.worker?.terminate();
    this.worker = null;
  }

  requiresCOOPCOEP(): boolean {
    const approach = this.options.approach;
    if ('SharedMemory' in approach || 'Hybrid' in approach) return true;
    return false;
  }
}
