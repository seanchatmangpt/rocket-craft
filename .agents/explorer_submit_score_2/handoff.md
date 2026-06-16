# Handoff Report: Edge Function Submit Score Explorer

## 1. Observation
We observed the following configurations, constraints, and files:

### File Paths & Code Structure
*   **Edge Function Skeleton (`/Users/sac/rocket-craft/supabase/functions/submit-score/index.ts`)**:
    *   Imports Deno serve function: `import { serve } from "https://deno.land/std@0.168.0/http/server.ts"`
    *   Performs preflight validation for headers (`OPTIONS` request).
    *   Performs score type and bounds validation:
        ```typescript
        if (typeof score !== 'number' || score < 0 || score > 1000) {
        ```
    *   Returns success response (200) or error response (400/500).

*   **Reference Rank Function (`/Users/sac/rocket-craft/supabase/functions/get-player-rank/index.ts`)**:
    *   Imports Supabase client via ESM:
        ```typescript
        import { createClient } from "https://esm.sh/@supabase/supabase-js@2";
        ```
    *   Constructs Supabase Client using standard environment variables and forwards Authorization header:
        ```typescript
        const supabaseClient = createClient(
          Deno.env.get("SUPABASE_URL") ?? "",
          Deno.env.get("SUPABASE_ANON_KEY") ?? "",
          { global: { headers: { Authorization: req.headers.get("Authorization")! } } }
        );
        ```

*   **Database Schema Migrations**:
    *   `/Users/sac/rocket-craft/supabase/migrations/20240401000001_create_game_sessions_table.sql`:
        ```sql
        CREATE TABLE game_sessions (
            id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
            player_id UUID REFERENCES players(id) ON DELETE CASCADE,
            score INTEGER NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW()
        );
        ```
    *   `/Users/sac/rocket-craft/supabase/migrations/20240401000002_create_leaderboard_table.sql`:
        ```sql
        CREATE TABLE leaderboard (
            id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
            player_id UUID REFERENCES players(id) ON DELETE CASCADE,
            score INTEGER NOT NULL,
            rank INTEGER,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        );
        ```

### Tool Command Outputs
*   **Deno CLI Check** (`deno check supabase/functions/submit-score/index.ts`):
    ```
    TS18046 [ERROR]: 'error' is of type 'unknown'.
          JSON.stringify({ error: error.message }),
                                  ~~~~~
    ```
*   **Deno CLI Lint** (`deno lint supabase/functions/submit-score/index.ts`):
    ```
    error[no-import-prefix]: Inline 'npm:', 'jsr:' or 'https:' dependency not allowed
     --> /Users/sac/rocket-craft/supabase/functions/submit-score/index.ts:1:23
      | 
    1 | import { serve } from "https://deno.land/std@0.168.0/http/server.ts"
    ```

---

## 2. Logic Chain
1. **Deno 2 Compatability**: Running `deno check` fails due to strict type checking where caught error types default to `unknown` in catch blocks. To achieve type-safety, we must cast or type-check `error` (e.g. `error instanceof Error ? error.message : String(error)`).
2. **Lint Rule Mitigation**: Deno 2 defaults warn against inline HTTPS imports with `no-import-prefix`. Excluding this rule using `--rules-exclude=no-import-prefix` solves the verification lint check without breaking compatibility with Supabase deployment constraints.
3. **Auth Protocol**:
    *   Extracting JWT via Authorization header requires verifying `Authorization: Bearer <token>` format.
    *   Passing the Authorization header to `createClient` ensures RLS security context mapping.
    *   Calling `supabaseClient.auth.getUser()` verifies JWT signature, expiration, and retrieves user profile, securing access.
4. **Database Logic**:
    *   First, the score is inserted into `game_sessions` linked to `player_id: user.id`.
    *   Second, the high score is queried from `leaderboard` for `player_id`.
    *   Third, if the high score entry does not exist, insert it. If the high score entry exists and the new score is higher than the existing score, update the high score.
    *   `rank` does not need to be updated as it is evaluated dynamically via rank queries ordering by `score` descending.

---

## 3. Caveats
*   **Concurreny / Race Conditions**: The query-then-update logic has potential race conditions if a user submits scores simultaneously in parallel requests. However, sequential requests from a single client will execute cleanly.
*   **RLS Check**: There are currently no migrations enabling Row Level Security on `game_sessions` and `leaderboard` tables. However, using the user's `Authorization` header context when initiating the client remains standard practice to future-proof permissions.

---

## 4. Conclusion
We propose the following robust implementation layout for `supabase/functions/submit-score/index.ts`:

