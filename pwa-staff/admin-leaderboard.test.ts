// @ts-nocheck
import { describe, it, expect, vi, beforeEach } from 'vitest';

// Lightweight DOM Mocks
class MockElement {
  tagName: string;
  childNodes: MockElement[] = [];
  attributes: Record<string, string> = {};
  textContent: string = '';
  _innerHTML: string = '';
  style = { display: 'none' };
  dataset: Record<string, string> = {};
  className: string = '';
  listeners: Record<string, Function[]> = {};

  constructor(tagName: string) {
    this.tagName = tagName.toUpperCase();
  }

  get innerHTML() {
    return this._innerHTML;
  }

  set innerHTML(value: string) {
    this._innerHTML = value;
    if (value === '') {
      this.childNodes = [];
    }
  }

  appendChild(child: MockElement) {
    this.childNodes.push(child);
    return child;
  }

  setAttribute(name: string, value: string) {
    this.attributes[name] = value;
    if (name.startsWith('data-')) {
      const prop = name.slice(5).replace(/-([a-z])/g, (g) => g[1].toUpperCase());
      this.dataset[prop] = value;
    }
  }

  addEventListener(event: string, callback: Function) {
    if (!this.listeners[event]) {
      this.listeners[event] = [];
    }
    this.listeners[event].push(callback);
  }

  getElementsByTagName(name: string) {
    if (name.toUpperCase() === 'TBODY') {
      const tbody = new MockElement('tbody');
      this.appendChild(tbody);
      return [tbody];
    }
    return [];
  }

  insertRow() {
    const row = new MockElement('tr');
    this.appendChild(row);
    row.insertCell = () => {
      const cell = new MockElement('td');
      row.appendChild(cell);
      return cell;
    };
    return row;
  }
}

const mockGetElementById = vi.fn();
const mockCreateElement = vi.fn((tag) => new MockElement(tag));
const mockCreateTextNode = vi.fn((text) => ({ textContent: text }));

globalThis.document = {
  getElementById: (id) => mockGetElementById(id),
  createElement: (tag) => mockCreateElement(tag),
  createTextNode: (text) => mockCreateTextNode(text),
} as any;

globalThis.window = {} as any;

// Mock Supabase
const mockSelect = vi.fn();
const mockSingle = vi.fn();
const mockEq = vi.fn();
const mockOrder = vi.fn();

const mockFrom = vi.fn((table) => {
  return {
    select: mockSelect,
    order: mockOrder,
  };
});

vi.mock('./src/lib/supabaseClient', () => {
  return {
    supabase: {
      from: (table) => mockFrom(table),
      channel: () => ({
        on: () => ({
          subscribe: () => {},
        }),
      }),
    },
  };
});

