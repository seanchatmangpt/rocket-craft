# Handoff Report: Supabase Migration Investigation & Design

## 1. Observation
We examined the Supabase migrations in `/Users/sac/rocket-craft/supabase/migrations/` and verified the following:

- **Migration Files**:
  1. `/Users/sac/rocket-craft/supabase/migrations/20240401000000_create_players_table.sql`
     ```sql
     CREATE TABLE players (
         id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
         username VARCHAR(255) UNIQUE NOT NULL,
         created_at TIMESTAMPTZ DEFAULT NOW()
     );
     ```
  2. `/Users/sac/rocket-craft/supabase/migrations/20240401000001_create_game_sessions_table.sql`
     ```sql
     CREATE TABLE game_sessions (
         id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
         player_id UUID REFERENCES players(id) ON DELETE CASCADE,
         score INTEGER NOT NULL,
         created_at TIMESTAMPTZ DEFAULT NOW()
     );
     ```
  3. `/Users/sac/rocket-craft/supabase/migrations/20240401000002_create_leaderboard_table.sql`
     ```sql
     CREATE TABLE leaderboard (
         id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
         player_id UUID REFERENCES players(id) ON DELETE CASCADE,
         score INTEGER NOT NULL,
         rank INTEGER,
         created_at TIMESTAMPTZ DEFAULT NOW(),
         updated_at TIMESTAMPTZ DEFAULT NOW()
     );
     -- Trigger logic for leaderboard update...
     ```

- **Frontend Staff Application Queries**:
  In `/Users/sac/rocket-craft/pwa-staff/src/admin.ts`, lines 87-97 query the `players` table expecting `id`, `name`, and `email` columns:
  ```typescript
  async function getPlayers(): Promise<Player[]> {
      const { data, error } = await supabase
          .from('players')
          .select('id, name, email');
      ...
  ```

- **Rust SDK Struct**:
  In `/Users/sac/rocket-craft/tools/rocket-sdk/src/supabase.rs`, lines 5-10 define:
  ```rust
  #[derive(Serialize, Deserialize, Debug)]
  pub struct Player {
      pub id: i64,
      pub name: String,
      pub score: i64,
  }
  ```
  *Note:* The Rust struct uses `id: i64` which is mismatched with the SQL schema's `UUID` primary key, and it does not define `email`.

---

## 2. Logic Chain
1. **Adding Columns**: The `pwa-staff` application queries `name` and `email` fields on `public.players`. Therefore, the table must be updated to support these columns (e.g., `email VARCHAR(255)` and `name VARCHAR(255)`).
2. **Synchronization Trigger**: Upon user registration, Supabase creates a user entry in `auth.users`. To automate syncing this user to the `public.players` table without application-level logic overhead, we require a PostgreSQL trigger function (`public.handle_new_user()`) running `AFTER INSERT ON auth.users`.
3. **Security Context**: The trigger needs to perform insertions on `public.players` even if the initiating user has limited public schema permissions. Thus, the trigger function must be executed with `SECURITY DEFINER`. To prevent search path hijacking, the search path is strictly locked: `SECURITY DEFINER SET search_path = public`.
4. **Collision and Edge Cases Handling**:
   - `public.players.username` is configured with a `UNIQUE` constraint.
   - Extracting `username` directly from email prefix (e.g. `john` from `john@domain.com`) could result in unique constraint violations if another user registration already claimed `john`.
   - To avoid blocking user registration, a collision loop checks if the proposed `username` exists. If so, it appends a sequential counter (e.g. `_1`, `_2`). If conflicts persist after 100 loops, it appends a random hash suffix (using `md5(random()::TEXT)`).
   - If a user registers without an email or metadata (e.g. anonymous sign-ups), a fallback base username `player_<8-char-uuid>` and fallback name `Player` are used.

---

## 3. Caveats
- **Metadata Fields**: The trigger relies on standard Supabase `raw_user_meta_data` fields (`name` and `username`) if they exist. If not present (e.g., in a default sign-up flow), the logic falls back to email prefixing or auto-generated UUID segments.
- **Rust SDK Discrepancy**: The Rust SDK struct `Player` uses an integer `i64` for `id` and has a `score` field, which diverges from the actual table schema where `id` is a `UUID` and score is tracked inside `game_sessions` and `leaderboard`. Modifying the Rust struct was not requested as part of the read-only migration design, but it should be addressed in subsequent backend updates.

---

## 4. Conclusion

