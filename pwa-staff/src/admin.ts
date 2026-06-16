import { supabase } from './lib/supabaseClient';

interface Player {
  id: string;
  name: string | null;
  email: string | null;
}

const playerList = document.getElementById('player-list');
const gameStateViewer = document.getElementById('game-state-viewer');
const viewModal = document.getElementById('view-modal');
const editModal = document.getElementById('edit-modal');
const editForm = document.getElementById('edit-form');

interface GameSession {
  [key: string]: any;
}

async function getGameSessions(): Promise<GameSession[]> {
  const { data, error } = await supabase.from('game_sessions').select('*');

  if (error) {
    throw error;
  }

  return data;
}

function renderGameSessions(gameSessions: GameSession[]) {
  if (!gameStateViewer) {
    return;
  }

  gameStateViewer.innerHTML = '';

  if (gameSessions.length === 0) {
    gameStateViewer.innerHTML = '<p>No game sessions found.</p>';
    return;
  }

  const table = document.createElement('table');
  const thead = document.createElement('thead');
  const tbody = document.createElement('tbody');
  const headers = Object.keys(gameSessions[0]);

  const headerRow = document.createElement('tr');
  headers.forEach((headerText) => {
    const th = document.createElement('th');
    th.textContent = headerText;
    headerRow.appendChild(th);
  });
  thead.appendChild(headerRow);

  gameSessions.forEach((session) => {
    const row = document.createElement('tr');
    headers.forEach((header) => {
      const cell = document.createElement('td');
      const cellValue = session[header];
      cell.textContent =
        typeof cellValue === 'object' ? JSON.stringify(cellValue, null, 2) : cellValue;
      row.appendChild(cell);
    });
    tbody.appendChild(row);
  });

  table.appendChild(thead);
  table.appendChild(tbody);
  gameStateViewer.appendChild(table);
}

async function getPlayer(id: string): Promise<Player> {
  const { data, error } = await supabase
    .from('players')
    .select('id, name, email')
    .eq('id', id)
    .single();

  if (error) {
    throw error;
  }

  return data as Player;
}

async function getPlayers(): Promise<Player[]> {
  const { data, error } = await supabase.from('players').select('id, name, email');

  if (error) {
    throw error;
  }

  return (data || []) as Player[];
}

export function renderPlayers(players: Player[]) {
  if (!playerList) {
    return;
  }

  playerList.innerHTML = '';

  const table = document.createElement('table');
  const thead = document.createElement('thead');
  const headerRow = document.createElement('tr');

  const thName = document.createElement('th');
  thName.textContent = 'Name';
  headerRow.appendChild(thName);

  const thEmail = document.createElement('th');
  thEmail.textContent = 'Email';
  headerRow.appendChild(thEmail);

  const thActions = document.createElement('th');
  thActions.textContent = 'Actions';
  headerRow.appendChild(thActions);

  thead.appendChild(headerRow);
  table.appendChild(thead);

  const tbody = document.createElement('tbody');
  players.forEach((player) => {
    const row = document.createElement('tr');

    const tdName = document.createElement('td');
    tdName.textContent = player.name ?? '';
    row.appendChild(tdName);

    const tdEmail = document.createElement('td');
    tdEmail.textContent = player.email ?? '';
    row.appendChild(tdEmail);

    const tdActions = document.createElement('td');

    const viewButton = document.createElement('button');
    viewButton.className = 'view-button';
    viewButton.dataset.id = player.id;
    viewButton.textContent = 'View';
    tdActions.appendChild(viewButton);

    const space = document.createTextNode(' ');
    tdActions.appendChild(space);

    const editButton = document.createElement('button');
    editButton.className = 'edit-button';
    editButton.dataset.id = player.id;
    editButton.textContent = 'Edit';
    tdActions.appendChild(editButton);

    row.appendChild(tdActions);
    tbody.appendChild(row);
  });
  table.appendChild(tbody);
  playerList.appendChild(table);
}

export async function handleViewClick(event: Event) {
  const target = event.target as HTMLElement;
  if (target.classList.contains('view-button')) {
    const playerId = target.dataset.id;
    if (playerId) {
      try {
        const player = await getPlayer(playerId);
        const playerName = document.getElementById('view-player-name');
        const playerEmail = document.getElementById('view-player-email');

        if (playerName && playerEmail && viewModal) {
          playerName.textContent = player.name ?? '';
          playerEmail.textContent = player.email ?? '';
          viewModal.style.display = 'block';
        }
      } catch (error) {
        console.error('Error fetching player details:', error);
      }
    }
  }
}

export async function handleEditClick(event: Event) {
  const target = event.target as HTMLElement;
  if (target.classList.contains('edit-button')) {
    const playerId = target.dataset.id;
    if (playerId) {
      try {
        const player = await getPlayer(playerId);
        const playerIdInput = document.getElementById('edit-player-id') as HTMLInputElement;
        const playerNameInput = document.getElementById('edit-player-name') as HTMLInputElement;
        const playerEmailInput = document.getElementById('edit-player-email') as HTMLInputElement;

        if (playerIdInput && playerNameInput && playerEmailInput && editModal) {
          playerIdInput.value = player.id;
          playerNameInput.value = player.name ?? '';
          playerEmailInput.value = player.email ?? '';
          editModal.style.display = 'block';
        }
      } catch (error) {
        console.error('Error fetching player for editing:', error);
      }
    }
  }
}

async function handleEditFormSubmit(event: Event) {
  event.preventDefault();
  const playerIdInput = document.getElementById('edit-player-id') as HTMLInputElement;
  const playerNameInput = document.getElementById('edit-player-name') as HTMLInputElement;
  const playerEmailInput = document.getElementById('edit-player-email') as HTMLInputElement;

  if (playerIdInput && playerNameInput && playerEmailInput) {
    try {
      const { error } = await supabase
        .from('players')
        .update({ name: playerNameInput.value, email: playerEmailInput.value })
        .eq('id', playerIdInput.value);

      if (error) {
        console.error('Error updating player:', error);
        alert('Error updating player: ' + error.message);
      } else {
        if (editModal) {
          editModal.style.display = 'none';
        }
        init();
      }
    } catch (error) {
      console.error('Error updating player:', error);
      alert('Error updating player: ' + (error instanceof Error ? error.message : String(error)));
    }
  }
}

function closeModal() {
  if (viewModal) {
    viewModal.style.display = 'none';
  }
  if (editModal) {
    editModal.style.display = 'none';
  }
}

async function init() {
  try {
    const players = await getPlayers();
    renderPlayers(players);
  } catch (error) {
    console.error('Error fetching players:', error);
    if (playerList) {
      playerList.innerHTML = '<p>Error fetching players. Please check the console for details.</p>';
    }
  }

  try {
    const gameSessions = await getGameSessions();
    renderGameSessions(gameSessions);
  } catch (error) {
    console.error('Error fetching game sessions:', error);
    if (gameStateViewer) {
      gameStateViewer.innerHTML =
        '<p>Error fetching game sessions. Please check the console for details.</p>';
    }
  }
}

playerList?.addEventListener('click', (event) => {
  handleViewClick(event);
  handleEditClick(event);
});
editForm?.addEventListener('submit', handleEditFormSubmit);
viewModal?.addEventListener('click', (event) => {
  if ((event.target as HTMLElement).classList.contains('close-button')) {
    closeModal();
  }
});
editModal?.addEventListener('click', (event) => {
  if ((event.target as HTMLElement).classList.contains('close-button')) {
    closeModal();
  }
});

init();
