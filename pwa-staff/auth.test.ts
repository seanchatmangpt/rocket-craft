// @ts-nocheck
import { describe, it, expect, vi, beforeEach } from 'vitest';
import fs from 'fs';
import path from 'path';

// Mock Supabase client
const mockGetUser = vi.fn();
const mockGetSession = vi.fn();
const mockSignOut = vi.fn();
const mockSignUp = vi.fn();
const mockSignInWithPassword = vi.fn();
const mockInsert = vi.fn(() => Promise.resolve({ data: null, error: null }));
const mockFrom = vi.fn((table) => {
  const chain = {
    select: vi.fn(() => chain),
    eq: vi.fn(() => chain),
    single: vi.fn(() => Promise.resolve({ data: { username: 'mock_user' }, error: null })),
    insert: mockInsert,
  };
  return chain;
});

vi.mock('./src/lib/supabaseClient', () => {
  return {
    supabase: {
      auth: {
        getUser: () => mockGetUser(),
        getSession: () => mockGetSession(),
        signOut: () => mockSignOut(),
        signUp: (args) => mockSignUp(args),
        signInWithPassword: (args) => mockSignInWithPassword(args),
      },
      from: (table) => mockFrom(table),
    },
  };
});

describe('Supabase Auth Integration & Redirection Verification', () => {
  describe('HTML Relative Asset Paths', () => {
    const htmlFiles = ['login.html', 'signup.html', 'profile.html'];

    htmlFiles.forEach((file) => {
      it(`should have correct dist/ asset paths in ${file}`, () => {
        const filePath = path.resolve(process.cwd(), file);
        const content = fs.readFileSync(filePath, 'utf8');

        // Verify style.css is under dist/
        expect(content).toContain('href="dist/style.css"');
        expect(content).not.toContain('href="css/style.css"');

        // Verify JS script is under dist/
        const scriptName = file.replace('.html', '.js');
        expect(content).toContain(`src="dist/${scriptName}"`);
        expect(content).not.toContain(`src="src/${file.replace('.html', '.ts')}"`);
        expect(content).not.toContain(`src="js/${scriptName}"`);
      });
    });
  });

  describe('Profile Page Auth and Redirects', () => {
    let mockUserEmailElement: any;
    let mockLogoutButton: any;
    let buttonListeners: Record<string, Function>;

    beforeEach(() => {
      vi.resetModules();
      mockGetUser.mockReset();
      mockGetSession.mockReset();
      mockSignOut.mockReset();

      buttonListeners = {};

      mockUserEmailElement = {
        textContent: '',
      };

      mockLogoutButton = {
        addEventListener: vi.fn((event, cb) => {
          buttonListeners[event] = cb;
        }),
      };

      // Mock DOM globals
      globalThis.window = {
        location: {
          href: '',
        },
      } as any;

      globalThis.document = {
        getElementById: vi.fn((id) => {
          if (id === 'user-email') return mockUserEmailElement;
          if (id === 'logout-button') return mockLogoutButton;
          return null;
        }),
      } as any;

      globalThis.alert = vi.fn();
    });

    it('redirects unauthenticated users to login.html', async () => {
      // Mock unauthenticated state (getUser returns null user or error)
      mockGetUser.mockResolvedValue({
        data: { user: null },
        error: new Error('No active session'),
      });
      mockGetSession.mockResolvedValue({
        data: { session: null },
        error: null,
      });

      // Import the profile module to trigger initProfile
      await import('./src/profile');

      // Allow microtasks to run (since initProfile is async)
      await new Promise((resolve) => setTimeout(resolve, 10));

      expect(globalThis.window.location.href).toBe('login.html');
    });

    it('shows user email in user-email element for authenticated users', async () => {
      // Mock authenticated state
      mockGetUser.mockResolvedValue({
        data: { user: { email: 'staff@rocketcraft.com' } },
        error: null,
      });
      mockGetSession.mockResolvedValue({
        data: { session: { user: { email: 'staff@rocketcraft.com' } } },
        error: null,
      });

      // Import the profile module to trigger initProfile
      await import('./src/profile');

      // Allow microtasks to run
      await new Promise((resolve) => setTimeout(resolve, 10));

      expect(mockUserEmailElement.textContent).toBe('staff@rocketcraft.com');
      expect(globalThis.window.location.href).not.toBe('login.html');
    });

    it('calls signOut and redirects to login.html on logout button click', async () => {
      // Mock authenticated state initially
      mockGetUser.mockResolvedValue({
        data: { user: { email: 'staff@rocketcraft.com' } },
        error: null,
      });
      mockGetSession.mockResolvedValue({
        data: { session: { user: { email: 'staff@rocketcraft.com' } } },
        error: null,
      });

      mockSignOut.mockResolvedValue({
        error: null,
      });

      // Import the profile module to trigger setup
      await import('./src/profile');

      // Allow microtasks to run
      await new Promise((resolve) => setTimeout(resolve, 10));

      // Verify the event listener was added to logout button
      expect(mockLogoutButton.addEventListener).toHaveBeenCalledWith('click', expect.any(Function));

      // Simulate clicking logout
      const clickHandler = buttonListeners['click'];
      expect(clickHandler).toBeDefined();

      await clickHandler();

      expect(mockSignOut).toHaveBeenCalled();
      expect(globalThis.window.location.href).toBe('login.html');
    });

    it('does not redirect and loads profile successfully even if telemetry insert fails', async () => {
      // Mock authenticated state
      mockGetUser.mockResolvedValue({
        data: { user: { id: 'test-user-id', email: 'staff@rocketcraft.com' } },
        error: null,
      });
      mockGetSession.mockResolvedValue({
        data: { session: { user: { id: 'test-user-id', email: 'staff@rocketcraft.com' } } },
        error: null,
      });

      // Force telemetry insert to reject/throw an error
      mockInsert.mockRejectedValueOnce(new Error('Database error'));

      // Import the profile module to trigger initProfile
      await import('./src/profile');

      // Allow microtasks to run
      await new Promise((resolve) => setTimeout(resolve, 10));

      // Profile should still be active, no redirect to login.html
      expect(mockUserEmailElement.textContent).toBe('staff@rocketcraft.com');
      expect(globalThis.window.location.href).not.toBe('login.html');
    });
  });

  describe('Signup Page Flow', () => {
    let mockSignupForm: any;
    let buttonListeners: Record<string, Function>;

    beforeEach(() => {
      vi.resetModules();
      mockSignUp.mockReset();
      mockInsert.mockReset();

      buttonListeners = {};

      mockSignupForm = {
        addEventListener: vi.fn((event, cb) => {
          buttonListeners[event] = cb;
        }),
      };

      globalThis.window = {
        location: {
          href: '',
        },
      } as any;

      globalThis.document = {
        getElementById: vi.fn((id) => {
          if (id === 'signup-form') return mockSignupForm;
          if (id === 'email') return { value: 'newuser@example.com' };
          if (id === 'password') return { value: 'password123' };
          return null;
        }),
      } as any;

      globalThis.alert = vi.fn();
    });

    it('submits form, inserts registration telemetry, and redirects to profile.html', async () => {
      mockSignUp.mockResolvedValue({
        data: { user: { id: 'new-user-id', email: 'newuser@example.com' } },
        error: null,
      });
      mockInsert.mockResolvedValue({ data: null, error: null });

      await import('./src/signup');

      const submitHandler = buttonListeners['submit'];
      expect(submitHandler).toBeDefined();

      const preventDefault = vi.fn();
      await submitHandler({ preventDefault });

      expect(preventDefault).toHaveBeenCalled();
      expect(mockSignUp).toHaveBeenCalledWith({
        email: 'newuser@example.com',
        password: 'password123',
      });
      expect(mockInsert).toHaveBeenCalledWith({
        player_id: 'new-user-id',
        event_type: 'registration',
        payload: { email: 'newuser@example.com' },
      });
      expect(globalThis.window.location.href).toBe('profile.html');
    });

    it('redirects to profile.html even if registration telemetry insert fails', async () => {
      mockSignUp.mockResolvedValue({
        data: { user: { id: 'new-user-id', email: 'newuser@example.com' } },
        error: null,
      });
      mockInsert.mockRejectedValueOnce(new Error('Telemetry DB Error'));

      await import('./src/signup');

      const submitHandler = buttonListeners['submit'];
      const preventDefault = vi.fn();
      await submitHandler({ preventDefault });

      expect(mockSignUp).toHaveBeenCalled();
      expect(globalThis.window.location.href).toBe('profile.html');
    });
  });

  describe('Login Page Flow', () => {
    let mockLoginForm: any;
    let buttonListeners: Record<string, Function>;

    beforeEach(() => {
      vi.resetModules();
      mockSignInWithPassword.mockReset();
      mockInsert.mockReset();

      buttonListeners = {};

      mockLoginForm = {
        addEventListener: vi.fn((event, cb) => {
          buttonListeners[event] = cb;
        }),
      };

      globalThis.window = {
        location: {
          href: '',
        },
      } as any;

      globalThis.document = {
        getElementById: vi.fn((id) => {
          if (id === 'login-form') return mockLoginForm;
          if (id === 'email') return { value: 'user@example.com' };
          if (id === 'password') return { value: 'password123' };
          return null;
        }),
      } as any;

      globalThis.alert = vi.fn();
    });

    it('submits form, inserts login telemetry, and redirects to profile.html', async () => {
      mockSignInWithPassword.mockResolvedValue({
        data: { user: { id: 'user-id', email: 'user@example.com' } },
        error: null,
      });
      mockInsert.mockResolvedValue({ data: null, error: null });

      await import('./src/login');

      const submitHandler = buttonListeners['submit'];
      expect(submitHandler).toBeDefined();

      const preventDefault = vi.fn();
      await submitHandler({ preventDefault });

      expect(preventDefault).toHaveBeenCalled();
      expect(mockSignInWithPassword).toHaveBeenCalledWith({
        email: 'user@example.com',
        password: 'password123',
      });
      expect(mockInsert).toHaveBeenCalledWith({
        player_id: 'user-id',
        event_type: 'login',
        payload: { email: 'user@example.com' },
      });
      expect(globalThis.window.location.href).toBe('profile.html');
    });

    it('redirects to profile.html even if login telemetry insert fails', async () => {
      mockSignInWithPassword.mockResolvedValue({
        data: { user: { id: 'user-id', email: 'user@example.com' } },
        error: null,
      });
      mockInsert.mockRejectedValueOnce(new Error('Telemetry DB Error'));

      await import('./src/login');

      const submitHandler = buttonListeners['submit'];
      const preventDefault = vi.fn();
      await submitHandler({ preventDefault });

      expect(mockSignInWithPassword).toHaveBeenCalled();
      expect(globalThis.window.location.href).toBe('profile.html');
    });
  });
});