### A. Recommended Filename
Following the Supabase timestamp convention after the latest migration (`20240401000002_create_leaderboard_table.sql`):
- **Sequential Option**: `20240401000003_sync_auth_users_to_players.sql`
- **Timestamp Option (Local)**: `20260615143855_sync_auth_users_to_players.sql`
- **Timestamp Option (UTC)**: `20260615213855_sync_auth_users_to_players.sql`

### B. Drafted SQL Migration Code

```sql
-- 1. Update public.players schema to support email and name columns
ALTER TABLE public.players 
ADD COLUMN IF NOT EXISTS email VARCHAR(255),
ADD COLUMN IF NOT EXISTS name VARCHAR(255);

-- 2. Create the PostgreSQL trigger function with SECURITY DEFINER and secure search path
CREATE OR REPLACE FUNCTION public.handle_new_user()
RETURNS TRIGGER AS $$
DECLARE
    base_username VARCHAR(255);
    temp_username VARCHAR(255);
    proposed_name VARCHAR(255);
    username_exists BOOLEAN;
    suffix_counter INT := 1;
BEGIN
    -- Extract name: use metadata if available, otherwise email prefix, otherwise fallback
    IF NEW.raw_user_meta_data->>'name' IS NOT NULL AND NEW.raw_user_meta_data->>'name' <> '' THEN
        proposed_name := SUBSTR(NEW.raw_user_meta_data->>'name', 1, 255);
    ELSIF NEW.email IS NOT NULL AND NEW.email <> '' THEN
        proposed_name := SUBSTR(split_part(NEW.email, '@', 1), 1, 255);
    ELSE
        proposed_name := 'Player';
    END IF;

    -- Determine base username: use metadata if available, otherwise email prefix, otherwise fallback
    IF NEW.raw_user_meta_data->>'username' IS NOT NULL AND NEW.raw_user_meta_data->>'username' <> '' THEN
        base_username := SUBSTR(NEW.raw_user_meta_data->>'username', 1, 240); -- leave space for suffix
    ELSIF NEW.email IS NOT NULL AND NEW.email <> '' THEN
        base_username := SUBSTR(split_part(NEW.email, '@', 1), 1, 240);
    ELSE
        base_username := 'player_' || SUBSTR(NEW.id::TEXT, 1, 8);
    END IF;

    -- Clean up base username whitespace and check fallback
    base_username := TRIM(base_username);
    IF base_username = '' THEN
        base_username := 'player_' || SUBSTR(NEW.id::TEXT, 1, 8);
    END IF;

    temp_username := base_username;

    -- Check for username uniqueness and handle conflicts gracefully in a loop
    LOOP
        SELECT EXISTS(
            SELECT 1 FROM public.players WHERE username = temp_username
        ) INTO username_exists;
        
        IF NOT username_exists THEN
            EXIT;
        END IF;

        -- If suffix_counter exceeds 100, use a random short string to prevent infinite loops
        IF suffix_counter > 100 THEN
            temp_username := SUBSTR(base_username, 1, 248) || '_' || SUBSTR(md5(random()::TEXT), 1, 6);
            EXIT; -- With md5 of random, it is highly likely to be unique, exit loop
        ELSE
            temp_username := SUBSTR(base_username, 1, 248) || '_' || suffix_counter;
            suffix_counter := suffix_counter + 1;
        END IF;
    END LOOP;

    -- Sync to public.players table using the authenticated user's ID
    INSERT INTO public.players (id, username, name, email)
    VALUES (NEW.id, temp_username, proposed_name, NEW.email);

    RETURN NEW;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER SET search_path = public;

-- 3. Bind the trigger function to the auth.users table
CREATE OR REPLACE TRIGGER on_auth_user_created
  AFTER INSERT ON auth.users
  FOR EACH ROW
  EXECUTE FUNCTION public.handle_new_user();
```

---

## 5. Verification Method
1. **Build Verification**: Run `cargo test` in `/Users/sac/rocket-craft/tools` to ensure there are no compile-time issues.
2. **Schema Integration Check**: Inspect `pwa-staff/src/admin.ts` to confirm columns are named exactly as `name` and `email`.
3. **Database Execution Check**: If a local PostgreSQL instance is running via Supabase, execute the migration and insert a mock user into `auth.users` to confirm automatic synchronization:
   ```sql
   INSERT INTO auth.users (id, email) 
   VALUES ('d3b07384-d113-4956-a5cc-af6b38c23674', 'testplayer@example.com');
   
   -- Check sync
   SELECT * FROM public.players WHERE id = 'd3b07384-d113-4956-a5cc-af6b38c23674';
   ```
4. **Collision Invalidation Condition**: Register a second user with the same email prefix or raw meta data and verify they get a distinct suffixed username in `public.players` instead of raising a UNIQUE constraint exception.
