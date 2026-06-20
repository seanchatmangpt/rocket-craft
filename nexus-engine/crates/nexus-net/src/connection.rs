//! Typestate connection — each legal state transition is encoded in the type system.
//!
//! Illegal transitions (e.g. sending a `CombatAction` while `Disconnected`) are a
//! compile-time error because the required `impl Connection<InMatch>` block simply
//! doesn't exist for the wrong state.
//!
//! `InMatch` carries the current `match_id` as a field on the `Connection` struct
//! (as `Option<u64>`) rather than as a const-generic or zero-sized struct with data,
//! which is the pragmatic approach for stable Rust.

use std::marker::PhantomData;

// ── State-marker types ────────────────────────────────────────────────────────

/// The TCP / WebSocket channel has not been opened yet.
pub struct Disconnected;

/// The WebSocket upgrade / handshake is in progress.
pub struct Handshaking;

/// Channel is open; the player has not yet sent credentials.
pub struct Connected;

/// The player has been authenticated.  They may now join the lobby or disconnect.
pub struct Authenticated;

/// The player is in the global lobby, visible to challengers.
pub struct InLobby;

/// The player is participating in an active duel match.
pub struct InMatch;

// ── Connection ────────────────────────────────────────────────────────────────

/// A connection to one remote player client.
///
/// The type parameter `S` is a zero-sized marker that encodes the current
/// protocol state.  Fields are intentionally `pub` so higher-level server code
/// can read them without boilerplate accessors.
///
/// # Examples
///
/// ```
/// use nexus_net::connection::{Connection, Disconnected, Connected};
///
/// // Create a default disconnected connection
/// let conn = Connection::new();
/// assert!(conn.player_id.is_none());
///
/// // Disconnected -> Handshaking -> Connected transition sequence
/// let handshaking = conn.begin_handshake();
/// let connected = handshaking.complete();
///
/// // Connected -> Authenticated transition
/// let authenticated = connected.authenticate(101, 9999);
/// assert_eq!(authenticated.player_id, Some(101));
/// assert_eq!(authenticated.session_id, Some(9999));
/// ```
pub struct Connection<S> {
    /// Populated after successful `Authenticate` flow.
    pub player_id: Option<u64>,
    /// Populated after `AuthSuccess`.
    pub session_id: Option<u64>,
    /// Round-trip latency in milliseconds (updated from `Ping`/`Pong`).
    pub latency_ms: u32,
    /// Total messages sent on this connection.
    pub messages_sent: u64,
    /// Total messages received on this connection.
    pub messages_received: u64,
    /// Active match id when in the `InMatch` state; `None` otherwise.
    pub match_id: Option<u64>,
    _state: PhantomData<S>,
}

/// A builder for [`Connection`] in the [`Disconnected`] state.
///
/// # Examples
///
/// ```
/// use nexus_net::connection::{Connection, ConnectionBuilder, Disconnected};
///
/// let conn = ConnectionBuilder::new()
///     .player_id(7)
///     .session_id(777)
///     .latency_ms(15)
///     .build();
///
/// assert_eq!(conn.player_id, Some(7));
/// assert_eq!(conn.session_id, Some(777));
/// assert_eq!(conn.latency_ms, 15);
/// ```
#[derive(Debug, Clone)]
pub struct ConnectionBuilder {
    player_id: Option<u64>,
    session_id: Option<u64>,
    latency_ms: u32,
    messages_sent: u64,
    messages_received: u64,
    match_id: Option<u64>,
}

impl ConnectionBuilder {
    /// Create a new builder with default parameters.
    pub fn new() -> Self {
        Self {
            player_id: None,
            session_id: None,
            latency_ms: 0,
            messages_sent: 0,
            messages_received: 0,
            match_id: None,
        }
    }

    /// Set the player ID.
    pub fn player_id(mut self, player_id: u64) -> Self {
        self.player_id = Some(player_id);
        self
    }

    /// Set the session ID.
    pub fn session_id(mut self, session_id: u64) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Set the initial latency estimate.
    pub fn latency_ms(mut self, latency_ms: u32) -> Self {
        self.latency_ms = latency_ms;
        self
    }

    /// Set the messages sent count.
    pub fn messages_sent(mut self, count: u64) -> Self {
        self.messages_sent = count;
        self
    }

