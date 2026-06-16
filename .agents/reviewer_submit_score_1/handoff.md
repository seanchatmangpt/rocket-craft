# Handoff Report - submit-score Edge Function Review

## Quality & Adversarial Review Report

**Verdict**: REQUEST_CHANGES

---

## 1. Observation

### Code Inspection
- **File Path**: `/Users/sac/rocket-craft/supabase/functions/submit-score/index.ts`
- **Authentication Header Retrieval (Lines 34-44)**:
  ```typescript
  // Extract authorization JWT token from headers case-insensitively
  const authHeader = req.headers.get("Authorization") || req.headers.get("authorization");
  if (!authHeader) {
    return new Response(
      JSON.stringify({ error: "Missing Authorization header" }),
      {
        headers: { ...corsHeaders, "Content-Type": "application/json" },
        status: 401,
      }
    );
  }
  ```
- **Authentication User Validation (Lines 100-110)**:
  ```typescript
  // Verify the user's identity and obtain their UUID via supabaseClient.auth.getUser()
  const { data: { user }, error: authError } = await supabaseClient.auth.getUser();
  if (authError || !user) {
    return new Response(
      JSON.stringify({ error: authError?.message || "Unauthorized" }),
      {
        headers: { ...corsHeaders, "Content-Type": "application/json" },
        status: 401,
      }
    );
  }
  ```
- **Score Validation Logic (Lines 62-77)**:
  ```typescript
  // Validate that the score is a valid, non-NaN integer between 0 and 1000 inclusive
  if (
    typeof score !== "number" ||
    Number.isNaN(score) ||
    !Number.isInteger(score) ||
    score < 0 ||
    score > 1000
  ) {
    return new Response(
      JSON.stringify({ error: "Invalid score. Score must be a valid integer between 0 and 1000 inclusive." }),
      {
        headers: { ...corsHeaders, "Content-Type": "application/json" },
        status: 400,
      }
    );
  }
  ```
- **Database Insertion & Update Logic (Lines 114-158)**:
  ```typescript
  // Save the score to the public.game_sessions table
  const { error: sessionError } = await supabaseClient
    .from("game_sessions")
    .insert({
      player_id: playerId,
      score: score,
    });
  // ...
  // Query the public.leaderboard table for the player's current high score
  const { data: leaderboardData, error: leaderboardSelectError } = await supabaseClient
    .from("leaderboard")
    .select("id, score")
    .eq("player_id", playerId)
    .maybeSingle();
  // ...
  // If no high score exists or the new score is higher than the existing maximum high score, update/insert the player's entry
  if (!leaderboardData) {
    const { error: insertError } = await supabaseClient
      .from("leaderboard")
      .insert({
        player_id: playerId,
        score: score,
      });
    // ...
  } else if (score > leaderboardData.score) {
    const { error: updateError } = await supabaseClient
      .from("leaderboard")
      .update({
        score: score,
      })
      .eq("id", leaderboardData.id);
    // ...
  }
  ```

### Tool Outputs
- **Compile & Lint Verification**:
  - Command: `deno check supabase/functions/submit-score/index.ts && deno lint --rules-exclude=no-import-prefix supabase/functions/submit-score/index.ts`
  - Output: `Check file:///Users/sac/rocket-craft/supabase/functions/submit-score/index.ts` (Success, no warnings or lint errors).
- **Unit Test Execution**:
  - Command: `deno test --node-modules-dir=none --allow-env supabase/functions/submit-score/index.test.ts`
  - Output:
    ```
    running 5 tests from ./supabase/functions/submit-score/index.test.ts
    submit-score edge function: OPTIONS method CORS headers ... ok (1ms)
    submit-score edge function: GET method not allowed ... ok (0ms)
    submit-score edge function: POST method missing Authorization header ... ok (0ms)
    submit-score edge function: POST method invalid JSON body ... ok (0ms)
    submit-score edge function: POST method score validation rules ... ok (0ms)

    ok | 5 passed | 0 failed (4ms)
    ```

---

## 2. Logic Chain

