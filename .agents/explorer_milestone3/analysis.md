# Milestone 3 Analysis Report: Admin Dashboard & Leaderboard

## Summary
This report analyzes `pwa-staff/src/admin.ts` and `pwa-staff/src/leaderboard.ts` in relation to the Supabase database migrations to fix registered players fetching/rendering and leaderboard player name rendering. 

We identify:
1. Necessary changes in `admin.ts` to support fetching `id`, `name`, and `email` from `players`, handling `null` values cleanly (to avoid displaying `"null"` in UI), and casting fetched database responses to satisfy strict TypeScript type checks.
2. Necessary changes in `leaderboard.ts` to perform a PostgREST relation join query targeting the `players` table (to retrieve `username` since `leaderboard` lacks a `player_name` column), updating the TypeScript `Score` interface, and handling fallback rendering for missing names.

---

## 1. Database Schema & Source Code Mapping

### Database Tables (from SQL Migrations)

#### `public.players`
Created and updated in:
- `supabase/migrations/20240401000000_create_players_table.sql`
- `supabase/migrations/20240401000003_sync_auth_users_to_players.sql`

Columns:
- `id`: `UUID PRIMARY KEY` (foreign key references `auth.users(id)`)
- `username`: `VARCHAR(255) UNIQUE NOT NULL`
- `name`: `VARCHAR(255)` (nullable)
- `email`: `VARCHAR(255)` (nullable)
- `created_at`: `TIMESTAMPTZ`

#### `public.leaderboard`
Created in `supabase/migrations/20240401000002_create_leaderboard_table.sql`

Columns:
- `id`: `UUID PRIMARY KEY`
- `player_id`: `UUID REFERENCES players(id)`
- `score`: `INTEGER NOT NULL`
- `rank`: `INTEGER` (nullable)
- `created_at`: `TIMESTAMPTZ`
- `updated_at`: `TIMESTAMPTZ`

---

## 2. Issues Identified

### Issue A: Potential `"null"` rendering in `admin.ts`
The `players` table columns `name` and `email` are nullable. If a player does not have a name or email populated, the current code renders them as literal string `${player.name}` which evaluates to `"null"`. Additionally, in strict TypeScript mode, the `data` returned from Supabase queries might be `null` or contain nullable properties, which should be explicitly typed and handled.

### Issue B: Missing player names on Leaderboard
The `leaderboard` table contains no column named `player_name`. It only has `player_id` pointing to `players.id`. The frontend code in `leaderboard.ts` makes a `select('*')` query on `leaderboard` and attempts to display `score.player_name`, which is undefined, resulting in blank/missing names on the scoreboard.

---

## 3. Precise, Step-by-Step Modification Plan

### Step 1: Update `pwa-staff/src/admin.ts`

1. **Update `Player` Interface**:
   Allow `name` and `email` properties to be `string | null` to match the database schema.
   ```typescript
   interface Player {
       id: string;
       name: string | null;
       email: string | null;
   }
   ```

2. **Add Safe Type Casting and Null Checks**:
   - In `getPlayer(id)`: Ensure `data` is not null and cast it to `Player`.
     ```typescript
     async function getPlayer(id: string): Promise<Player> {
         const { data, error } = await supabase
             .from('players')
             .select('id, name, email')
             .eq('id', id)
             .single();

         if (error) {
             throw error;
         }

         if (!data) {
             throw new Error('Player not found');
         }

         return data as Player;
     }
     ```
   - In `getPlayers()`: Handle `null` data return and cast to `Player[]`.
     ```typescript
     async function getPlayers(): Promise<Player[]> {
         const { data, error } = await supabase
             .from('players')
             .select('id, name, email');

         if (error) {
             throw error;
         }

         return (data || []) as Player[];
     }
     ```

3. **Handle Null Values Safely in Rendering**:
   - In `renderPlayers`: Fallback to empty string `''` when `player.name` or `player.email` are null.
     ```typescript
     <td>${player.name || ''}</td>
     <td>${player.email || ''}</td>
     ```
   - In `handleViewClick`: Fallback when displaying in modal.
     ```typescript
     playerName.textContent = player.name || '';
     playerEmail.textContent = player.email || '';
     ```
   - In `handleEditClick`: Fallback when pre-filling the inputs.
     ```typescript
     playerNameInput.value = player.name || '';
     playerEmailInput.value = player.email || '';
     ```

---

### Step 2: Update `pwa-staff/src/leaderboard.ts`