    /// Set the messages received count.
    pub fn messages_received(mut self, count: u64) -> Self {
        self.messages_received = count;
        self
    }

    /// Set the current match ID.
    pub fn match_id(mut self, match_id: u64) -> Self {
        self.match_id = Some(match_id);
        self
    }

    /// Build the connection in [`Disconnected`] state.
    pub fn build(self) -> Connection<Disconnected> {
        Connection {
            player_id: self.player_id,
            session_id: self.session_id,
            latency_ms: self.latency_ms,
            messages_sent: self.messages_sent,
            messages_received: self.messages_received,
            match_id: self.match_id,
            _state: PhantomData,
        }
    }
}

impl Default for ConnectionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents the dynamic runtime state of a WebSocket connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ConnectionState {
    Disconnected,
    Handshaking,
    Connected,
    Authenticated,
    InLobby,
    InMatch,
}

/// Errors returned when a connection transition is invalid at runtime.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("Illegal connection state transition: cannot transition from {current:?} to {target:?}. Reason: {reason}")]
pub struct ConnectionTransitionError {
    pub current: ConnectionState,
    pub target: ConnectionState,
    pub reason: String,
}

// ── Internal helper ───────────────────────────────────────────────────────────

/// Plain-data bundle — all fields except the state marker.
struct ConnectionBase {
    player_id: Option<u64>,
    session_id: Option<u64>,
    latency_ms: u32,
    messages_sent: u64,
    messages_received: u64,
    match_id: Option<u64>,
}

impl<S> Connection<S> {
    fn base(&self) -> ConnectionBase {
        ConnectionBase {
            player_id: self.player_id,
            session_id: self.session_id,
            latency_ms: self.latency_ms,
            messages_sent: self.messages_sent,
            messages_received: self.messages_received,
            match_id: self.match_id,
        }
    }

    fn from_base<T>(b: ConnectionBase) -> Connection<T> {
        Connection {
            player_id: b.player_id,
            session_id: b.session_id,
            latency_ms: b.latency_ms,
            messages_sent: b.messages_sent,
            messages_received: b.messages_received,
            match_id: b.match_id,
            _state: PhantomData,
        }
    }
}

// ── State transitions ─────────────────────────────────────────────────────────

impl Connection<Disconnected> {
    /// Create a fresh, un-connected connection record.
    pub fn new() -> Self {
        Connection {
            player_id: None,
            session_id: None,
            latency_ms: 0,
            messages_sent: 0,
            messages_received: 0,
            match_id: None,
            _state: PhantomData,
        }
    }

    /// Start the WebSocket upgrade handshake.
    pub fn begin_handshake(self) -> Connection<Handshaking> {
        Self::from_base(self.base())
    }
}

impl Default for Connection<Disconnected> {
    fn default() -> Self {
        Self::new()
    }
}

impl Connection<Handshaking> {
    /// Handshake succeeded — upgrade to a fully-open channel.
    pub fn complete(self) -> Connection<Connected> {
        Self::from_base(self.base())
    }

    /// Handshake failed — move back to disconnected.
    pub fn fail(self) -> Connection<Disconnected> {
        Self::from_base(self.base())
    }
}

impl Connection<Connected> {
    /// Credentials accepted by the server.
    pub fn authenticate(self, player_id: u64, session_id: u64) -> Connection<Authenticated> {
        let mut b = self.base();
        b.player_id = Some(player_id);
        b.session_id = Some(session_id);
        Self::from_base(b)
    }

    /// Remote disconnected before authenticating.
    pub fn disconnect(self) -> Connection<Disconnected> {
        Self::from_base(self.base())
    }
}

impl Connection<Authenticated> {
    /// Player enters the public matchmaking lobby.
    pub fn join_lobby(self) -> Connection<InLobby> {
        Self::from_base(self.base())
    }

    /// Player disconnects cleanly.
    pub fn disconnect(self) -> Connection<Disconnected> {
        Self::from_base(self.base())
    }
}

impl Connection<InLobby> {
    /// A duel match was found or accepted; record the match id.
    pub fn enter_match(self, match_id: u64) -> Connection<InMatch> {
        let mut b = self.base();
        b.match_id = Some(match_id);
        Self::from_base(b)
    }

