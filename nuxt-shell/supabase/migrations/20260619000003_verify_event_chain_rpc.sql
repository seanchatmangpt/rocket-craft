-- Migration: verify_event_chain RPC
-- Van der Aalst doctrine: if the event log cannot prove a lawful process happened,
-- then it did not work.
--
-- This function walks ocel_events ordered by (session_id, seq) and verifies:
--   (a) The genesis event (seq = min) has prev_hash IS NULL.
--   (b) Every subsequent event's prev_hash matches the previous row's event_hash.
--
-- Call with no argument to check every session, or pass a UUID to scope to one.

CREATE OR REPLACE FUNCTION verify_event_chain(
  p_session_id UUID DEFAULT NULL
)
RETURNS TABLE (
  ok          BOOLEAN,
  message     TEXT,
  broken_at   BIGINT,   -- ocel_events.id of the first offending row, NULL when ok
  session_id  UUID
)
LANGUAGE plpgsql
STABLE
AS $$
DECLARE
  r_prev        RECORD;
  r_cur         RECORD;
  v_first       BOOLEAN;
  v_session     UUID;
  v_broke       BOOLEAN;
  v_event_count INTEGER;
BEGIN
  FOR v_session IN
    SELECT DISTINCT e.session_id
    FROM   ocel_events e
    WHERE  p_session_id IS NULL OR e.session_id = p_session_id
    ORDER  BY 1
  LOOP
    v_first       := TRUE;
    v_broke       := FALSE;
    v_event_count := 0;
    r_prev        := NULL;

    FOR r_cur IN
      SELECT e.id, e.seq, e.prev_hash, e.event_hash
      FROM   ocel_events e
      WHERE  e.session_id = v_session
      ORDER  BY e.seq ASC
    LOOP
      v_event_count := v_event_count + 1;

      IF v_first THEN
        -- Genesis event: prev_hash must be NULL
        IF r_cur.prev_hash IS NOT NULL THEN
          ok        := FALSE;
          message   := format(
            'Genesis event (seq=%s, id=%s) has non-NULL prev_hash: %s',
            r_cur.seq, r_cur.id, r_cur.prev_hash
          );
          broken_at  := r_cur.id;
          "session_id" := v_session;
          RETURN NEXT;
          v_broke := TRUE;
          EXIT;  -- skip remaining events for this session
        END IF;
        v_first := FALSE;

      ELSE
        -- Subsequent event: prev_hash must equal the prior row's event_hash
        IF r_cur.prev_hash IS DISTINCT FROM r_prev.event_hash THEN
          ok        := FALSE;
          message   := format(
            'Chain break at seq=%s (id=%s): expected prev_hash=%s, got %s',
            r_cur.seq, r_cur.id,
            r_prev.event_hash,
            coalesce(r_cur.prev_hash, '<NULL>')
          );
          broken_at  := r_cur.id;
          "session_id" := v_session;
          RETURN NEXT;
          v_broke := TRUE;
          EXIT;
        END IF;
      END IF;

      r_prev := r_cur;
    END LOOP;

    -- Emit PASS row when no break was detected
    IF NOT v_broke THEN
      ok         := TRUE;
      broken_at  := NULL;
      "session_id" := v_session;
      IF v_event_count = 0 THEN
        message := 'No events';
      ELSE
        message := format('Chain intact (%s events)', v_event_count);
      END IF;
      RETURN NEXT;
    END IF;

  END LOOP;
END;
$$;

COMMENT ON FUNCTION verify_event_chain(UUID) IS
  'Verify hash-chain integrity of ocel_events per session. '
  'Pass NULL to check all sessions. '
  'Returns one row per session: ok=TRUE means chain intact, '
  'ok=FALSE includes the offending event id in broken_at.';
