use std::marker::PhantomData;

// Session state markers — zero-sized types used only as type-level tags.
pub struct Connecting;
pub struct Authenticated;
pub struct InLobby;
pub struct InMatch;
pub struct Spectating;
pub struct Disconnected;

/// A player session typed by its current state `S`.
///
/// Callers can only call the transition methods available on the concrete
/// instantiation they hold, so illegal state transitions are rejected at
/// compile time.
pub struct PlayerSession<S> {
    pub player_id: u64,
    pub username: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    // Private — forces use of transition methods rather than direct construction.
    _state: PhantomData<S>,
}

// ────────────────────────────────────────────────────────────────────────────
// Connecting
// ────────────────────────────────────────────────────────────────────────────

impl PlayerSession<Connecting> {
    /// Create a brand-new session that has not yet been authenticated.
    pub fn new(player_id: u64, username: String) -> Self {
        PlayerSession {
            player_id,
            username,
            created_at: chrono::Utc::now(),
            _state: PhantomData,
        }
    }

    /// Validate the bearer token.  Returns an `Authenticated` session on
    /// success or `SessionError::AuthFailed` when the token is invalid.
    pub fn authenticate(
        self,
        token_valid: bool,
    ) -> Result<PlayerSession<Authenticated>, SessionError> {
        if token_valid {
            Ok(PlayerSession {
                player_id: self.player_id,
                username: self.username,
                created_at: self.created_at,
                _state: PhantomData,
            })
        } else {
            Err(SessionError::AuthFailed)
        }
    }

    /// Explicitly reject (ban / bad handshake) — moves directly to
    /// `Disconnected` without ever reaching `Authenticated`.
    pub fn reject(self) -> PlayerSession<Disconnected> {
        PlayerSession {
            player_id: self.player_id,
            username: self.username,
            created_at: self.created_at,
            _state: PhantomData,
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Authenticated
// ────────────────────────────────────────────────────────────────────────────

impl PlayerSession<Authenticated> {
    /// Move into the lobby after a successful login.
    pub fn enter_lobby(self) -> PlayerSession<InLobby> {
        PlayerSession {
            player_id: self.player_id,
            username: self.username,
            created_at: self.created_at,
            _state: PhantomData,
        }
    }

    /// Clean disconnect before the player ever enters the lobby.
    pub fn disconnect(self) -> PlayerSession<Disconnected> {
        PlayerSession {
            player_id: self.player_id,
            username: self.username,
            created_at: self.created_at,
            _state: PhantomData,
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// InLobby
// ────────────────────────────────────────────────────────────────────────────

impl PlayerSession<InLobby> {
    /// Join an active match.  Returns the session-in-match together with the
    /// match identifier so the caller can wire up the session to the right
    /// game instance.
    pub fn enter_match(self, match_id: u64) -> (PlayerSession<InMatch>, u64) {
        let session = PlayerSession {
            player_id: self.player_id,
            username: self.username,
            created_at: self.created_at,
            _state: PhantomData,
        };
        (session, match_id)
    }

    /// Watch an ongoing match without participating.
    pub fn spectate(self, match_id: u64) -> (PlayerSession<Spectating>, u64) {
        let session = PlayerSession {
            player_id: self.player_id,
            username: self.username,
            created_at: self.created_at,
            _state: PhantomData,
        };
        (session, match_id)
    }

    /// Disconnect from the lobby.
    pub fn disconnect(self) -> PlayerSession<Disconnected> {
        PlayerSession {
            player_id: self.player_id,
            username: self.username,
            created_at: self.created_at,
            _state: PhantomData,
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// InMatch
// ────────────────────────────────────────────────────────────────────────────

impl PlayerSession<InMatch> {
    /// The match has finished (verified, defeat, or time-out).  The player
    /// returns to the lobby for a new run.
    pub fn match_complete(self) -> PlayerSession<InLobby> {
        PlayerSession {
            player_id: self.player_id,
            username: self.username,
            created_at: self.created_at,
            _state: PhantomData,
        }
    }

    /// Connection dropped mid-match.
    pub fn disconnect(self) -> PlayerSession<Disconnected> {
        PlayerSession {
            player_id: self.player_id,
            username: self.username,
            created_at: self.created_at,
            _state: PhantomData,
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Spectating
// ────────────────────────────────────────────────────────────────────────────

impl PlayerSession<Spectating> {
    /// Stop watching and go back to the lobby.
    pub fn leave_spectate(self) -> PlayerSession<InLobby> {
        PlayerSession {
            player_id: self.player_id,
            username: self.username,
            created_at: self.created_at,
            _state: PhantomData,
        }
    }

    /// Disconnect while spectating.
    pub fn disconnect(self) -> PlayerSession<Disconnected> {
        PlayerSession {
            player_id: self.player_id,
            username: self.username,
            created_at: self.created_at,
            _state: PhantomData,
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Errors
// ────────────────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("authentication failed: invalid token")]
    AuthFailed,

    #[error("session already in state {0}")]
    AlreadyInState(&'static str),

    #[error("match not found: {0}")]
    MatchNotFound(u64),

    #[error("insufficient gold: need {need}, have {have}")]
    InsufficientGold { need: u32, have: u32 },
}