1. **Update `Score` Interface**:
   Define `id` as `string` (corresponds to UUID) and replace `player_name: string` with a nested joined relation structure: `players: { username: string } | null`.
   ```typescript
   interface Score {
       id: string;
       score: number;
       players: {
           username: string;
       } | null;
   }
   ```

2. **Update Database Fetch Query**:
   Modify the `.select('*')` query to fetch the leaderboard fields along with the related player's `username` using PostgREST join syntax: `.select('id, score, players(username)')`.
   ```typescript
   const { data: scores, error } = await supabase
       .from('leaderboard')
       .select(`
           id,
           score,
           players (
               username
           )
       `)
       .order('score', { ascending: false });
   ```

3. **Update Table Rendering Logic**:
   Update `scores.forEach` to extract the `username` from the joined `players` object, providing a fallback (e.g. `'Anonymous'`) in case the player record is deleted or not found.
   ```typescript
   scores.forEach((score: Score, index: number) => {
       const row = leaderboardTable.insertRow();
       const playerName = score.players?.username || 'Anonymous';
       row.innerHTML = `
           <td>${index + 1}</td>
           <td>${playerName}</td>
           <td>${score.score}</td>
       `;
   });
   ```

---

## Proposed Code Diffs

### Diff for `pwa-staff/src/admin.ts`
```diff
--- pwa-staff/src/admin.ts
+++ pwa-staff/src/admin.ts
@@ -3,9 +3,9 @@
 
 interface Player {
     id: string;
-    name: string;
-    email: string;
+    name: string | null;
+    email: string | null;
 }
 
 const playerList = document.getElementById('player-list');
@@ -78,7 +78,11 @@
     if (error) {
         throw error;
     }
 
-    return data;
+    if (!data) {
+        throw new Error('Player not found');
+    }
+
+    return data as Player;
 }
 
 async function getPlayers(): Promise<Player[]> {
@@ -93,7 +97,7 @@
     if (error) {
         throw error;
     }
 
-    return data;
+    return (data || []) as Player[];
 }
 
 function renderPlayers(players: Player[]) {
@@ -115,8 +119,8 @@
         <tbody>
             ${players.map(player => `
                 <tr>
-                    <td>${player.name}</td>
-                    <td>${player.email}</td>
+                    <td>${player.name || ''}</td>
+                    <td>${player.email || ''}</td>
                     <td>
                         <button class="view-button" data-id="${player.id}">View</button>
                         <button class="edit-button" data-id="${player.id}">Edit</button>
@@ -138,8 +142,8 @@
             const playerEmail = document.getElementById('view-player-email');
 
             if (playerName && playerEmail && viewModal) {
-                playerName.textContent = player.name;
-                playerEmail.textContent = player.email;
+                playerName.textContent = player.name || '';
+                playerEmail.textContent = player.email || '';
                 viewModal.style.display = 'block';
             }
         }
@@ -157,9 +161,9 @@
             const playerEmailInput = document.getElementById('edit-player-email') as HTMLInputElement;
 
             if (playerIdInput && playerNameInput && playerEmailInput && editModal) {
                 playerIdInput.value = player.id;
-                playerNameInput.value = player.name;
-                playerEmailInput.value = player.email;
+                playerNameInput.value = player.name || '';
+                playerEmailInput.value = player.email || '';
                 editModal.style.display = 'block';
             }
         }
```

### Diff for `pwa-staff/src/leaderboard.ts`
```diff
--- pwa-staff/src/leaderboard.ts
+++ pwa-staff/src/leaderboard.ts
@@ -2,19 +2,25 @@
 
 interface Score {
-    id: number;
-    player_name: string;
-    score: number;
+    id: string;
+    score: number;
+    players: {
+        username: string;
+    } | null;
 }
 
 const leaderboardTable = document.getElementById('leaderboard-table')?.getElementsByTagName('tbody')[0];
 
 const fetchScores = async () => {
     const { data: scores, error } = await supabase
         .from('leaderboard')
-        .select('*')
+        .select(`
+            id,
+            score,
+            players (
+                username
+            )
+        `)
         .order('score', { ascending: false });
 
     if (error) {
@@ -24,9 +30,10 @@
     if (scores && leaderboardTable) {
         leaderboardTable.innerHTML = '';
         scores.forEach((score: Score, index: number) => {
             const row = leaderboardTable.insertRow();
+            const playerName = score.players?.username || 'Anonymous';
             row.innerHTML = `
                 <td>${index + 1}</td>
-                <td>${score.player_name}</td>
+                <td>${playerName}</td>
                 <td>${score.score}</td>
             `;
         });
```
