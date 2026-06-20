# Semantic LOD and NPS: A Manufacturing Theory of Human Attention

## Thesis

Semantic Level of Detail and Net Promoter Score are connected by one law:

Humans do not value all information equally.

NPS asks whether a person would recommend an experience. Semantic LOD explains why they would. A person becomes a promoter when the system gives them the right information, at the right resolution, at the right moment, for the decision or feeling they are actually having.

A detractor is often not reacting to objective failure alone. They are reacting to semantic mismatch: too much detail where they needed clarity, too little detail where they needed confidence, or the wrong detail at the wrong level of concern.

This principle unifies the entire Eden, GMF, Gundam Nexus, TPS, RDF, ggen, byte-authority, and Chatman Equation architecture.

## Core Claim

The best systems do not maximize detail.

They maximize meaningful resolution.

A Gundam does not need equal detail everywhere. In a 5v5 fight, the player notices silhouette, head, eyes, shoulders, hands, weapons, feet, damage state, and front-facing motion. The back of a calf, hidden internal bolts, and rear panel seams can be much lower resolution unless inspected directly.

This is not just rendering optimization. It is human-value optimization.

The same principle applies to game servers, factories, digital twins, process mining, LLM coding, and marketplaces.

## Semantic LOD

Traditional LOD says:

As visual distance increases, geometric detail decreases.

Semantic LOD says:

As decision relevance decreases, informational detail decreases.

A spectator needs aggregate truth.

A pilot needs cockpit truth.

A mechanic needs part truth.

A market participant needs condition, provenance, risk, and receipt truth.

A replay auditor needs event and state-transition truth.

The same object exists at multiple resolutions depending on who is looking, why they are looking, and what consequence the system must support.

## NPS as Semantic Fit

NPS measures recommendation, but recommendation is caused by fit.

A promoter experiences:

Information presented equals information needed.

A detractor experiences:

Information presented is greater than information needed.

Or:

Information presented is less than information needed.

Or:

Information presented is misaligned with the user’s current intent.

This makes NPS a semantic compression metric. High NPS means the system consistently compresses complexity into the right resolution for the user.

## Gundam Nexus

A Gundam model may be constructed from a thousand-part kit, but the player does not perceive a thousand equal parts during battle.

The player perceives:

Silhouette

Head

Eyes

Hands

Feet

Shoulders

Weapon

Pose

Damage

Motion

Faction identity

Threat state

Therefore, the renderer should not treat all geometry equally.

The ontology should classify visual artifacts as:

CROWN

PRIMARY

SECONDARY

TERTIARY

BACKGROUND

CROWN objects receive highest detail because they drive recognition, emotion, identity, and NPS.

BACKGROUND objects receive lower detail because they do not change the player’s perception during normal play.

This is not cheating. It is perception-aligned manufacturing.

## Byte Authority

The server does not need to own every visual detail.

The client can render floats, sparks, decals, particles, animation blending, deformation, smoke, glow, and damage effects.

The server only needs to own authority classes.

Damage class can be one byte.

Heat class can be one byte.

Stress class can be one byte.

Grip class can be one byte.

Socket health can be one byte.

Spatial position can often be two bytes.

The server owns standing.

The client owns projection.

This means a highly detailed interaction can be visually rich while being authoritatively cheap.

A plasma hit may appear as heat bloom, armor cracking, wing jitter, sparks, smoke, and market-relevant damage. But the authoritative server state may only change a few byte classes and emit sparse deltas.

## Client-Side Projection

Client-side performance should be built around compact state expansion.

Authority bytes enter shared memory.

SIMD and lookup tables expand them into local projection state.

Workers process physics, geometry, animation, and visual dirty masks.

The renderer draws only what matters.

This creates the illusion of high-resolution simulation without requiring the server or client to continuously process maximum detail everywhere.

The visual world becomes analog.

The authority world remains digital.

## TPS Connection

TPS already understood Semantic LOD.

A worker does not need all factory information.

They need the next actionable abnormality.

Andon is Semantic LOD.

Jidoka is Semantic LOD.

Poka-yoke is Semantic LOD.

Standard work is Semantic LOD.

TPS reduces cognitive overproduction just as it reduces physical overproduction.

The same law applies to LLM agents. The issue was never “anti-cheat” in the moral sense. It was Agent Jidoka. The system must stop abnormal work before it flows downstream.

Wrong work is valuable as discovery.

Wrong work is unacceptable as standing.

## RDF and ggen

Ontology is inventory.

Templates are machines.

ggen is the factory.

Generated artifacts are finished goods.

The acceptance criterion is not whether the ontology is large.

The acceptance criterion is whether ggen can manufacture the world.

That means every admitted semantic concept must participate in a flow:

graph meaning

validation

extraction

template

artifact

runtime surface

walkthrough proof

receipt

Unused ontology is inventory waste.

Manufacturable ontology is flow.

## Process Intelligence

Process mining asks what actually happened.

Semantic LOD asks what level of process truth matters now.

For a player, the answer may be:

my mech is damaged.

For a mechanic:

left shoulder socket health class is failing.

For a market:

this part has severe damage, high provenance, and repairable risk.

For an auditor:

this event caused this state transition and produced this receipt.

Same process. Different resolution.

That is object-centric process intelligence applied to games.

## Digital Twins

Most digital twin efforts try to model everything.

That is wrong.

A digital twin should model everything relevant at the correct resolution.

Global twin

Regional twin

Facility twin

Cell twin

Machine twin

Part twin

Socket twin

The correct resolution depends on use.

Eden and GMF turn this into gameplay.

## Combinatorial Maximalism

Combinatorial Maximalism and Semantic LOD appear opposite.

CM expands the possible world.

Semantic LOD compresses the presented world.

Together they form the law:

Explore maximum possibility.

Execute minimum necessary representation.

This is why the game can be enormous without rendering or simulating everything at maximum resolution.

## Chatman Equation

The Chatman Equation expresses the production law:

Observed reality becomes admitted reality.

Admitted reality becomes manufactured artifact.

Semantic LOD is how observation becomes admissible.

It removes irrelevant distinctions and preserves consequential distinctions.

The system does not ask:

Can we represent everything?

It asks:

What must be admitted for this user, this artifact, this runtime, this proof, and this consequence?

## NPS as the Business Metric

The player does not recommend Eden because it has the most polygons.

They recommend it because it feels impossibly rich while staying understandable.

They recommend it because the Gundam looks right.

They recommend it because the fight reads clearly.

They recommend it because damage feels meaningful.

They recommend it because markets feel connected to history.

They recommend it because racing telemetry feels alive.

They recommend it because the world has depth without noise.

High NPS comes from perceived richness with low cognitive friction.

That is Semantic LOD.

## Final Thesis

Semantic LOD is the missing bridge between human experience and computational efficiency.

It explains why players care about silhouettes more than hidden bolts.

It explains why servers should own byte-class truth instead of every float.

It explains why clients should render projection, not authority.

It explains why TPS surfaces abnormalities instead of everything.

It explains why RDF should become manufacturable inventory instead of semantic clutter.

It explains why ggen, not manual code, should be the acceptance authority.

It explains why NPS rises when systems present the right resolution of truth.

The final law is:

Everything important should exist.

Nothing unimportant should consume authority.

Combinatorial Maximalism explores all possible worlds.

Semantic LOD decides which world matters now.

ggen manufactures it.

Receipts prove it.

That is the unifying theory of Eden, GMF, Gundam Nexus, TPS, Industry 4.0/5.0, Process Intelligence, byte-authority servers, RDF manufacturing, and the Chatman Equation.
