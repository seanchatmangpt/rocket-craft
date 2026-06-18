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
///
/// # Examples
///
/// ```
/// use nexus_session::session::{PlayerSession, Connecting, Authenticated};
///
/// // Create pilot session
/// let session = PlayerSession::new(42, "amuro_ray".to_string());
/// assert_eq!(session.player_id, 42);
///
/// // Connecting -> Authenticated transition (token validated)
/// let authenticated = session.authenticate(true).unwrap();
///
/// // Authenticated -> InLobby transition
/// let lobby = authenticated.enter_lobby();
/// ```
pub struct PlayerSession<S> {
    pub player_id: u64,
    pub username: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    // Private — forces use of transition methods rather than direct construction.
    _state: PhantomData<S>,
}

/// Errors arising from invalid PlayerSession construction.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum SessionBuildError {
    /// Username cannot be empty.
    #[error("username is required and cannot be empty")]
    EmptyUsername,

    /// Player ID must be non-zero.
    #[error("player_id must be non-zero")]
    ZeroPlayerId,
}

/// A builder for [`PlayerSession`] to initialize player session details with validation.
///
/// # Examples
///
/// ```
/// use nexus_session::session::{PlayerSessionBuilder, Connecting};
///
/// let session = PlayerSessionBuilder::new()
///     .player_id(12345)
///     .username("gundam_pilot".to_string())
///     .build()
///     .unwrap();
///
/// assert_eq!(session.player_id, 12345);
/// assert_eq!(session.username, "gundam_pilot");
/// ```
#[derive(Debug, Clone)]
pub struct PlayerSessionBuilder {
    player_id: Option<u64>,
    username: Option<String>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl PlayerSessionBuilder {
    /// Create a new builder with default parameters.
    pub fn new() -> Self {
        Self {
            player_id: None,
            username: None,
            created_at: None,
        }
    }

    /// Set the player ID.
    pub fn player_id(mut self, player_id: u64) -> Self {
        self.player_id = Some(player_id);
        self
    }

    /// Set the pilot's username.
    pub fn username(mut self, username: String) -> Self {
        self.username = Some(username);
        self
    }

    /// Set the session creation time. If not specified, defaults to the current time.
    pub fn created_at(mut self, created_at: chrono::DateTime<chrono::Utc>) -> Self {
        self.created_at = Some(created_at);
        self
    }

    /// Validate the parameters and build a [`PlayerSession`] in [`Connecting`] state.
    pub fn build(self) -> Result<PlayerSession<Connecting>, SessionBuildError> {
        let player_id = self.player_id.ok_or(SessionBuildError::ZeroPlayerId)?;
        if player_id == 0 {
            return Err(SessionBuildError::ZeroPlayerId);
        }
        let username = self.username.ok_or(SessionBuildError::EmptyUsername)?;
        if username.trim().is_empty() {
            return Err(SessionBuildError::EmptyUsername);
        }
        let created_at = self.created_at.unwrap_or_else(chrono::Utc::now);

        Ok(PlayerSession {
            player_id,
            username,
            created_at,
            _state: PhantomData,
        })
    }
}

impl Default for PlayerSessionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents the dynamic runtime state of a player session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SessionState {
    Connecting,
    Authenticated,
    InLobby,
    InMatch,
    Spectating,
    Disconnected,
}

/// Errors returned when a session state transition is invalid.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("Illegal session state transition: cannot transition from {current:?} to {target:?}. Reason: {reason}")]
pub struct SessionTransitionError {
    pub current: SessionState,
    pub target: SessionState,
    pub reason: String,
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
