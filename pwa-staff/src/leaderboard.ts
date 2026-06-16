import { supabase } from './lib/supabaseClient';

interface Score {
  id: string;
  score: number;
  players: {
    username: string;
  } | null;
}

const leaderboardTable = document
  .getElementById('leaderboard-table')
  ?.getElementsByTagName('tbody')[0];

export const fetchScores = async () => {
  try {
    const { data, error } = await supabase
      .from('leaderboard')
      .select(
        `
                id,
                score,
                players (
                    username
                )
            `
      )
      .order('score', { ascending: false });

    if (error) {
      console.error('Error fetching scores:', error);
      return;
    }

    const scores = data as unknown as Score[];

    if (scores && leaderboardTable) {
      leaderboardTable.innerHTML = '';
      scores.forEach((score: Score, index: number) => {
        const row = leaderboardTable.insertRow();
        const playerName = score.players?.username || 'Anonymous';

        const cellRank = row.insertCell();
        cellRank.textContent = (index + 1).toString();

        const cellName = row.insertCell();
        cellName.textContent = playerName;

        const cellScore = row.insertCell();
        cellScore.textContent = score.score.toString();
      });
    }
  } catch (error) {
    console.error('Error fetching scores:', error);
  }
};

const setupRealtimeSubscription = () => {
  supabase
    .channel('public:leaderboard')
    .on('postgres_changes', { event: '*', schema: 'public', table: 'leaderboard' }, fetchScores)
    .subscribe();
};

fetchScores();
setupRealtimeSubscription();
