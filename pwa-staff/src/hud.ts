import { supabase } from './lib/supabaseClient';
import { supabaseUrl } from './config';

if (typeof window !== 'undefined') {
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => initHUD());
  } else {
    initHUD();
  }
}

export function initHUD() {
  // 1. Inject Styles
  const styleEl = document.createElement('style');
  styleEl.textContent = `
    .hud-toggle-btn {
      position: fixed;
      bottom: 20px;
      right: 20px;
      z-index: 9999;
      background-color: #0a0b0e;
      color: #00f0ff;
      border: 2px solid #00f0ff;
      border-radius: 8px;
      padding: 10px 18px;
      font-family: 'Courier New', Courier, Monaco, Consolas, monospace;
      font-weight: bold;
      cursor: pointer;
      box-shadow: 0 0 10px rgba(0, 240, 255, 0.4);
      transition: all 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
      text-transform: uppercase;
      letter-spacing: 1.5px;
      outline: none;
    }
    .hud-toggle-btn:hover {
      color: #ff007f;
      border-color: #ff007f;
      box-shadow: 0 0 15px rgba(255, 0, 127, 0.8);
      transform: translateY(-2px);
    }
    .hud-drawer {
      position: fixed;
      top: 0;
      right: 0;
      width: 380px;
      max-width: 100vw;
      height: 100vh;
      z-index: 9998;
      background-color: rgba(10, 11, 14, 0.95);
      border-left: 2px solid #00f0ff;
      box-shadow: -5px 0 25px rgba(0, 0, 0, 0.9);
      transition: transform 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
      overflow-y: auto;
      padding: 25px;
      display: flex;
      flex-direction: column;
      gap: 20px;
      font-family: 'Courier New', Courier, Monaco, Consolas, monospace;
      color: #e2e8f0;
      box-sizing: border-box;
    }
    .hud-drawer.hud-collapsed {
      transform: translateX(100%);
    }
    .hud-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      border-bottom: 1px solid rgba(0, 240, 255, 0.3);
      padding-bottom: 10px;
    }
    .hud-header h3 {
      margin: 0;
      font-size: 1.4em;
      color: #00f0ff;
      text-shadow: 0 0 5px rgba(0, 240, 255, 0.5);
      letter-spacing: 2px;
    }
    .hud-close-btn {
      background: none;
      border: none;
      color: #94a3b8;
      font-size: 28px;
      font-weight: bold;
      cursor: pointer;
      transition: color 0.3s;
      line-height: 1;
      padding: 0;
    }
    .hud-close-btn:hover {
      color: #ff007f;
    }
    .hud-section {
      display: flex;
      flex-direction: column;
      gap: 10px;
      border: 1px solid rgba(0, 240, 255, 0.15);
      border-radius: 8px;
      padding: 15px;
      background-color: rgba(13, 16, 23, 0.6);
    }
    .hud-section h4 {
      margin: 0 0 5px 0;
      font-size: 0.95em;
      color: #ff007f;
      text-shadow: 0 0 3px rgba(255, 0, 127, 0.4);
      letter-spacing: 1px;
      text-transform: uppercase;
    }
    .hud-data-group {
      display: flex;
      flex-direction: column;
      gap: 8px;
    }
    .hud-data-row {
      display: flex;
      justify-content: space-between;
      font-size: 0.9em;
      word-break: break-all;
      gap: 10px;
    }
    .hud-data-label {
      color: #94a3b8;
      flex-shrink: 0;
    }
    .hud-value {
      color: #00f0ff;
      font-weight: bold;
      text-align: right;
    }
    .hud-value.unauthenticated {
      color: #ff007f;
      text-shadow: 0 0 3px rgba(255, 0, 127, 0.4);
      text-align: left;
    }
    .hud-action-btn {
      background-color: #00f0ff;
      color: #0a0b0e;
      border: 1px solid #00f0ff;
      border-radius: 6px;
      padding: 10px;
      font-size: 0.9em;
      font-weight: bold;
      cursor: pointer;
      transition: all 0.3s;
      text-transform: uppercase;
      letter-spacing: 1px;
      font-family: inherit;
      margin-top: 5px;
      outline: none;
    }
    .hud-action-btn:hover {
      background-color: #ff007f;
      border-color: #ff007f;
      color: #ffffff;
      box-shadow: 0 0 10px rgba(255, 0, 127, 0.8);
    }
    .hud-action-btn.neon-pink-btn {
      background-color: transparent;
      color: #ff007f;
      border-color: #ff007f;
      box-shadow: 0 0 5px rgba(255, 0, 127, 0.3);
    }
    .hud-action-btn.neon-pink-btn:hover {
      background-color: #ff007f;
      color: #ffffff;
      box-shadow: 0 0 15px rgba(255, 0, 127, 0.8);
    }
    .hud-console-log {
      background-color: #050508;
      border: 1px solid rgba(0, 240, 255, 0.2);
      border-radius: 6px;
      height: 120px;
      overflow-y: auto;
      padding: 10px;
      font-size: 0.8em;
      color: #39ff14;
      display: flex;
      flex-direction: column;
      gap: 6px;
      box-shadow: inset 0 2px 5px rgba(0,0,0,0.8);
    }
    .hud-log-entry {
      line-height: 1.4;
      word-break: break-all;
    }
    .hud-log-entry.error {
      color: #ff007f;
    }
    .hud-log-entry.system {
      color: #fffe03;
    }
  `;
  document.head.appendChild(styleEl);

  // 2. Create Floating Toggle Button
  const toggleBtn = document.createElement('button');
  toggleBtn.className = 'hud-toggle-btn';
  toggleBtn.textContent = '🛠️ HUD';
  document.body.appendChild(toggleBtn);

  // 3. Create Side Drawer Container
  const drawer = document.createElement('div');
  drawer.className = 'hud-drawer hud-collapsed';

  // Header
  const header = document.createElement('div');
  header.className = 'hud-header';

  const title = document.createElement('h3');
  title.textContent = 'SYSTEM HUD';

  const closeBtn = document.createElement('button');
  closeBtn.className = 'hud-close-btn';
  closeBtn.textContent = '×';

  header.appendChild(title);
  header.appendChild(closeBtn);
  drawer.appendChild(header);

  // Active Session Section
  const authSection = document.createElement('div');
  authSection.className = 'hud-section';
  const authTitle = document.createElement('h4');
  authTitle.textContent = 'Active Session';
  const authDetails = document.createElement('div');
  authDetails.className = 'hud-data-group';
  authDetails.id = 'hud-auth-details';
  authSection.appendChild(authTitle);
  authSection.appendChild(authDetails);
  drawer.appendChild(authSection);

  // Database Stats Section
  const statsSection = document.createElement('div');
  statsSection.className = 'hud-section';
  const statsTitle = document.createElement('h4');
  statsTitle.textContent = 'Database Stats';

  const playersRow = document.createElement('div');
  playersRow.className = 'hud-data-row';
  const playersLabel = document.createElement('span');
  playersLabel.className = 'hud-data-label';
  playersLabel.textContent = 'Registered Players:';
  const playersVal = document.createElement('span');
  playersVal.className = 'hud-value';
  playersVal.id = 'hud-stats-players';
  playersVal.textContent = 'Loading...';
  playersRow.appendChild(playersLabel);
  playersRow.appendChild(playersVal);

  const sessionsRow = document.createElement('div');
  sessionsRow.className = 'hud-data-row';
  const sessionsLabel = document.createElement('span');
  sessionsLabel.className = 'hud-data-label';
  sessionsLabel.textContent = 'Total Game Sessions:';
  const sessionsVal = document.createElement('span');
  sessionsVal.className = 'hud-value';
  sessionsVal.id = 'hud-stats-sessions';
  sessionsVal.textContent = 'Loading...';
  sessionsRow.appendChild(sessionsLabel);
  sessionsRow.appendChild(sessionsVal);

  const refreshStatsBtn = document.createElement('button');
  refreshStatsBtn.className = 'hud-action-btn';
  refreshStatsBtn.id = 'dev-hud-refresh-stats-btn';
  refreshStatsBtn.textContent = 'Refresh Stats';

  statsSection.appendChild(statsTitle);
  statsSection.appendChild(playersRow);
  statsSection.appendChild(sessionsRow);
  statsSection.appendChild(refreshStatsBtn);
  drawer.appendChild(statsSection);

  // Quick Triggers Section
  const triggerSection = document.createElement('div');
  triggerSection.className = 'hud-section';
  const triggerTitle = document.createElement('h4');
  triggerTitle.textContent = 'Quick Triggers';

  const submitScoreBtn = document.createElement('button');
  submitScoreBtn.className = 'hud-action-btn neon-pink-btn';
  submitScoreBtn.id = 'dev-hud-submit-score-btn';
  submitScoreBtn.textContent = 'Submit Mock Score';

  triggerSection.appendChild(triggerTitle);
  triggerSection.appendChild(submitScoreBtn);
  drawer.appendChild(triggerSection);

  // System Console Log Section
  const consoleSection = document.createElement('div');
  consoleSection.className = 'hud-section';
  const consoleTitle = document.createElement('h4');
  consoleTitle.textContent = 'System Console';

  const consoleLog = document.createElement('div');
  consoleLog.className = 'hud-console-log';
  consoleLog.id = 'dev-hud-console';
  consoleSection.appendChild(consoleTitle);
  consoleSection.appendChild(consoleLog);
  drawer.appendChild(consoleSection);

  document.body.appendChild(drawer);

  // 4. Set up Event Listeners
  toggleBtn.addEventListener('click', () => {
    drawer.classList.toggle('hud-collapsed');
  });

  closeBtn.addEventListener('click', () => {
    drawer.classList.add('hud-collapsed');
  });

  // Logging function
  function logToConsole(message: string, type: 'info' | 'error' | 'system' = 'info') {
    const entry = document.createElement('div');
    entry.className = `hud-log-entry ${type}`;
    const timestamp = new Date().toLocaleTimeString();
    entry.textContent = `[${timestamp}] ${message}`;
    consoleLog.appendChild(entry);
    consoleLog.scrollTop = consoleLog.scrollHeight;
    if (type === 'error') {
      console.error(`[HUD] ${message}`);
    } else {
      console.log(`[HUD] ${message}`);
    }
  }

  logToConsole('HUD initialized successfully.', 'system');

  // Decode JWT function
  function decodeJWT(token: string) {
    try {
      const base64Url = token.split('.')[1];
      const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/');
      const padLength = (4 - (base64.length % 4)) % 4;
      const paddedBase64 = base64 + '='.repeat(padLength);
      const decoded = JSON.parse(atob(paddedBase64));
      return {
        email: decoded.email || 'N/A',
        user_id: decoded.sub || 'N/A',
        role: decoded.role || 'N/A',
        exp: decoded.exp ? new Date(decoded.exp * 1000).toLocaleString() : 'N/A',
      };
    } catch (error) {
      logToConsole(
        `Failed to decode JWT: ${error instanceof Error ? error.message : error}`,
        'error'
      );
      return null;
    }
  }

  // Update Auth State
  async function updateAuthState(session: any) {
    authDetails.textContent = ''; // Clear existing content safely
    if (!session) {
      const unauthText = document.createElement('div');
      unauthText.className = 'hud-value unauthenticated';
      unauthText.textContent = 'Unauthenticated';
      authDetails.appendChild(unauthText);
      logToConsole('Session state: Unauthenticated', 'system');
      return;
    }

    const decoded = decodeJWT(session.access_token);
    if (!decoded) {
      const errorText = document.createElement('div');
      errorText.className = 'hud-log-entry error';
      errorText.textContent = 'Invalid Session JWT';
      authDetails.appendChild(errorText);
      return;
    }

    const fields = [
      { label: 'Email', value: decoded.email },
      { label: 'User ID', value: decoded.user_id },
      { label: 'Role', value: decoded.role },
      { label: 'Expires', value: decoded.exp },
    ];

    fields.forEach((f) => {
      const row = document.createElement('div');
      row.className = 'hud-data-row';
      const lbl = document.createElement('span');
      lbl.className = 'hud-data-label';
      lbl.textContent = `${f.label}:`;
      const val = document.createElement('span');
      val.className = 'hud-value';
      val.textContent = f.value;
      row.appendChild(lbl);
      row.appendChild(val);
      authDetails.appendChild(row);
    });

    logToConsole(`Session state: Authenticated (${decoded.email})`, 'system');
  }

  // Fetch Database Stats
  async function fetchDbStats() {
    try {
      playersVal.textContent = 'Fetching...';
      sessionsVal.textContent = 'Fetching...';

      const { count: playersCount, error: playersErr } = await supabase
        .from('players')
        .select('*', { count: 'exact', head: true });

      if (playersErr) throw playersErr;
      playersVal.textContent = playersCount !== null ? playersCount.toString() : '0';

      const { count: sessionsCount, error: sessionsErr } = await supabase
        .from('game_sessions')
        .select('*', { count: 'exact', head: true });

      if (sessionsErr) throw sessionsErr;
      sessionsVal.textContent = sessionsCount !== null ? sessionsCount.toString() : '0';

      logToConsole(`Stats refreshed: ${playersCount} players, ${sessionsCount} sessions`, 'info');
    } catch (err) {
      const errMsg = err instanceof Error ? err.message : String(err);
      playersVal.textContent = 'Error';
      sessionsVal.textContent = 'Error';
      logToConsole(`Failed to fetch database stats: ${errMsg}`, 'error');
    }
  }

  // Submit Mock Score
  async function submitMockScore() {
    try {
      const {
        data: { session },
      } = await supabase.auth.getSession();
      if (!session) {
        logToConsole('Cannot submit score: Unauthenticated', 'error');
        alert('You must be logged in to submit a score.');
        return;
      }

      const score = Math.floor(Math.random() * 501) + 500; // 500-1000
      logToConsole(`Submitting mock score: ${score}...`, 'info');

      const response = await fetch(`${supabaseUrl}/functions/v1/submit-score`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${session.access_token}`,
        },
        body: JSON.stringify({ score }),
      });

      const responseText = await response.text();
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}, message: ${responseText}`);
      }

      logToConsole(`Mock score submission successful: ${responseText}`, 'info');
    } catch (err) {
      const errMsg = err instanceof Error ? err.message : String(err);
      logToConsole(`Failed to submit mock score: ${errMsg}`, 'error');
    }
  }

  // Register refresh and submit click triggers
  refreshStatsBtn.addEventListener('click', fetchDbStats);
  submitScoreBtn.addEventListener('click', submitMockScore);

  // Initialize Auth state check and subscription
  supabase.auth.getSession().then(({ data: { session } }) => {
    updateAuthState(session);
  });

  supabase.auth.onAuthStateChange((_event, session) => {
    updateAuthState(session);
  });

  // Initial fetch of DB stats
  fetchDbStats();

  // Listen for real game score updates from the WASM game worker.
  // Dispatched by the game bridge as: window.dispatchEvent(new CustomEvent('game-score-update', { detail: { score, tick } }))
  window.addEventListener('game-score-update', async (e: Event) => {
    const event = e as CustomEvent<{ score: number; tick: number }>;
    const realScore = event.detail.score;
    if (realScore <= 0) return;
    try {
      const {
        data: { session },
      } = await supabase.auth.getSession();
      if (!session) return;
      const response = await fetch(`${supabaseUrl}/functions/v1/submit-score`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${session.access_token}`,
        },
        body: JSON.stringify({ score: realScore }),
      });
      if (response.ok) {
        logToConsole(`Auto-submitted game score: ${realScore} (tick ${event.detail.tick})`, 'info');
      }
    } catch (err) {
      logToConsole(
        `Failed to auto-submit game score: ${err instanceof Error ? err.message : String(err)}`,
        'error'
      );
    }
  });
}
