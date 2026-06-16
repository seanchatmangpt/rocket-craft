## 2026-06-15T21:40:17Z

Create a new migration file: `/Users/sac/rocket-craft/supabase/migrations/20240401000003_sync_auth_users_to_players.sql` with the following content:

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

Please execute the implementation of this migration file. Perform any necessary build checks or tests to confirm that your change compiles/works correctly (e.g. run local database tests/checks or SDK builds if there is any dependent code). Write a handoff report documenting the file created, commands run, and results to `/Users/sac/rocket-craft/.agents/worker_db_schema_1/handoff.md`.
