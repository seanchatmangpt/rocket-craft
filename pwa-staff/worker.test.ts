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

  describe('Fetch Event Handling', () => {
    it('should ignore non-GET requests', () => {
      const fetchCallback = env.listeners['fetch'];
      const event = {
        request: new Request('http://localhost:3000/index.html', { method: 'POST' }),
        respondWith: vi.fn(),
      };

      fetchCallback(event);
      expect(event.respondWith).not.toHaveBeenCalled();
    });

    it('should implement Cache First strategy for standard assets', async () => {
      const fetchCallback = env.listeners['fetch'];
      const cachedResponse = { status: 200, type: 'basic' };
      env.caches.match.mockResolvedValue(cachedResponse);

      const event = {
        request: new Request('http://localhost:3000/index.html', { method: 'GET' }),
        respondWith: vi.fn(),
      };

      fetchCallback(event);

      expect(event.respondWith).toHaveBeenCalled();
      const response = await event.respondWith.mock.calls[0][0];
      expect(response).toBe(cachedResponse);
      expect(globalThis.fetch).not.toHaveBeenCalled();
    });

    it('should fall back to network if cache misses for standard assets', async () => {
      const fetchCallback = env.listeners['fetch'];
      env.caches.match.mockResolvedValue(null);

      const networkResponse = {
        status: 200,
        type: 'basic',
        clone: vi.fn().mockReturnValue({ status: 200, type: 'basic' }),
      };
      globalThis.fetch.mockResolvedValue(networkResponse);

      const event = {
        request: new Request('http://localhost:3000/index.html', { method: 'GET' }),
        respondWith: vi.fn(),
      };

      fetchCallback(event);

      expect(event.respondWith).toHaveBeenCalled();
      const response = await event.respondWith.mock.calls[0][0];
      expect(response).toBe(networkResponse);
      expect(globalThis.fetch).toHaveBeenCalledWith(event.request);
    });

    it('should implement Network First strategy for manufactured assets', async () => {
      const fetchCallback = env.listeners['fetch'];
      
      const networkResponse = {
        status: 200,
        type: 'basic',
        clone: vi.fn().mockReturnValue({ status: 200, type: 'basic' }),
      };
      globalThis.fetch.mockResolvedValue(networkResponse);

      const event = {
        request: new Request('http://localhost:3000/manufactured/Brm-HTML5-Shipping.wasm', { method: 'GET' }),
        respondWith: vi.fn(),
      };

      fetchCallback(event);

      expect(event.respondWith).toHaveBeenCalled();
      const response = await event.respondWith.mock.calls[0][0];
      expect(response).toBe(networkResponse);
      expect(globalThis.fetch).toHaveBeenCalledWith(event.request);
    });

    it('should rewrite root Brm-HTML5-Shipping requests to /manufactured/ paths', async () => {
      const fetchCallback = env.listeners['fetch'];

      const networkResponse = {
        status: 200,
        type: 'basic',
        clone: vi.fn().mockReturnValue({ status: 200, type: 'basic' }),
      };
      globalThis.fetch.mockResolvedValue(networkResponse);

      const event = {
        request: new Request('http://localhost:3000/Brm-HTML5-Shipping.wasm', { method: 'GET' }),
        respondWith: vi.fn(),
      };

      fetchCallback(event);

      expect(event.respondWith).toHaveBeenCalled();
      await event.respondWith.mock.calls[0][0];
      
      expect(globalThis.fetch).toHaveBeenCalled();
      const fetchArg = globalThis.fetch.mock.calls[0][0];
      expect(fetchArg.url).toBe('http://localhost:3000/manufactured/Brm-HTML5-Shipping.wasm');
    });

    it('should fall back to cache for rewritten root Brm requests if fetch fails', async () => {
      const fetchCallback = env.listeners['fetch'];
      globalThis.fetch.mockRejectedValue(new Error('Network error'));

      const cachedResponse = { status: 200, type: 'basic' };
      env.caches.match.mockResolvedValue(cachedResponse);

      const event = {
        request: new Request('http://localhost:3000/Brm-HTML5-Shipping.wasm', { method: 'GET' }),
        respondWith: vi.fn(),
      };

      fetchCallback(event);

      expect(event.respondWith).toHaveBeenCalled();
      const response = await event.respondWith.mock.calls[0][0];
      expect(response).toBe(cachedResponse);
      expect(env.caches.match).toHaveBeenCalledWith('http://localhost:3000/manufactured/Brm-HTML5-Shipping.wasm');
    });
  });
});
