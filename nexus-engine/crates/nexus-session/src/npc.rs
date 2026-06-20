use crate::inventory::NpcInventory;

// ────────────────────────────────────────────────────────────────────────────
// NPC state machine
// ────────────────────────────────────────────────────────────────────────────

/// All states an NPC can occupy.  `InDialogue` carries the active node index
/// so the game server can resume an interrupted conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcState {
    Idle,
    Greeting,
    InDialogue { node: u32 },
    Trading,
    Hostile,
    Fleeing,
    Dead,
}

// ────────────────────────────────────────────────────────────────────────────
// Dialogue tree
// ────────────────────────────────────────────────────────────────────────────

/// A branching dialogue tree stored as a flat `Vec` of nodes addressed by id.
pub struct NpcDialogueTree {
    pub nodes: Vec<DialogueNode>,
    pub current_node: u32,
}

impl NpcDialogueTree {
    /// Construct a tree with a pre-built node list.
    pub fn new(nodes: Vec<DialogueNode>) -> Self {
        NpcDialogueTree {
            nodes,
            current_node: 0,
        }
    }

    /// Return a minimal single-node "no dialogue" tree.
    pub fn empty() -> Self {
        NpcDialogueTree {
            nodes: vec![DialogueNode {
                id: 0,
                speaker: String::from("???"),
                text: String::from("..."),
                responses: Vec::new(),
                triggers_trade: false,
                triggers_quest: None,
            }],
            current_node: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DialogueNode {
    pub id: u32,
    pub speaker: String,
    pub text: String,
    pub responses: Vec<DialogueResponse>,
    /// If `true`, the player can open the shop from this node.
    pub triggers_trade: bool,
    /// If `Some`, reaching this node triggers the named quest.
    pub triggers_quest: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DialogueResponse {
    pub text: String,
    /// `None` ends the dialogue; `Some(n)` jumps to node `n`.
    pub next_node: Option<u32>,
    /// Minimum bloodline level required to select this response.
    pub unlock_requires: Option<u32>,
}

// ────────────────────────────────────────────────────────────────────────────
// NPC types
// ────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcType {
    Merchant,
    QuestGiver,
    Ally,
    NeutralDeathless,
    Saydhi,
    Architect,
}

// ────────────────────────────────────────────────────────────────────────────
// Events & Actions
// ────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub enum NpcEvent {
    PlayerApproach,
    PlayerLeave,
    PlayerSpeak,
    /// The player selected response index `r` in the current dialogue node.
    SelectResponse(u32),
    OpenTrade,
    PlayerAttack,
}

#[derive(Debug, Clone)]
pub enum NpcAction {
    PlayGreeting,
    ShowDialogue(u32),
    EndDialogue,
    OpenShop,
    BecomeHostile,
    Flee,
}

// ────────────────────────────────────────────────────────────────────────────
// Npc
// ────────────────────────────────────────────────────────────────────────────

pub struct Npc {
    pub id: u64,
    pub name: String,
    pub npc_type: NpcType,
    pub state: NpcState,
    pub dialogue: NpcDialogueTree,
    pub inventory: NpcInventory,
    /// If `true` the NPC has a shop the player can browse.
    pub is_vendor: bool,
    /// Price multiplier applied to all shop items (e.g. `1.2` = 20 % markup).
    pub vendor_markup: f32,
}

impl Npc {
    /// Create a new NPC in the `Idle` state.
    pub fn new(
        id: u64,
        name: impl Into<String>,
        npc_type: NpcType,
        dialogue: NpcDialogueTree,
    ) -> Self {
        Npc {
            id,
            name: name.into(),
            npc_type,
            state: NpcState::Idle,
            dialogue,
            inventory: NpcInventory::new(),
            is_vendor: false,
            vendor_markup: 1.0,
        }
    }

    /// Drive the state machine with `event`.
    ///
    /// Returns `Some(action)` when the event triggers an observable side
    /// effect, or `None` when the event is ignored in the current state.
    pub fn transition(&mut self, event: NpcEvent) -> Option<NpcAction> {
        match (&self.state, event) {
            // ── Idle → Greeting ──────────────────────────────────────────────
            (NpcState::Idle, NpcEvent::PlayerApproach) => {
                self.state = NpcState::Greeting;
                Some(NpcAction::PlayGreeting)
            }

            // ── Greeting → InDialogue(0) ──────────────────────────────────
            (NpcState::Greeting, NpcEvent::PlayerSpeak) => {
                self.state = NpcState::InDialogue { node: 0 };
                Some(NpcAction::ShowDialogue(0))
            }

            // ── Greeting / InDialogue → Idle (player walks away) ──────────
            (NpcState::Greeting, NpcEvent::PlayerLeave)
            | (NpcState::InDialogue { .. }, NpcEvent::PlayerLeave) => {
                self.state = NpcState::Idle;
                Some(NpcAction::EndDialogue)
            }

            // ── InDialogue → InDialogue | Idle ───────────────────────────
            (NpcState::InDialogue { node }, NpcEvent::SelectResponse(r)) => {
                let node_idx = *node as usize;
                // Guard: dialogue tree must have the node.
                if node_idx >= self.dialogue.nodes.len() {
                    return None;
                }
                let responses = &self.dialogue.nodes[node_idx].responses;
                if let Some(response) = responses.get(r as usize) {
                    if let Some(next) = response.next_node {
                        self.state = NpcState::InDialogue { node: next };
                        Some(NpcAction::ShowDialogue(next))
                    } else {
                        self.state = NpcState::Idle;
                        Some(NpcAction::EndDialogue)
                    }
                } else {
                    // Invalid response index — ignore.
                    None
                }
            }

            // ── InDialogue → Trading (only when the node allows it) ───────
            (NpcState::InDialogue { node }, NpcEvent::OpenTrade) => {
                let node_idx = *node as usize;
                if node_idx < self.dialogue.nodes.len()
                    && self.dialogue.nodes[node_idx].triggers_trade
                    && self.is_vendor
                {
                    self.state = NpcState::Trading;
                    Some(NpcAction::OpenShop)
                } else {
                    None
                }
            }

            // ── Trading → Idle ────────────────────────────────────────────
            (NpcState::Trading, NpcEvent::PlayerLeave) => {
                self.state = NpcState::Idle;
                Some(NpcAction::EndDialogue)
            }

            // ── Any non-Dead state → Hostile when attacked ─────────────────
            (state, NpcEvent::PlayerAttack) if *state != NpcState::Dead => {
                self.state = NpcState::Hostile;
                Some(NpcAction::BecomeHostile)
            }

            // All other combinations are ignored.
            _ => None,
        }
    }

    /// Sell price for an item at this vendor (item base value × markup).
    pub fn sell_price(&self, base_value: u32) -> u32 {
        ((base_value as f32) * self.vendor_markup).round() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vendor() -> Npc {
        let tree = NpcDialogueTree::new(vec![
            DialogueNode {
                id: 0,
                speaker: "Arms Dealer".into(),
                text: "Welcome, pilot.".into(),
                responses: vec![
                    DialogueResponse {
                        text: "Show me your wares.".into(),
                        next_node: None,
                        unlock_requires: None,
                    },
                ],
                triggers_trade: true,
                triggers_quest: None,
            },
        ]);
        let mut npc = Npc::new(1, "Arms Dealer", NpcType::Merchant, tree);
        npc.is_vendor = true;
        npc.vendor_markup = 1.2;
        npc
    }

    fn neutral() -> Npc {
        Npc::new(2, "Guard", NpcType::NeutralDeathless, NpcDialogueTree::empty())
    }

    // ── idle → greeting ──────────────────────────────────────────────────────

    #[test]
    fn approach_triggers_greeting() {
        let mut npc = vendor();
        let action = npc.transition(NpcEvent::PlayerApproach);
        assert!(matches!(action, Some(NpcAction::PlayGreeting)));
        assert_eq!(npc.state, NpcState::Greeting);
    }

    // ── greeting → dialogue ───────────────────────────────────────────────────

    #[test]
    fn speak_advances_from_greeting_to_dialogue() {
        let mut npc = vendor();
        npc.transition(NpcEvent::PlayerApproach);
        let action = npc.transition(NpcEvent::PlayerSpeak);
        assert!(matches!(action, Some(NpcAction::ShowDialogue(0))));
        assert!(matches!(npc.state, NpcState::InDialogue { node: 0 }));
    }

    // ── dialogue → trade ──────────────────────────────────────────────────────

    #[test]
    fn open_trade_from_trade_node_returns_open_shop() {
        let mut npc = vendor();
        npc.transition(NpcEvent::PlayerApproach);
        npc.transition(NpcEvent::PlayerSpeak);
        let action = npc.transition(NpcEvent::OpenTrade);
        assert!(matches!(action, Some(NpcAction::OpenShop)));
        assert_eq!(npc.state, NpcState::Trading);
    }

    #[test]
    fn open_trade_on_non_vendor_returns_none() {
        let mut npc = neutral();
        npc.state = NpcState::InDialogue { node: 0 };
        let action = npc.transition(NpcEvent::OpenTrade);
        assert!(action.is_none(), "non-vendor must not open shop");
    }

    // ── player leave resets to idle ───────────────────────────────────────────

    #[test]
    fn leave_from_greeting_returns_to_idle() {
        let mut npc = vendor();
        npc.transition(NpcEvent::PlayerApproach);
        let action = npc.transition(NpcEvent::PlayerLeave);
        assert!(matches!(action, Some(NpcAction::EndDialogue)));
        assert_eq!(npc.state, NpcState::Idle);
    }

    #[test]
    fn leave_from_dialogue_returns_to_idle() {
        let mut npc = vendor();
        npc.transition(NpcEvent::PlayerApproach);
        npc.transition(NpcEvent::PlayerSpeak);
        npc.transition(NpcEvent::PlayerLeave);
        assert_eq!(npc.state, NpcState::Idle);
    }

    #[test]
    fn leave_from_trading_returns_to_idle() {
        let mut npc = vendor();
        npc.transition(NpcEvent::PlayerApproach);
        npc.transition(NpcEvent::PlayerSpeak);
        npc.transition(NpcEvent::OpenTrade);
        npc.transition(NpcEvent::PlayerLeave);
        assert_eq!(npc.state, NpcState::Idle);
    }

    // ── attack makes any NPC hostile ─────────────────────────────────────────

    #[test]
    fn attacking_idle_npc_makes_it_hostile() {
        let mut npc = neutral();
        let action = npc.transition(NpcEvent::PlayerAttack);
        assert!(matches!(action, Some(NpcAction::BecomeHostile)));
        assert_eq!(npc.state, NpcState::Hostile);
    }

    #[test]
    fn attacking_while_greeting_makes_npc_hostile() {
        let mut npc = vendor();
        npc.transition(NpcEvent::PlayerApproach);
        npc.transition(NpcEvent::PlayerAttack);
        assert_eq!(npc.state, NpcState::Hostile);
    }

    // ── attacking a dead NPC has no effect ───────────────────────────────────

    #[test]
    fn attacking_dead_npc_returns_none() {
        let mut npc = neutral();
        npc.state = NpcState::Dead;
        let action = npc.transition(NpcEvent::PlayerAttack);
        assert!(action.is_none(), "dead NPCs must not transition on attack");
        assert_eq!(npc.state, NpcState::Dead);
    }

    // ── sell_price ────────────────────────────────────────────────────────────

    #[test]
    fn vendor_sell_price_applies_markup() {
        let npc = vendor(); // markup = 1.2
        assert_eq!(npc.sell_price(100), 120);
    }

    #[test]
    fn sell_price_rounds_to_nearest_gold() {
        let npc = vendor(); // markup = 1.2
        // 10 * 1.2 = 12.0 → 12
        assert_eq!(npc.sell_price(10), 12);
        // 5 * 1.2 = 6.0 → 6
        assert_eq!(npc.sell_price(5), 6);
    }

    // ── invalid response index ────────────────────────────────────────────────

    #[test]
    fn out_of_range_response_index_returns_none() {
        let mut npc = vendor();
        npc.transition(NpcEvent::PlayerApproach);
        npc.transition(NpcEvent::PlayerSpeak);
        // dialogue node 0 has only 1 response (index 0); index 99 is invalid
        let action = npc.transition(NpcEvent::SelectResponse(99));
        assert!(action.is_none());
    }
}
