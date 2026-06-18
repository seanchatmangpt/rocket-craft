import { supabase } from './lib/supabaseClient';

interface User {
  name: string;
  email: string;
}

interface Session {
  user: User;
  token: string;
}

let currentSession: Session | null = null;

function dispatchAuthChange() {
  window.dispatchEvent(new CustomEvent('auth-change', { detail: { session: currentSession } }));
}

function initializeAuth() {
  // Fetch initial session
  supabase.auth
    .getSession()
    .then(({ data: { session } }) => {
      if (session && session.user) {
        currentSession = {
          user: {
            name: session.user.email?.split('@')[0] || 'User',
            email: session.user.email || '',
          },
          token: session.access_token,
        };
      } else {
        currentSession = null;
      }
      dispatchAuthChange();
    })
    .catch((error) => {
      console.error('Error fetching initial session:', error);
    });

  // Listen to Supabase auth changes
  supabase.auth.onAuthStateChange((event, session) => {
    if (session && session.user) {
      currentSession = {
        user: {
          name: session.user.email?.split('@')[0] || 'User',
          email: session.user.email || '',
        },
        token: session.access_token,
      };
    } else {
      currentSession = null;
    }
    dispatchAuthChange();
  });
}

export function getSession(): Session | null {
  return currentSession;
}

export function login(user: User, token: string) {
  currentSession = { user, token };
  dispatchAuthChange();
}

/** Sign in via Supabase with email and password, updating local session state. */
export async function loginWithCredentials(email: string, password: string): Promise<void> {
  const { data, error } = await supabase.auth.signInWithPassword({ email, password });
  if (error) {
    throw error;
  }
  if (data.session && data.user) {
    currentSession = {
      user: {
        name: data.user.email?.split('@')[0] || 'User',
        email: data.user.email || '',
      },
      token: data.session.access_token,
    };
    dispatchAuthChange();
  }
}

export function logout() {
  supabase.auth
    .signOut()
    .then(() => {
      currentSession = null;
      dispatchAuthChange();
    })
    .catch((error) => {
      console.error('Error signing out:', error);
    });
}

export function useAuth(callback: (session: Session | null) => void): () => void {
  const handler = (event: Event) => {
    callback((event as CustomEvent).detail.session);
  };

  window.addEventListener('auth-change', handler);
  callback(currentSession);

  return () => {
    window.removeEventListener('auth-change', handler);
  };
}

initializeAuth();
