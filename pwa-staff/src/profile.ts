import { supabase } from './lib/supabaseClient';

const userEmailElement = document.getElementById('user-email');
const logoutButton = document.getElementById('logout-button');

async function initProfile() {
  const {
    data: { user },
    error,
  } = await supabase.auth.getUser();

  if (error || !user) {
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
