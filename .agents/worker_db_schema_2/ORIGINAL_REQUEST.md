## 2026-06-15T21:47:18Z

You are a Worker. Your task is to update the Supabase database migration file at `/Users/sac/rocket-craft/supabase/migrations/20240401000003_sync_auth_users_to_players.sql` to implement the following improvements:

1. Add a foreign key constraint from `public.players(id)` to `auth.users(id)` with cascade on delete.
2. In `public.handle_new_user()`:
   - Set the security definer search path safely: `SET search_path = pg_catalog, public`.
   - Trim the extracted name and handle whitespace-only input by falling back to `'Player'`.
   - Resolve the concurrency race condition by trying to perform the insert inside a `BEGIN ... EXCEPTION WHEN unique_violation THEN` block within the loop, handling conflicts atomically.

Here is the recommended SQL structure for the updated migration file:

```sql
-- 1. Update public.players schema to support email and name columns, and add foreign key reference
ALTER TABLE public.players 
ADD COLUMN IF NOT EXISTS email VARCHAR(255),
ADD COLUMN IF NOT EXISTS name VARCHAR(255);

ALTER TABLE public.players
ADD CONSTRAINT fk_players_auth_user FOREIGN KEY (id) REFERENCES auth.users(id) ON DELETE CASCADE;

-- 2. Create the PostgreSQL trigger function with SECURITY DEFINER and secure search path
CREATE OR REPLACE FUNCTION public.handle_new_user()
RETURNS TRIGGER AS $$
DECLARE
    base_username VARCHAR(255);
    temp_username VARCHAR(255);
    proposed_name VARCHAR(255);
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

    -- Trim proposed name and check if it becomes empty
    proposed_name := TRIM(proposed_name);
    IF proposed_name = '' THEN
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

    -- Loop to handle username conflicts atomically using exceptions
    LOOP
        BEGIN
            INSERT INTO public.players (id, username, name, email)
            VALUES (NEW.id, temp_username, proposed_name, NEW.email);
            EXIT; -- Insert succeeded, exit loop
        EXCEPTION WHEN unique_violation THEN
            -- Handle username collision
            IF suffix_counter > 100 THEN
                temp_username := SUBSTR(base_username, 1, 248) || '_' || SUBSTR(md5(random()::TEXT), 1, 6);
                -- Try final insert, if it fails, let the error propagate
                INSERT INTO public.players (id, username, name, email)
                VALUES (NEW.id, temp_username, proposed_name, NEW.email);
                EXIT;
            ELSE
                temp_username := SUBSTR(base_username, 1, 248) || '_' || suffix_counter;
                suffix_counter := suffix_counter + 1;
            END IF;
        END;
    END LOOP;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER SET search_path = pg_catalog, public;

-- 3. Bind the trigger function to the auth.users table
CREATE OR REPLACE TRIGGER on_auth_user_created
  AFTER INSERT ON auth.users
  FOR EACH ROW
  EXECUTE FUNCTION public.handle_new_user();
```

Please update the migration file, run validation checks (`supabase db reset`, `supabase db lint`, `./rocket test`), and verify all edge cases (including the whitespace-only metadata case) using `psql`. Document your implementation and tests in `/Users/sac/rocket-craft/.agents/worker_db_schema_2/handoff.md`.
