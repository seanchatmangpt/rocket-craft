# Scope: Milestone 3 — Admin Dashboard & Leaderboard

## Architecture
- **PWA Staff Frontend App**: Static HTML files loaded with bundled TypeScript assets from `dist/`.
- **Database Backend**: PostgreSQL on Supabase, with tables:
  - `public.players`: contains registered player data (columns: `id`, `username`, `name`, `email`, `created_at`).
  - `public.leaderboard`: contains scores linked to players (columns: `id`, `player_id`, `score`, `rank`, `created_at`, `updated_at`).
- **Data Flow**:
  - `admin.ts` fetches players (`id`, `name`, `email`) from the `players` table and renders them in the Player Management table. It allows editing and viewing players.
  - `leaderboard.ts` fetches scores from the `leaderboard` table, joined with `players` to retrieve the player's `username`, and renders them sorted by score in descending order.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | Exploration | Analyze codebase files (`admin.ts`, `leaderboard.ts`), schemas, and define modification strategy | None | PLANNED |
| 2 | Implementation | Update TypeScript files (`admin.ts`, `leaderboard.ts`), compile frontend bundle, check types/build | Exploration | PLANNED |
| 3 | Review | Verify queries, logic correctness, RLS compliance, and lint checks | Implementation | PLANNED |
| 4 | Challenger / Audit | Run tests, stress-test rendering, run Forensic Auditor checks | Review | PLANNED |

## Interface Contracts
### Supabase Client ↔ public.players Table
- **Select**: Query `id`, `name`, `email`.
- **Update**: Update `name`, `email` by `id`.

### Supabase Client ↔ public.leaderboard Joined with public.players Table
- **Select**: Query leaderboard scores (`score`) joined with players' `username`.
- **Expected Data Shape**:
  ```typescript
  interface Score {
      id: string;
      score: number;
      players: {
          username: string;
      } | null;
  }
  ```
