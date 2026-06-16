# Supabase Integration

This document serves as the definitive guide to the Supabase integration within the Rocket Craft project. It covers the local development environment, database schema, Edge Functions, Row Level Security (RLS) policies, and the Rust `SupabaseService`.

## Local Docker Setup

For local development and testing, we use a dockerized Supabase instance. This ensures a consistent environment without relying on the cloud platform.

- **API URL:** `http://127.0.0.1:54321`
- **DB URL:** `postgresql://postgres:postgres@127.0.0.1:54322/postgres`
- **Studio UI:** `http://127.0.0.1:54323`
- **GoTrue Auth:** `http://127.0.0.1:9999`
- **Default Anon Key:** `sb_publishable_ACJWlzQHlZjBrEguHvfOxg_3BJgxAaH`

To start the local environment, run:
```bash
supabase start
```
Ensure Docker is running before executing this command.

## Database Schema

Our application relies on three core entities defined in the `public` schema.

### 1. `players`
Stores player profiles, linked to Supabase Auth users.
- `id` (UUID, Primary Key): Matches `auth.users.id`.
- `username` (Text, Unique): Player's display name.
- `created_at` (Timestamp): Record creation time.

### 2. `game_sessions`
Logs individual game matches and their outcomes.
- `id` (UUID, Primary Key): Unique session identifier.
- `player_id` (UUID, Foreign Key -> `players.id`): The player who played the session.
- `score` (Integer): The score achieved in this session.
- `completed_at` (Timestamp): When the session ended.

### 3. `leaderboard`
A view (or materialized table) aggregating the highest scores.
- `player_id` (UUID)
- `username` (Text)
- `high_score` (Integer): The maximum score from `game_sessions` for the player.

## Row Level Security (RLS) Policies

Security is enforced at the database level using RLS to ensure players can only access and modify their own data.

- **`players` table:**
  - `SELECT`: Publicly readable (so leaderboards work).
  - `UPDATE`: Players can only update their own row (`auth.uid() = id`).
- **`game_sessions` table:**
  - `SELECT`: Players can only read their own sessions.
  - `INSERT`: Prevented directly from the client. Inserts are handled exclusively via the `submit-score` Edge Function to prevent cheating.

## Edge Functions

We utilize Supabase Edge Functions for secure, server-side logic that cannot be trusted to the client.

### `submit-score`
This function is responsible for securely validating and recording game scores.
- **Path:** `/functions/v1/submit-score`
- **Authorization:** Requires a valid Bearer token (JWT) of the authenticated player.
- **Logic:**
  1. Verifies the user's JWT.
  2. Validates the incoming score.
  3. Securely inserts a new record into `game_sessions` using a service role key (bypassing RLS for the insert).

## Rust `SupabaseService`

In the backend ecosystem, the `SupabaseService` struct (implemented in Rust) manages programmatic interactions with Supabase.

### Features
- **Authentication:** Handles user login and JWT management.
- **REST Client:** Wraps the PostgREST API for querying the database securely from backend services.
- **Realtime:** Subscribes to database changes (e.g., live leaderboard updates) via Supabase Realtime/WebSockets.

The `SupabaseService` abstracts the HTTP requests and provides a type-safe Rust interface for our database operations.