    /// Player leaves the lobby voluntarily.
    pub fn leave_lobby(self) -> Connection<Authenticated> {
        Self::from_base(self.base())
    }

    /// Player disconnects from the lobby.
    pub fn disconnect(self) -> Connection<Disconnected> {
        Self::from_base(self.base())
    }
}

impl Connection<InMatch> {
    /// Retrieve the current match id (always `Some` in this state).
    pub fn current_match_id(&self) -> u64 {
        self.match_id
            .expect("InMatch connection must have a match_id")
    }

    /// Match ended; return to the lobby so the player can queue again.
    pub fn finish_match(self) -> Connection<InLobby> {
        let mut b = self.base();
        b.match_id = None;
        Self::from_base(b)
    }

    /// Player disconnects mid-match.
    pub fn disconnect(self) -> Connection<Disconnected> {
        let mut b = self.base();
        b.match_id = None;
        Self::from_base(b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn disconnected() -> Connection<Disconnected> {
        ConnectionBuilder::new().player_id(1).session_id(100).build()
    }

    // ── builder ───────────────────────────────────────────────────────────────

    #[test]
    fn builder_sets_player_and_session_id() {
        let c = ConnectionBuilder::new().player_id(7).session_id(77).build();
        assert_eq!(c.player_id, Some(7));
        assert_eq!(c.session_id, Some(77));
    }

    #[test]
    fn builder_default_latency_is_zero() {
        let c = ConnectionBuilder::new().build();
        assert_eq!(c.latency_ms, 0);
    }

    #[test]
    fn builder_latency_ms_is_set() {
        let c = ConnectionBuilder::new().latency_ms(42).build();
        assert_eq!(c.latency_ms, 42);
    }

    // ── Disconnected → Handshaking ────────────────────────────────────────────

    #[test]
    fn begin_handshake_transitions_to_handshaking() {
        let c = disconnected();
        let h = c.begin_handshake();
        // Handshaking state has a `complete()` method — presence confirms typestate
        let _ = h.complete();
    }

    // ── Handshaking → Connected → Authenticated ───────────────────────────────

    #[test]
    fn handshaking_complete_then_authenticate() {
        let c = disconnected();
        let authenticated = c.begin_handshake().complete().authenticate(99, 999);
        assert_eq!(authenticated.player_id, Some(99));
        assert_eq!(authenticated.session_id, Some(999));
    }

    // ── Handshaking → Disconnected (fail) ─────────────────────────────────────

    #[test]
    fn handshaking_fail_returns_to_disconnected() {
        let c = disconnected();
        let back = c.begin_handshake().fail();
        // Can start handshake again from Disconnected
        let _ = back.begin_handshake();
    }

    // ── Authenticated → InLobby ───────────────────────────────────────────────

    #[test]
    fn authenticated_can_join_lobby() {
        let c = disconnected()
            .begin_handshake().complete()
            .authenticate(1, 100)
            .join_lobby();
        // InLobby has enter_match — presence confirms typestate
        let _ = c.enter_match(42);
    }

    // ── InLobby → InMatch ─────────────────────────────────────────────────────

    #[test]
    fn in_lobby_enters_match_sets_match_id() {
        let c = disconnected()
            .begin_handshake().complete()
            .authenticate(1, 100)
            .join_lobby()
            .enter_match(777);
        assert_eq!(c.current_match_id(), 777);
    }

    // ── InLobby → Authenticated (leave lobby) ────────────────────────────────

    #[test]
    fn in_lobby_can_leave_back_to_authenticated() {
        let auth = disconnected()
            .begin_handshake().complete()
            .authenticate(1, 100)
            .join_lobby()
            .leave_lobby();
        // Back to Authenticated: can join lobby again
        let _ = auth.join_lobby();
    }

    // ── Disconnect from any state ─────────────────────────────────────────────

    #[test]
    fn authenticated_disconnect_returns_to_disconnected() {
        let back = disconnected()
            .begin_handshake().complete()
            .authenticate(1, 100)
            .disconnect();
        let _ = back.begin_handshake(); // Disconnected state confirmed
    }

    #[test]
    fn in_lobby_disconnect_returns_to_disconnected() {
        let back = disconnected()
            .begin_handshake().complete()
            .authenticate(1, 100)
            .join_lobby()
            .disconnect();
        let _ = back.begin_handshake();
    }
}
