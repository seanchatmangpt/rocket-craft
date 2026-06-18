// @ts-nocheck
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { initHUD } from './src/hud';

// Helper to construct base64url encoded tokens
function makeFakeToken(payload: object) {
  const payloadStr = JSON.stringify(payload);
  const base64 = btoa(payloadStr).replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '');
  return `header.${base64}.signature`;
}

// Lightweight DOM Mocks
class MockElement {
  id: string = '';
  tagName: string;
  childNodes: MockElement[] = [];
  attributes: Record<string, string> = {};
  textContent: string = '';
  innerHTML: string = '';
  style = { display: 'none' };
  className: string = '';
  listeners: Record<string, Function[]> = {};

  classList = {
    add: (name: string) => {
      const classes = this.className.split(' ').filter(Boolean);
      if (!classes.includes(name)) {
        classes.push(name);
      }
      this.className = classes.join(' ');
    },
    remove: (name: string) => {
      const classes = this.className.split(' ').filter(Boolean);
      const idx = classes.indexOf(name);
      if (idx !== -1) {
        classes.splice(idx, 1);
      }
      this.className = classes.join(' ');
    },
    toggle: (name: string) => {
      const classes = this.className.split(' ').filter(Boolean);
      const idx = classes.indexOf(name);
      if (idx !== -1) {
        classes.splice(idx, 1);
      } else {
        classes.push(name);
      }
      this.className = classes.join(' ');
    },
    contains: (name: string) => {
      return this.className.split(' ').filter(Boolean).includes(name);
    },
  };

  constructor(tagName: string) {
    this.tagName = tagName.toUpperCase();
  }

  appendChild(child: MockElement) {
    this.childNodes.push(child);
    return child;
  }

  setAttribute(name: string, value: string) {
    this.attributes[name] = value;
  }

  addEventListener(event: string, callback: Function) {
    if (!this.listeners[event]) {
      this.listeners[event] = [];
    }
    this.listeners[event].push(callback);
  }

  get firstChild() {
    return this.childNodes[0] || null;
  }

  removeChild(child: MockElement) {
    const idx = this.childNodes.indexOf(child);
    if (idx !== -1) {
      this.childNodes.splice(idx, 1);
    }
  }
}

const mockGetElementById = vi.fn();
const mockCreatedElements: MockElement[] = [];
const mockCreateElement = vi.fn((tag) => {
  const el = new MockElement(tag);
  mockCreatedElements.push(el);
  return el;
});

globalThis.document = {
  readyState: 'complete',
  head: {
    appendChild: vi.fn(),
  },
  body: {
    appendChild: vi.fn(),
  },
  getElementById: (id) => {
    // Return mock elements created with matching IDs
    const found = mockCreatedElements.find((el) => el.id === id);
    if (found) return found;
    // Fallback search in attributes
    return mockGetElementById(id);
  },
  createElement: (tag) => mockCreateElement(tag),
} as any;

const mockAlert = vi.fn();
globalThis.window = {
  alert: mockAlert,
  addEventListener: vi.fn(),
  removeEventListener: vi.fn(),
} as any;
globalThis.alert = mockAlert;

globalThis.fetch = vi.fn();

// Mock Supabase
const mockGetSession = vi.fn();
const mockOnAuthStateChange = vi.fn();
const mockSelect = vi.fn();

vi.mock('./src/lib/supabaseClient', () => {
  return {
    supabase: {
      auth: {
        getSession: () => mockGetSession(),
        onAuthStateChange: (cb) => mockOnAuthStateChange(cb),
      },
      from: (table) => ({
        select: mockSelect,
      }),
    },
  };
});

function getDeepTextContent(el: MockElement): string {
  let text = el.textContent || '';
  for (const child of el.childNodes) {
    text += getDeepTextContent(child);
  }
  return text;
}

