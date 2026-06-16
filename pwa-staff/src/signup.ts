import { supabase } from './lib/supabaseClient';

const signupForm = document.getElementById('signup-form');

signupForm!.addEventListener('submit', async (event) => {
  event.preventDefault();

  const email = (document.getElementById('email') as HTMLInputElement).value;
  const password = (document.getElementById('password') as HTMLInputElement).value;

  try {
    const { data, error } = await supabase.auth.signUp({
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
          event_type: 'registration',
          payload: { email },
        });
        if (telemetryError) {
          console.error('Telemetry logging error during signup:', telemetryError);
        }
      } catch (telemetryError) {
        console.error('Telemetry logging error during signup:', telemetryError);
      }
      window.location.href = 'profile.html';
    }
  } catch (error) {
    console.error('Signup error:', error);
    alert(error instanceof Error ? error.message : String(error));
  }
});
