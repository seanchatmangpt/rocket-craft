import { supabase } from './lib/supabaseClient';

const userEmailElement = document.getElementById('user-email');
const logoutButton = document.getElementById('logout-button');

async function initProfile() {
  let user: any = null;
  try {
    const { data, error } = await supabase.auth.getUser();
    if (!error && data && data.user) {
      user = data.user;
    }
  } catch (e) {
    console.warn('getUser failed, attempting fallback to local session:', e);
  }

  if (!user) {
    try {
      const { data } = await supabase.auth.getSession();
      if (data && data.session && data.session.user) {
        user = data.session.user;
        console.log('Using local session for offline access:', user.email);
      }
    } catch (e) {
      console.error('getSession fallback failed:', e);
    }
  }

  if (!user) {
    window.location.href = 'login.html';
    return;
  }

  if (userEmailElement) {
    userEmailElement.textContent = user.email || '';
  }

  const { data: playerData } = await supabase
    .from('players')
    .select('username')
    .eq('id', user.id)
    .single();

  try {
    const { error: telemetryError } = await supabase.from('telemetry_logs').insert({
      player_id: user.id,
      event_type: 'profile_view',
      payload: {
        player_id: user.id,
        email: user.email || '',
        username: playerData?.username || '',
      },
    });
    if (telemetryError) {
      console.error('Telemetry logging error during profile view:', telemetryError);
    }
  } catch (telemetryError) {
    console.error('Telemetry logging error during profile view:', telemetryError);
  }

  try {
    const origin = typeof window !== 'undefined' && window.location && window.location.origin ? window.location.origin : 'http://localhost:3000';
    const baseUrl = (origin && origin !== 'null') ? origin : 'http://localhost:3000';
    const specRes = await fetch(`${baseUrl}/api/spec`);
    if (specRes.ok) {
      const spec = await specRes.json();
      if (spec.receipts && spec.receipts.length > 0) {
        const latest = spec.receipts[spec.receipts.length - 1];
        const receiptDetailsDiv = document.getElementById('receipt-details');
        if (receiptDetailsDiv) {
          receiptDetailsDiv.innerHTML = `
            <p><strong>Hash:</strong> ${latest.hash}</p>
            <p><strong>Issued At:</strong> ${new Date(latest.issued_at).toLocaleString()}</p>
          `;
        }
      }
      const { error: upsertError } = await supabase.from('world_specs').upsert({
        player_id: user.id,
        spec: spec,
        updated_at: new Date().toISOString()
      }, { onConflict: 'player_id' });

      if (upsertError) {
        console.error('Error saving world spec to Supabase:', upsertError);
      }
    } else {
      console.error('Failed to fetch /api/spec, status:', specRes.status);
    }
  } catch (err) {
    console.error('Failed to fetch or save world spec:', err);
  }

  if (typeof document !== 'undefined' && typeof document.createElement === 'function' && document.body) {
    try {
      const script = document.createElement('script');
      script.src = '/manufactured/Brm-HTML5-Shipping.js';
      document.body.appendChild(script);
    } catch (scriptError) {
      console.error('Failed to load simulator script dynamically:', scriptError);
    }
  }
}

initProfile().catch((err) => {
  console.error('Failed to initialize profile:', err);
  window.location.href = 'login.html';
});

if (logoutButton) {
  logoutButton.addEventListener('click', async () => {
    try {
      const { error } = await supabase.auth.signOut();

      if (error) {
        alert(error.message);
      } else {
        window.location.href = 'login.html';
      }
    } catch (error) {
      console.error('Logout error:', error);
      alert(error instanceof Error ? error.message : String(error));
    }
  });
}
