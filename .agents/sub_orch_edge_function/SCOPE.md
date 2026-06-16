# Scope: Milestone 4: Edge Function Submit Score

## Architecture
- Deno Edge Function located at `supabase/functions/submit-score/index.ts`.
- Database interaction via `@supabase/supabase-js` client.
- Authentic authorization verification via Supabase Auth API using Authorization Bearer JWT.
- Score validation: check that the score is a valid number, between 0 and 1000 inclusive.
- Insert record into `public.game_sessions`.
- Conditionally insert or update record in `public.leaderboard` if new score > current high score.

## Interface Contracts
### `submit-score` Edge Function API
- **Endpoint**: `/functions/v1/submit-score`
- **Method**: `POST`
- **Headers**:
  - `Content-Type: application/json`
  - `Authorization: Bearer <JWT>`
- **Request Body**:
  ```json
  {
    "score": number
  }
  ```
- **Response**:
  - Success (200):
    ```json
    {
      "message": "Score of <score> submitted successfully!",
      "score": <score>
    }
    ```
  - Invalid Score (400):
    ```json
    {
      "error": "Invalid score. Score must be a number between 0 and 1000."
    }
    ```
  - Unauthorized (401):
    ```json
    {
      "error": "Missing or invalid Authorization header"
    }
    ```
  - Internal Error (500):
    ```json
    {
      "error": "<error message>"
    }
    ```

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | Submit Score Implementation | Complete edge function auth verification, validation, database insert, and high score upsert logic | None | PLANNED |

## Code Layout
- `supabase/functions/submit-score/index.ts`: Implementation file.
