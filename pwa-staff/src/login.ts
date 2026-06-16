import { supabase } from './lib/supabaseClient';

const loginForm = document.getElementById('login-form');

loginForm!.addEventListener('submit', async (event) => {
  event.preventDefault();

  const email = (document.getElementById('email') as HTMLInputElement).value;
  const password = (document.getElementById('password') as HTMLInputElement).value;

  try {
    const { data, error } = await supabase.auth.signInWithPassword({
      email,
      password,
    });

    if (error) {
      alert(error.message);
    } else {
      const player_id = data.user?.id;
      try {
        const { error: telemetryError } = await supabase.from('telemetry_logs').insert({
          player_id,
          event_type: 'login',
          payload: { email },
        });
        if (telemetryError) {
          console.error('Telemetry logging error during login:', telemetryError);
        }
      } catch (telemetryError) {
        console.error('Telemetry logging error during login:', telemetryError);
      }
      window.location.href = 'profile.html';
    }
  } catch (error) {
    console.error('Login error:', error);
    alert(error instanceof Error ? error.message : String(error));
  }
});
