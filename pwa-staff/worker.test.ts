// @ts-nocheck
import { describe, it, expect, vi, beforeEach } from 'vitest';
import fs from 'fs';
import path from 'path';

// Mocking the Service Worker global scope
const makeServiceWorkerEnv = () => {
  const listeners = {};
  const self = {
    addEventListener: vi.fn((event, callback) => {
      listeners[event] = callback;
    }),
    skipWaiting: vi.fn().mockResolvedValue(),
    clients: {
      claim: vi.fn().mockResolvedValue(),
    },
    registration: {
      scope: '/',
    },
  };

  const caches = {
    open: vi.fn().mockResolvedValue({
      add: vi.fn().mockResolvedValue(),
      addAll: vi.fn().mockResolvedValue(),
      put: vi.fn().mockResolvedValue(),
      match: vi.fn().mockResolvedValue(null),
    }),
    keys: vi.fn().mockResolvedValue([]),
    delete: vi.fn().mockResolvedValue(true),
    match: vi.fn().mockResolvedValue(null),
  };

  const Response = {
    error: vi.fn().mockReturnValue({ type: 'error' }),
  };

  return { self, listeners, caches, Response };
};

describe('Service Worker', () => {
  let env;
  let workerCode: any;

  beforeEach(() => {
    env = makeServiceWorkerEnv();
    globalThis.self = env.self;
    globalThis.caches = env.caches;
    globalThis.Response = env.Response;
    globalThis.fetch = vi.fn();

    if (!workerCode) {
      const workerPath = path.resolve(process.cwd(), 'worker.js');
      workerCode = fs.readFileSync(workerPath, 'utf8');
    }

    // Execute worker code in the mocked global scope
    // We use eval here because it's a simple script not using modules
    eval(workerCode);
  });

  it('should register install, activate and fetch event listeners', () => {
    expect(env.self.addEventListener).toHaveBeenCalledWith('install', expect.any(Function));
    expect(env.self.addEventListener).toHaveBeenCalledWith('activate', expect.any(Function));
    expect(env.self.addEventListener).toHaveBeenCalledWith('fetch', expect.any(Function));
  });

  it('should pre-cache assets on install', async () => {
    const installCallback = env.listeners['install'];
    const event = {
      waitUntil: vi.fn(),
    };

    installCallback(event);

    expect(event.waitUntil).toHaveBeenCalled();
    const promise = event.waitUntil.mock.calls[0][0];
    await promise;

    expect(env.caches.open).toHaveBeenCalledWith(expect.stringContaining('static-assets'));
  });

  it('should fail install if critical asset (offline.html) fails to cache', async () => {
    const installCallback = env.listeners['install'];
    const event = {
      waitUntil: vi.fn(),
    };

    env.caches.open.mockResolvedValue({
      add: vi.fn().mockImplementation((url) => {
        if (url === 'offline.html') {
          return Promise.reject(new Error('Failed to fetch offline.html'));
        }
        return Promise.resolve();
      }),
    });

    installCallback(event);

    expect(event.waitUntil).toHaveBeenCalled();
    const promise = event.waitUntil.mock.calls[0][0];
    await expect(promise).rejects.toThrow('Critical asset (offline.html) failed to cache');
  });

  it('should clean up old caches on activate', async () => {
    const activateCallback = env.listeners['activate'];
    const event = {
      waitUntil: vi.fn(),
    };

    env.caches.keys.mockResolvedValue(['old-cache', 'static-assets-v2']);

    activateCallback(event);

    const promise = event.waitUntil.mock.calls[0][0];
    await promise;

    expect(env.caches.delete).toHaveBeenCalledWith('old-cache');
    expect(env.caches.delete).not.toHaveBeenCalledWith('static-assets-v2');
  });
});