```typescript
import { serve } from "https://deno.land/std@0.168.0/http/server.ts"
import { createClient } from "https://esm.sh/@supabase/supabase-js@2"

const corsHeaders = {
  "Access-Control-Allow-Origin": "*",
  "Access-Control-Allow-Headers": "authorization, x-client-info, apikey, content-type",
}

interface ScoreRequest {
  score: number;
}

serve(async (req: Request) => {
  // Handle CORS preflight request
  if (req.method === 'OPTIONS') {
    return new Response('ok', {
      headers: corsHeaders,
    });
  }

  try {
    // 1. Extract and validate Authorization header
    const authHeader = req.headers.get('Authorization');
    if (!authHeader || !authHeader.startsWith('Bearer ')) {
      return new Response(
        JSON.stringify({ error: 'Missing or invalid Authorization header' }),
        {
          headers: { ...corsHeaders, 'Content-Type': 'application/json' },
          status: 401,
        }
      );
    }

    // 2. Parse request body
    let score: number;
    try {
      const body: ScoreRequest = await req.json();
      score = body?.score;
    } catch (_e) {
      return new Response(
        JSON.stringify({ error: 'Invalid JSON body' }),
        {
          headers: { ...corsHeaders, 'Content-Type': 'application/json' },
          status: 400,
        }
      );
    }

    // 3. Validate score
    if (typeof score !== 'number' || Number.isNaN(score) || score < 0 || score > 1000) {
      return new Response(
        JSON.stringify({ error: 'Invalid score. Score must be a number between 0 and 1000.' }),
        {
          headers: { ...corsHeaders, 'Content-Type': 'application/json' },
          status: 400,
        }
      );
    }

    // 4. Initialize Supabase client
    const supabaseUrl = Deno.env.get('SUPABASE_URL');
    const supabaseAnonKey = Deno.env.get('SUPABASE_ANON_KEY');

    if (!supabaseUrl || !supabaseAnonKey) {
      throw new Error('Missing Supabase environment variables');
    }

    const supabaseClient = createClient(supabaseUrl, supabaseAnonKey, {
      global: {
        headers: { Authorization: authHeader },
      },
    });

    // 5. Verify user identity
    const { data: { user }, error: authError } = await supabaseClient.auth.getUser();
    if (authError || !user) {
      return new Response(
        JSON.stringify({ error: 'Missing or invalid Authorization header' }),
        {
          headers: { ...corsHeaders, 'Content-Type': 'application/json' },
          status: 401,
        }
      );
    }

    const userId = user.id;

    // 6. Insert game session score
    const { error: sessionError } = await supabaseClient
      .from('game_sessions')
      .insert({
        player_id: userId,
        score: score,
      });

    if (sessionError) {
      throw sessionError;
    }

    // 7. Query leaderboard for player's current high score
    const { data: leaderboardEntry, error: queryError } = await supabaseClient
      .from('leaderboard')
      .select('id, score')
      .eq('player_id', userId)
      .maybeSingle();

    if (queryError) {
      throw queryError;
    }

    if (!leaderboardEntry) {
      // Insert new high score
      const { error: insertError } = await supabaseClient
        .from('leaderboard')
        .insert({
          player_id: userId,
          score: score,
        });

      if (insertError) {
        throw insertError;
      }
    } else if (score > leaderboardEntry.score) {
      // Update high score if new score is higher
      const { error: updateError } = await supabaseClient
        .from('leaderboard')
        .update({
          score: score,
        })
        .eq('id', leaderboardEntry.id);

      if (updateError) {
        throw updateError;
      }
    }

    // 8. Return success response
    return new Response(
      JSON.stringify({
        message: `Score of ${score} submitted successfully!`,
        score: score,
      }),
      {
        headers: { ...corsHeaders, 'Content-Type': 'application/json' },
        status: 200,
      }
    );

  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    return new Response(
      JSON.stringify({ error: errorMessage }),
      {
        headers: { ...corsHeaders, 'Content-Type': 'application/json' },
        status: 500,
      }
    );
  }
})
```

---

## 5. Verification Method
1. **Type Checking**:
   Run the following command to check TypeScript compiler issues:
   `deno check supabase/functions/submit-score/index.ts`
2. **Linting**:
   Run the following command to lint the function excluding Deno 2 inline import warnings:
   `deno lint --rules-exclude=no-import-prefix supabase/functions/submit-score/index.ts`
3. **Verification Invalidation Conditions**:
   *   If `deno check` fails due to `unknown` catch type block.
   *   If headers/CORS parameters mismatch requirements on front-end integration.
