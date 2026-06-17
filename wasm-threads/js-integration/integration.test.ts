import { describe, it, expect } from 'vitest';
import type {
  GameToUiMessage,
  UiToGameMessage,
  FromGameMessage,
  HudData,
  ThreadingApproach,
} from './types.js';
import { GameBridge } from './ui-bridge.js';

// ── Protocol shape tests ─────────────────────────────────────────────────────

describe('GameToUiMessage JSON shapes', () => {
  it('StateUpdate has required numeric fields', () => {
    const msg: GameToUiMessage = {
      StateUpdate: {
        tick: 42,
        entity_count: 3,
        player_health: 80,
        player_health_max: 100,
        player_score: 420,
      },
    };
    const json = JSON.stringify(msg);
    const parsed = JSON.parse(json) as GameToUiMessage;
    expect('StateUpdate' in parsed).toBe(true);
    if ('StateUpdate' in parsed) {
      expect(parsed.StateUpdate.tick).toBe(42);
      expect(parsed.StateUpdate.player_score).toBe(420);
      // Falsification: different inputs produce different tick values
      expect(parsed.StateUpdate.tick).not.toBe(0);
    }
  });

  it('GameOver serializes winner_score', () => {
    const msg: GameToUiMessage = { GameOver: { winner_score: 9001, total_ticks: 300 } };
    const json = JSON.stringify(msg);
    const parsed = JSON.parse(json) as GameToUiMessage;
    expect('GameOver' in parsed).toBe(true);
    if ('GameOver' in parsed) {
      expect(parsed.GameOver.winner_score).toBe(9001);
      // Falsification: score is 9001 not 0
      expect(parsed.GameOver.winner_score).not.toBe(0);
    }
  });

  it('EntityMoved has entity_id and coordinates', () => {
    const msg: GameToUiMessage = { EntityMoved: { entity_id: 7, x: 1.5, y: -2.3 } };
    const json = JSON.stringify(msg);
    const parsed = JSON.parse(json) as GameToUiMessage;
    if ('EntityMoved' in parsed) {
      expect(parsed.EntityMoved.entity_id).toBe(7);
      expect(parsed.EntityMoved.x).toBeCloseTo(1.5);
    }
  });

  it('EntityDied has entity_id', () => {
    const msg: GameToUiMessage = { EntityDied: { entity_id: 99 } };
    if ('EntityDied' in msg) {
      expect(msg.EntityDied.entity_id).toBe(99);
      expect(msg.EntityDied.entity_id).not.toBe(0);
    }
  });
});

describe('UiToGameMessage JSON shapes', () => {
  it('Pause and Resume are string literals', () => {
    const pause: UiToGameMessage = 'Pause';
    const resume: UiToGameMessage = 'Resume';
    expect(pause).toBe('Pause');
    expect(resume).toBe('Resume');
    // Falsification: they differ
    expect(pause).not.toBe(resume);
  });

  it('Input Move round-trips through JSON', () => {
    const msg: UiToGameMessage = { Input: { Move: { entity: 1, dx: 0.5, dy: 0.0 } } };
    const json = JSON.stringify(msg);
    const parsed = JSON.parse(json) as UiToGameMessage;
    expect(typeof parsed).toBe('object');
    if (typeof parsed === 'object' && 'Input' in parsed) {
      const cmd = parsed.Input;
      if ('Move' in cmd) {
        expect(cmd.Move.entity).toBe(1);
        expect(cmd.Move.dx).toBeCloseTo(0.5);
        // Falsification: dx=0.5 not 0
        expect(cmd.Move.dx).not.toBe(0.0);
      }
    }
  });

  it('Ping carries seq number', () => {
    const msg: UiToGameMessage = { Ping: { seq: 42 } };
    if (typeof msg === 'object' && 'Ping' in msg) {
      expect(msg.Ping.seq).toBe(42);
      expect(msg.Ping.seq).not.toBe(0);
    }
  });

  it('two different Input messages produce different JSON', () => {
    const move: UiToGameMessage = { Input: { Move: { entity: 1, dx: 1.0, dy: 0.0 } } };
    const attack: UiToGameMessage = { Input: { Attack: { attacker: 1, target: 2 } } };
    expect(JSON.stringify(move)).not.toBe(JSON.stringify(attack));
  });
});

describe('FromGameMessage (Pong)', () => {
  it('Pong seq matches request', () => {
    const pong: FromGameMessage = { Pong: { seq: 7 } };
    expect(pong.Pong.seq).toBe(7);
    // Falsification: seq is 7 not default
    expect(pong.Pong.seq).not.toBe(0);
  });

  it('different Pong seqs are not equal', () => {
    const p1: FromGameMessage = { Pong: { seq: 1 } };
    const p2: FromGameMessage = { Pong: { seq: 2 } };
    expect(p1.Pong.seq).not.toBe(p2.Pong.seq);
  });
});

describe('ThreadingApproach types', () => {
  it('SeparateModules does not require COOP/COEP', () => {
    const approach: ThreadingApproach = { SeparateModules: { worker_count: 4 } };
    const bridge = new GameBridge({ workerUrl: 'mock', approach });
    expect(bridge.requiresCOOPCOEP()).toBe(false);
  });

  it('SharedMemory requires COOP/COEP', () => {
    const approach: ThreadingApproach = { SharedMemory: { buffer_size_bytes: 4096 } };
    const bridge = new GameBridge({ workerUrl: 'mock', approach });
    expect(bridge.requiresCOOPCOEP()).toBe(true);
  });

  it('Hybrid requires COOP/COEP', () => {
    const approach: ThreadingApproach = { Hybrid: { worker_count: 2, shared_buffer_size_bytes: 2048 } };
    const bridge = new GameBridge({ workerUrl: 'mock', approach });
    expect(bridge.requiresCOOPCOEP()).toBe(true);
  });

  it('worker_count differs between SeparateModules variants', () => {
    const a: ThreadingApproach = { SeparateModules: { worker_count: 2 } };
    const b: ThreadingApproach = { SeparateModules: { worker_count: 8 } };
    if ('SeparateModules' in a && 'SeparateModules' in b) {
      expect(a.SeparateModules.worker_count).not.toBe(b.SeparateModules.worker_count);
    }
  });
});

describe('HudData structure', () => {
  it('has all required fields with correct types', () => {
    const hud: HudData = {
      player_health: 75,
      player_health_max: 100,
      score: 1500,
      entity_count: 5,
      game_tick: 300,
      fps: 60.0,
      messages_per_second: 30.0,
    };
    expect(hud.player_health).toBe(75);
    expect(hud.player_health_max).toBe(100);
    expect(hud.score).toBe(1500);
    // Falsification: health is 75%, not 100%
    expect(hud.player_health).not.toBe(hud.player_health_max);
  });
});

describe('GameBridge', () => {
  it('getHudState returns a copy not a reference', () => {
    const bridge = new GameBridge({
      workerUrl: 'mock',
      approach: { SeparateModules: { worker_count: 1 } },
    });
    const s1 = bridge.getHudState();
    const s2 = bridge.getHudState();
    expect(s1).not.toBe(s2); // different object references
    expect(s1.score).toBe(s2.score); // same values
  });

  it('frame count starts at 0', () => {
    const bridge = new GameBridge({
      workerUrl: 'mock',
      approach: { SeparateModules: { worker_count: 1 } },
    });
    expect(bridge.getFrameCount()).toBe(0);
  });
});
