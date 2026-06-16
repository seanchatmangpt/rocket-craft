# Scope: DB Schema & Trigger

## Architecture
- Database migration to update the schema of the `public.players` table.
- PostgreSQL trigger and trigger function on `auth.users` to automatically populate `public.players` when a new user signs up in Supabase Auth.
- All files are created within the standard Supabase structure: `supabase/migrations/`.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|---|---|---|---|
| 1 | Design Migration | Explore existing migrations and define the SQL statement for schema alter and trigger setup | none | DONE |
| 2 | Write Migration File | Create the SQL migration file in `supabase/migrations/` with a timestamp-prefixed filename | 1 | DONE |
| 3 | Review SQL Logic | Verify syntax, correctness, safety constraints (e.g. UNIQUE conflict resolution, SECURITY DEFINER) | 2 | DONE |
| 4 | Verify Schema & Integrity | Test execution/mock run (if possible) or dry-run, and run Forensic Audit to ensure no bypasses/cheating | 3 | DONE |

## Interface Contracts
### `auth.users` (Supabase Auth) ↔ `public.players` (Public Schema)
- When a row is inserted in `auth.users`, a trigger executes a security definer function.
- The trigger function reads:
  - `new.id` (UUID) -> inserted as `id` in `public.players`
  - `new.email` (VARCHAR) -> inserted as `email` in `public.players`
  - Prefix of `new.email` (using `split_part(new.email, '@', 1)`) -> inserted as `username` and `name` in `public.players`.
- Error Handling & Uniqueness:
  - The trigger function must execute with `SECURITY DEFINER` privileges to bypass RLS policies on `public.players` when inserting from `auth.users` schema (owned by the system).
  - Robust handling for edge cases such as null email or duplicate email prefix (e.g., if there's a unique constraint on username, we can append a random/unique identifier, or default to email prefix and handle conflict dynamically). Wait! Let's let the Explorer research the details of existing players table and how best to handle it.