describe('HUD Console UI & Logic', () => {
  beforeEach(() => {
    mockGetElementById.mockReset();
    mockCreateElement.mockClear();
    mockGetSession.mockReset();
    mockOnAuthStateChange.mockReset();
    mockSelect.mockReset();
    globalThis.fetch.mockReset();
    globalThis.document.body.appendChild.mockClear();
    globalThis.document.head.appendChild.mockClear();
    mockCreatedElements.length = 0;
  });

  it('should construct DOM elements for HUD drawer and toggle button', async () => {
    mockGetSession.mockResolvedValue({ data: { session: null } });
    mockSelect.mockResolvedValue({ count: 10, error: null });

    // Call initHUD manually
    initHUD();

    // Verify elements are created and appended
    expect(mockCreateElement).toHaveBeenCalledWith('style');
    expect(mockCreateElement).toHaveBeenCalledWith('button');
    expect(mockCreateElement).toHaveBeenCalledWith('div');

    // Verify stylesheet has been appended to head
    expect(globalThis.document.head.appendChild).toHaveBeenCalled();

    // Verify toggle button and drawer appended to body
    expect(globalThis.document.body.appendChild).toHaveBeenCalledTimes(2);
  });

  it('should handle unauthenticated session state correctly', async () => {
    mockGetSession.mockResolvedValue({ data: { session: null } });
    mockSelect.mockResolvedValue({ count: 0, error: null });

    // Call initHUD manually
    initHUD();

    const authDetailsEl = mockCreatedElements.find((el) => el.id === 'hud-auth-details');
    expect(authDetailsEl).toBeDefined();

    // Wait for microtasks
    await new Promise((resolve) => setTimeout(resolve, 5));

    const allText = getDeepTextContent(authDetailsEl);
    expect(allText).toBe('Unauthenticated');
  });

  it('should decode JWT token and populate session details when logged in', async () => {
    const fakePayload = {
      email: 'admin@rocketcraft.com',
      sub: 'user-sub-id-123',
      role: 'authenticated',
      exp: 1770000000,
    };
    const token = makeFakeToken(fakePayload);
    const mockSession = {
      access_token: token,
      user: { email: 'admin@rocketcraft.com' },
    };

    mockGetSession.mockResolvedValue({ data: { session: mockSession } });
    mockSelect.mockResolvedValue({ count: 0, error: null });

    // Call initHUD manually
    initHUD();

    const authDetailsEl = mockCreatedElements.find((el) => el.id === 'hud-auth-details');
    expect(authDetailsEl).toBeDefined();

    // Wait for promise resolution
    await new Promise((resolve) => setTimeout(resolve, 5));

    // Verify that JWT fields are displayed
    const allText = getDeepTextContent(authDetailsEl);
    expect(allText).toContain('admin@rocketcraft.com');
    expect(allText).toContain('user-sub-id-123');
    expect(allText).toContain('authenticated');
  });

  it('should query registered players and total game sessions count', async () => {
    mockGetSession.mockResolvedValue({ data: { session: null } });

    // Mock the counts returned by supabase select client method
    mockSelect.mockResolvedValue({ count: 42, error: null });

    // Call initHUD manually
    initHUD();

    // Wait for async stats load
    await new Promise((resolve) => setTimeout(resolve, 5));

    const playersValEl = mockCreatedElements.find((el) => el.id === 'hud-stats-players');
    const sessionsValEl = mockCreatedElements.find((el) => el.id === 'hud-stats-sessions');

    expect(playersValEl.textContent).toBe('42');
    expect(sessionsValEl.textContent).toBe('42');
  });

  it('should trigger score submission and make a POST request with the bearer token', async () => {
    const fakePayload = { email: 'staff@test.com' };
    const token = makeFakeToken(fakePayload);
    const mockSession = { access_token: token };

    mockGetSession.mockResolvedValue({ data: { session: mockSession } });
    mockSelect.mockResolvedValue({ count: 0, error: null });
    globalThis.fetch.mockResolvedValue({
      ok: true,
      text: () => Promise.resolve('Success'),
    });

    // Call initHUD manually
    initHUD();

    const submitScoreBtn = mockCreatedElements.find((el) => el.id === 'dev-hud-submit-score-btn');
    expect(submitScoreBtn).toBeDefined();

    const clickListeners = submitScoreBtn.listeners['click'];
    expect(clickListeners).toBeDefined();

    // Trigger submission click
    await clickListeners[0]();

    // Verify POST was called
    expect(globalThis.fetch).toHaveBeenCalledWith(
      'http://127.0.0.1:54321/functions/v1/submit-score',
      expect.objectContaining({
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${token}`,
        },
      })
    );
  });

  it('should toggle and collapse HUD drawer on toggle and close button clicks', async () => {
    mockGetSession.mockResolvedValue({ data: { session: null } });
    mockSelect.mockResolvedValue({ count: 0, error: null });

    initHUD();

    const toggleBtn = mockCreatedElements.find((el) => el.className === 'hud-toggle-btn');
    const drawer = mockCreatedElements.find((el) => el.classList.contains('hud-drawer'));
    const closeBtn = mockCreatedElements.find((el) => el.className === 'hud-close-btn');

    expect(toggleBtn).toBeDefined();
    expect(drawer).toBeDefined();
    expect(closeBtn).toBeDefined();

    // Starts collapsed
    expect(drawer.classList.contains('hud-collapsed')).toBe(true);

    // Click toggle button -> should open (not collapsed)
    const toggleClick = toggleBtn.listeners['click'][0];
    toggleClick();
    expect(drawer.classList.contains('hud-collapsed')).toBe(false);

    // Click toggle button again -> should collapse
    toggleClick();
    expect(drawer.classList.contains('hud-collapsed')).toBe(true);

    // Click toggle button to open
    toggleClick();
    expect(drawer.classList.contains('hud-collapsed')).toBe(false);

    // Click close button -> should collapse
    const closeClick = closeBtn.listeners['click'][0];
    closeClick();
    expect(drawer.classList.contains('hud-collapsed')).toBe(true);
  });

  it('should handle invalid JWT decoding error, display message, and log error', async () => {
    const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    const mockSession = { access_token: 'invalid.jwt.token' };

    mockGetSession.mockResolvedValue({ data: { session: mockSession } });
    mockSelect.mockResolvedValue({ count: 0, error: null });

    initHUD();

    const authDetailsEl = mockCreatedElements.find((el) => el.id === 'hud-auth-details');
    expect(authDetailsEl).toBeDefined();

    await new Promise((resolve) => setTimeout(resolve, 5));

    // Displays "Invalid Session JWT"
    const allText = getDeepTextContent(authDetailsEl);
    expect(allText).toContain('Invalid Session JWT');

    // Logs error
    expect(consoleErrorSpy).toHaveBeenCalledWith(
      expect.stringContaining('[HUD] Failed to decode JWT:')
    );

    consoleErrorSpy.mockRestore();
  });

  it('should handle database stats API select query failures, display Error, and log error', async () => {
    const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

    mockGetSession.mockResolvedValue({ data: { session: null } });
    // Mock select error
    mockSelect.mockResolvedValue({ count: null, error: new Error('DB Select Failed') });

    initHUD();

    await new Promise((resolve) => setTimeout(resolve, 5));

    const playersValEl = mockCreatedElements.find((el) => el.id === 'hud-stats-players');
    const sessionsValEl = mockCreatedElements.find((el) => el.id === 'hud-stats-sessions');

    expect(playersValEl.textContent).toBe('Error');
    expect(sessionsValEl.textContent).toBe('Error');

    expect(consoleErrorSpy).toHaveBeenCalledWith(
      expect.stringContaining('[HUD] Failed to fetch database stats:')
    );

    consoleErrorSpy.mockRestore();
  });

  it('should handle mock score submission failure when session is null', async () => {
    const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    mockAlert.mockClear();

    mockGetSession.mockResolvedValue({ data: { session: null } });
    mockSelect.mockResolvedValue({ count: 0, error: null });

    initHUD();

    const submitScoreBtn = mockCreatedElements.find((el) => el.id === 'dev-hud-submit-score-btn');
    const clickListener = submitScoreBtn.listeners['click'][0];

    await clickListener();

    expect(consoleErrorSpy).toHaveBeenCalledWith(
      expect.stringContaining('[HUD] Cannot submit score: Unauthenticated')
    );
    expect(mockAlert).toHaveBeenCalledWith('You must be logged in to submit a score.');

    consoleErrorSpy.mockRestore();
  });

  it('should handle mock score submission failure when fetch response is not ok', async () => {
    const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    const fakePayload = { email: 'staff@test.com' };
    const token = makeFakeToken(fakePayload);
    const mockSession = { access_token: token };

    mockGetSession.mockResolvedValue({ data: { session: mockSession } });
    mockSelect.mockResolvedValue({ count: 0, error: null });
    globalThis.fetch.mockResolvedValue({
      ok: false,
      status: 500,
      text: () => Promise.resolve('Internal Server Error'),
    });

    initHUD();

    const submitScoreBtn = mockCreatedElements.find((el) => el.id === 'dev-hud-submit-score-btn');
    const clickListener = submitScoreBtn.listeners['click'][0];

    await clickListener();

    expect(consoleErrorSpy).toHaveBeenCalledWith(
      expect.stringContaining(
        '[HUD] Failed to submit mock score: HTTP error! status: 500, message: Internal Server Error'
      )
    );

    consoleErrorSpy.mockRestore();
  });
});