describe('Admin & Leaderboard Dynamic Rendering and Error Trapping', () => {
  beforeEach(() => {
    vi.resetModules();
    mockGetElementById.mockReset();
    mockCreateElement.mockClear();
    mockCreateTextNode.mockClear();
    mockSelect.mockReset();
    mockSingle.mockReset();
    mockEq.mockReset();
    mockOrder.mockReset();
    mockFrom.mockClear();
  });

  it('admin: renderPlayers should dynamically create table elements and use textContent to avoid XSS', async () => {
    const playerList = new MockElement('div');
    mockGetElementById.mockImplementation((id) => {
      if (id === 'player-list') return playerList;
      return new MockElement('div');
    });

    mockSelect.mockResolvedValue({ data: [], error: null });

    // Import admin to trigger rendering logic
    const adminModule = await import('./src/admin');

    const testPlayers = [
      { id: '1', name: '<script>alert("XSS")</script>', email: '<b>test@test.com</b>' },
    ];

    // Clear element creators trace
    mockCreateElement.mockClear();

    // Call the function
    adminModule.renderPlayers(testPlayers);

    // Verify child elements were created via DOM API
    expect(mockCreateElement).toHaveBeenCalledWith('table');
    expect(mockCreateElement).toHaveBeenCalledWith('thead');
    expect(mockCreateElement).toHaveBeenCalledWith('tbody');
    expect(mockCreateElement).toHaveBeenCalledWith('tr');
    expect(mockCreateElement).toHaveBeenCalledWith('th');
    expect(mockCreateElement).toHaveBeenCalledWith('td');
    expect(mockCreateElement).toHaveBeenCalledWith('button');

    // Verify that playerList innerHTML was cleared, but the table content was NOT set via innerHTML interpolation
    expect(playerList.innerHTML).toBe('');

    // Verify that textContent was set with the raw/unsafe HTML strings (which escaping handles correctly)
    const tableEl = playerList.childNodes[0];
    const tbodyEl = tableEl.childNodes.find((c) => c.tagName === 'TBODY');
    const rowEl = tbodyEl.childNodes[0];
    const tdElements = rowEl.childNodes;

    expect(tdElements[0].textContent).toBe('<script>alert("XSS")</script>');
    expect(tdElements[1].textContent).toBe('<b>test@test.com</b>');
  });

  it('admin: handleViewClick and handleEditClick should wrap getPlayer in try/catch to log errors', async () => {
    const playerList = new MockElement('div');
    const viewModal = new MockElement('div');
    const editModal = new MockElement('div');

    mockGetElementById.mockImplementation((id) => {
      if (id === 'player-list') return playerList;
      if (id === 'view-modal') return viewModal;
      if (id === 'edit-modal') return editModal;
      return new MockElement('div');
    });

    // Mock getPlayer to reject/fail
    mockSelect.mockReturnValue({
      eq: () => ({
        single: () => Promise.resolve({ data: null, error: new Error('DB Lookup Failed') }),
      }),
    });

    const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

    const adminModule = await import('./src/admin');

    // Simulate clicking View button
    const mockViewEvent = {
      target: {
        classList: {
          contains: (cls) => cls === 'view-button',
        },
        dataset: {
          id: 'player-123',
        },
      },
    } as any;

    await adminModule.handleViewClick(mockViewEvent);

    // Verify try/catch caught the error and logged it
    expect(consoleErrorSpy).toHaveBeenCalledWith(
      expect.stringContaining('Error fetching player details:'),
      expect.anything()
    );

    // Simulate clicking Edit button
    const mockEditEvent = {
      target: {
        classList: {
          contains: (cls) => cls === 'edit-button',
        },
        dataset: {
          id: 'player-123',
        },
      },
    } as any;

    await adminModule.handleEditClick(mockEditEvent);

    // Verify try/catch caught the error and logged it
    expect(consoleErrorSpy).toHaveBeenCalledWith(
      expect.stringContaining('Error fetching player for editing:'),
      expect.anything()
    );

    consoleErrorSpy.mockRestore();
  });

  it('leaderboard: fetchScores should populate rows dynamically and set textContent for XSS prevention', async () => {
    const leaderboardTable = new MockElement('table');

    mockGetElementById.mockImplementation((id) => {
      if (id === 'leaderboard-table') return leaderboardTable;
      return null;
    });

    // Mock scores response with potential XSS player name
    const mockScores = [{ id: '1', score: 100, players: { username: '<u>XSS</u>' } }];
    mockSelect.mockReturnValue({
      order: () => Promise.resolve({ data: mockScores, error: null }),
    });

    const leaderboardModule = await import('./src/leaderboard');

    // Wait for async fetchScores in module loading
    await new Promise((resolve) => setTimeout(resolve, 10));

    // Verify the rows were populated using insertRow/insertCell and textContent
    const mockTbody = leaderboardTable.childNodes[0];
    expect(mockTbody).toBeDefined();

    const row = mockTbody.childNodes[0];
    expect(row).toBeDefined();

    // index + 1
    expect(row.childNodes[0].textContent).toBe('1');
    // username
    expect(row.childNodes[1].textContent).toBe('<u>XSS</u>');
    // score
    expect(row.childNodes[2].textContent).toBe('100');
  });
});
