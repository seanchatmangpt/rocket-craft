use anyhow::Result;
use clap::Subcommand;

/// A trait for "nouns" in a clap-based CLI that can execute "verbs" (subcommands).
///
/// This pattern allows mapping CLI subcommands directly to SDK actions.
pub trait ClapNoun {
    /// The enum representing the available verbs (subcommands) for this noun.
    type Verb: Subcommand;

    /// Executes the given verb on this noun.
    fn handle(&mut self, verb: Self::Verb) -> Result<()>;
}