### Authentication Verification
- **Observation**: Headers are retrieved case-insensitively and verified via Supabase Auth `getUser()`.
- **Reasoning**: This correctly delegates JWT parsing, signature validation, and expiration checks to the Supabase Auth server, avoiding fragile and insecure custom parsing. 
- **Verdict**: Pass.

### Score Validation
- **Observation**: The score check verifies type, NaN, non-integer, negative, and excessive values.
- **Reasoning**:
  - `typeof score !== "number"` blocks non-number types (e.g. strings, objects).
  - `Number.isNaN(score)` blocks `NaN` inputs.
  - `!Number.isInteger(score)` blocks decimals/floats.
  - `score < 0 || score > 1000` enforces lower/upper bounds.
- **Verdict**: Pass.

### Database Insertion & Safety
- **Observation**: Standard Supabase parameterized query/mutation methods are used. No raw string interpolation is present in SQL statements.
- **Reasoning**: Parameterized queries built by the client prevent SQL injection vulnerabilities.
- **Verdict**: Pass (regarding SQL safety).

### Concurrency Risk (Adversarial Challenge)
- **Observation**: Updating the `leaderboard` table uses a read-then-write approach (lines 127-158).
- **Reasoning**:
  1. **Lost Update Anomaly**: If a player submits score `150` (Request A) and score `200` (Request B) concurrently:
     - Both requests query the leaderboard and fetch the current score (say `100`).
     - Request B updates the high score to `200`.
     - Request A updates the high score to `150` because `150 > 100`. The high score is incorrectly downgraded to `150`.
  2. **Duplicate Insertion Anomaly**: Under concurrent inserts when no row exists, both requests may evaluate `!leaderboardData` as `true` and perform an `insert()`. Because there is no `UNIQUE` constraint on `leaderboard.player_id` (verified in `20240401000002_create_leaderboard_table.sql`), this will result in duplicate rows for the same player, breaking the `maybeSingle()` query on subsequent calls.
- **Verdict**: Fail (requires mitigation via atomic upsert or unique constraints).

### Unit Test Completeness
- **Observation**: The unit tests in `index.test.ts` only cover failing conditions and preflight check. There is no test verifying the success path.
- **Reasoning**: The absence of success path verification leaves database mutations and client initialization untested. A mock client/stub should be introduced to verify the successful submission path.
- **Verdict**: Fail (requires mock implementation in tests).

---

## 3. Caveats

- Local testing of database mutations was simulated via logical code path analysis, as setting up a mock local PostgreSQL/Supabase instance is outside the unit tests' execution context without modifying implementation files.
- Did not verify OAuth2 token validity since that depends on the external Supabase auth provider's runtime behaviour.

---

## 4. Conclusion

The `submit-score` edge function is cleanly written, safe from raw SQL injections, and successfully passes compilation and lint rules. However, it suffers from two major/critical gaps:
1. **Concurrency Race Conditions**: The read-then-write high-score update logic is vulnerable to lost updates and duplicate row inserts.
2. **Incomplete Test Coverage**: The unit tests lack success-path scenarios due to missing client mocking/stubbing.

We recommend modifying:
- The `leaderboard` table to add a `UNIQUE` constraint on `player_id`.
- The update logic to perform a single PostgreSQL atomic upsert (or an RPC call) rather than read-then-write.
- The unit tests to stub the Supabase Client so that success paths (status 200) can be verified under tests.

---

## 5. Verification Method

- **Compile Check**:
  `deno check supabase/functions/submit-score/index.ts`
- **Lint Check**:
  `deno lint --rules-exclude=no-import-prefix supabase/functions/submit-score/index.ts`
- **Test Command**:
  `deno test --node-modules-dir=none --allow-env supabase/functions/submit-score/index.test.ts`
- **Manual Code Inspection**:
  Verify lines 127-158 in `/Users/sac/rocket-craft/supabase/functions/submit-score/index.ts` to inspect the read-then-write logic and compare it with the leaderboard migration `20240401000002_create_leaderboard_table.sql` for the missing unique constraint.
