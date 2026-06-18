use crate::spec::{RelationshipType, RuleSeverity, Vector3, WorldSpec};
use std::collections::{HashMap, HashSet};
use unify_core::{Admit, Refusal, StaticLaw};
use unify_rdf::{
    shacl::{validate as shacl_validate, ShaclConstraint, ShaclShape},
    store::TripleStore,
    triple::{Term, Triple},
};

/// Law constraint representing semantic and structural coherence of a manufactured world.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct WorldCoherenceLaw;

impl StaticLaw for WorldCoherenceLaw {
    const NAME: &'static str = "WorldCoherence";
    const DESCRIPTION: &'static str = "Enforces referential integrity, unique IDs, positive boundaries, finite floating-point values, acyclic place hierarchy, and SHACL compliance";
}

/// Admission gate enforcing the WorldCoherenceLaw.
pub struct WorldCoherenceGate;

impl Default for WorldCoherenceGate {
    fn default() -> Self {
        Self::new()
    }
}

impl WorldCoherenceGate {
    /// Construct a new gate.
    pub fn new() -> Self {
        WorldCoherenceGate
    }

    /// Check if the WorldSpec conforms to all registered laws.
    /// Returns `Ok(())` on success, or `Err(Vec<String>)` containing all validation errors.
    pub fn validate(&self, spec: &WorldSpec) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // 1. ID Uniqueness
        let mut global_entity_ids = HashSet::new();
        for place in &spec.places {
            if !global_entity_ids.insert(place.id.clone()) {
                errors.push(format!(
                    "Duplicate Entity ID '{}' found in places",
                    place.id
                ));
            }
        }
        for actor in &spec.actors {
            if !global_entity_ids.insert(actor.id.clone()) {
                errors.push(format!(
                    "Duplicate Entity ID '{}' found in actors",
                    actor.id
                ));
            }
        }
        for obj in &spec.objects {
            if !global_entity_ids.insert(obj.id.clone()) {
                errors.push(format!("Duplicate Entity ID '{}' found in objects", obj.id));
            }
        }

        let mut rel_ids = HashSet::new();
        for rel in &spec.relationships {
            if !rel_ids.insert(rel.id.clone()) {
                errors.push(format!("Duplicate Relationship ID '{}'", rel.id));
            }
        }

        let mut rule_ids = HashSet::new();
        for rule in &spec.rules {
            if !rule_ids.insert(rule.id.clone()) {
                errors.push(format!("Duplicate Rule ID '{}'", rule.id));
            }
        }

        let mut proc_ids = HashSet::new();
        for proc in &spec.processes {
            if !proc_ids.insert(proc.id.clone()) {
                errors.push(format!("Duplicate Process ID '{}'", proc.id));
            }
        }

        let mut evt_ids = HashSet::new();
        for evt in &spec.history {
            if !evt_ids.insert(evt.id.clone()) {
                errors.push(format!("Duplicate HistoryEvent ID '{}'", evt.id));
            }
        }

        // Gather lookup sets
        let place_ids: HashSet<&str> = spec.places.iter().map(|p| p.id.as_str()).collect();
        let actor_ids: HashSet<&str> = spec.actors.iter().map(|a| a.id.as_str()).collect();
        let object_ids: HashSet<&str> = spec.objects.iter().map(|o| o.id.as_str()).collect();
        let valid_entity_ids: HashSet<&str> = place_ids
            .iter()
            .copied()
            .chain(actor_ids.iter().copied())
            .chain(object_ids.iter().copied())
            .collect();

        // 2. Referential Integrity Check
        for actor in &spec.actors {
            if !place_ids.contains(actor.place_id.as_str()) {
                errors.push(format!(
                    "Referential Integrity Violation: Actor '{}' points to non-existent place '{}'",
                    actor.id, actor.place_id
                ));
            }
        }

        for object in &spec.objects {
            if !place_ids.contains(object.place_id.as_str()) {
                errors.push(format!(
                    "Referential Integrity Violation: Object '{}' points to non-existent place '{}'",
                    object.id, object.place_id
                ));
            }
        }

        for rel in &spec.relationships {
            if !valid_entity_ids.contains(rel.source.as_str()) {
                errors.push(format!(
                    "Referential Integrity Violation: Relationship '{}' source '{}' does not exist",
                    rel.id, rel.source
                ));
            }
            if !valid_entity_ids.contains(rel.target.as_str()) {
                errors.push(format!(
                    "Referential Integrity Violation: Relationship '{}' target '{}' does not exist",
                    rel.id, rel.target
                ));
            }
        }

        for place in &spec.places {
            if let Some(parent_id) = &place.parent_place_id {
                if !place_ids.contains(parent_id.as_str()) {
                    errors.push(format!(
                        "Referential Integrity Violation: Place '{}' parent_place_id '{}' does not exist",
                        place.id, parent_id
                    ));
                }
            }
        }

        // Rules check
        let rule_entity_re = regex::Regex::new(r"\b([a-zA-Z_][a-zA-Z0-9_-]*)\.").unwrap();
        for rule in &spec.rules {
            for cap in rule_entity_re.captures_iter(&rule.expression) {
                let potential_id = &cap[1];
                if !valid_entity_ids.contains(potential_id) {
                    errors.push(format!(
                        "Referential Integrity Violation: Rule '{}' expression '{}' references non-existent entity '{}'",
                        rule.id, rule.expression, potential_id
                    ));
                }
            }
        }

        // Process step check
        for proc in &spec.processes {
            for step in &proc.steps {
                if let Some(actor_id) = &step.assigned_actor {
                    if !actor_ids.contains(actor_id.as_str()) {
                        errors.push(format!(
                            "Referential Integrity Violation: Process '{}' Step {} assigned actor '{}' does not exist",
                            proc.id, step.step_number, actor_id
                        ));
                    }
                }
                for input in &step.inputs {
                    if !valid_entity_ids.contains(input.as_str()) {
                        errors.push(format!(
                            "Referential Integrity Violation: Process '{}' Step {} input entity '{}' does not exist",
                            proc.id, step.step_number, input
                        ));
                    }
                }
                for output in &step.outputs {
                    if !valid_entity_ids.contains(output.as_str()) {
                        errors.push(format!(
                            "Referential Integrity Violation: Process '{}' Step {} output entity '{}' does not exist",
                            proc.id, step.step_number, output
                        ));
                    }
                }
            }
        }

        // HistoryEvent actor check
        for event in &spec.history {
            if let Some(actor_id) = &event.actor_id {
                if !actor_ids.contains(actor_id.as_str()) {
                    errors.push(format!(
                        "Referential Integrity Violation: HistoryEvent '{}' references non-existent actor '{}'",
                        event.id, actor_id
                    ));
                }
            }
        }

        // 3. Bounds validation
        for place in &spec.places {
            if place.bounds.half_extents.x <= 0.0
                || place.bounds.half_extents.y <= 0.0
                || place.bounds.half_extents.z <= 0.0
            {
                errors.push(format!(
                    "Bounds Violation: Place '{}' half_extents ({}, {}, {}) must be positive",
                    place.id,
                    place.bounds.half_extents.x,
                    place.bounds.half_extents.y,
                    place.bounds.half_extents.z
                ));
            }
        }

        for proc in &spec.processes {
            let mut step_numbers = HashSet::new();
            for step in &proc.steps {
                if step.duration_seconds < 0.0 {
                    errors.push(format!(
                        "Process Step Violation: Process '{}' Step {} duration_seconds ({}) cannot be negative",
                        proc.id, step.step_number, step.duration_seconds
                    ));
                }
                if step.step_number == 0 {
                    errors.push(format!(
                        "Process Step Violation: Process '{}' step number cannot be 0",
                        proc.id
                    ));
                }
                if !step_numbers.insert(step.step_number) {
                    errors.push(format!(
                        "Process Step Violation: Process '{}' has duplicate step number {}",
                        proc.id, step.step_number
                    ));
                }
            }
        }

        // 4. Floating-point coordinates validation (prevent NaN/Infinity)
        let check_finite =
            |v: Vector3| -> bool { v.x.is_finite() && v.y.is_finite() && v.z.is_finite() };

        for place in &spec.places {
            if !check_finite(place.bounds.center) {
                errors.push(format!("Floating-point Safety Violation: Place '{}' center coordinates contain NaN/Infinity: ({}, {}, {})",
                    place.id, place.bounds.center.x, place.bounds.center.y, place.bounds.center.z));
            }
            if !check_finite(place.bounds.half_extents) {
                errors.push(format!("Floating-point Safety Violation: Place '{}' half_extents coordinates contain NaN/Infinity: ({}, {}, {})",
                    place.id, place.bounds.half_extents.x, place.bounds.half_extents.y, place.bounds.half_extents.z));
            }
        }

        for actor in &spec.actors {
            if !check_finite(actor.placement.position) {
                errors.push(format!("Floating-point Safety Violation: Actor '{}' position coordinates contain NaN/Infinity: ({}, {}, {})",
                    actor.id, actor.placement.position.x, actor.placement.position.y, actor.placement.position.z));
            }
            if !check_finite(actor.placement.rotation) {
                errors.push(format!("Floating-point Safety Violation: Actor '{}' rotation coordinates contain NaN/Infinity: ({}, {}, {})",
                    actor.id, actor.placement.rotation.x, actor.placement.rotation.y, actor.placement.rotation.z));
            }
        }

        for obj in &spec.objects {
            if !check_finite(obj.placement.position) {
                errors.push(format!("Floating-point Safety Violation: Object '{}' position coordinates contain NaN/Infinity: ({}, {}, {})",
                    obj.id, obj.placement.position.x, obj.placement.position.y, obj.placement.position.z));
            }
            if !check_finite(obj.placement.rotation) {
                errors.push(format!("Floating-point Safety Violation: Object '{}' rotation coordinates contain NaN/Infinity: ({}, {}, {})",
                    obj.id, obj.placement.rotation.x, obj.placement.rotation.y, obj.placement.rotation.z));
            }
        }

        for proc in &spec.processes {
            for step in &proc.steps {
                if !step.duration_seconds.is_finite() {
                    errors.push(format!(
                        "Floating-point Safety Violation: Process '{}' Step {} duration_seconds ({}) is NaN/Infinity",
                        proc.id, step.step_number, step.duration_seconds
                    ));
                }
            }
        }

        // 5. Containment Cycle detection
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();
        for place in &spec.places {
            if let Some(parent_id) = &place.parent_place_id {
                adj.entry(parent_id.clone())
                    .or_default()
                    .push(place.id.clone());
            }
        }
        for rel in &spec.relationships {
            if rel.rel_type == RelationshipType::Contains {
                adj.entry(rel.source.clone())
                    .or_default()
                    .push(rel.target.clone());
            }
        }

        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        fn dfs(
            node: &str,
            adj: &HashMap<String, Vec<String>>,
            visited: &mut HashSet<String>,
            rec_stack: &mut HashSet<String>,
            path: &mut Vec<String>,
            errors: &mut Vec<String>,
        ) {
            visited.insert(node.to_string());
            rec_stack.insert(node.to_string());
            path.push(node.to_string());

            if let Some(neighbors) = adj.get(node) {
                for neighbor in neighbors {
                    if rec_stack.contains(neighbor) {
                        let cycle_start = path.iter().position(|x| x == neighbor).unwrap();
                        let mut cycle_path = path[cycle_start..].to_vec();
                        cycle_path.push(neighbor.clone());
                        errors.push(format!(
                            "Cyclic Containment Violation: containment cycle detected: {}",
                            cycle_path.join(" -> ")
                        ));
                    } else if !visited.contains(neighbor) {
                        dfs(neighbor, adj, visited, rec_stack, path, errors);
                    }
                }
            }

            path.pop();
            rec_stack.remove(node);
        }

        let all_nodes: HashSet<String> = adj.keys().cloned().collect();
        for node in all_nodes {
            if !visited.contains(&node) {
                let mut path = Vec::new();
                dfs(
                    &node,
                    &adj,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                    &mut errors,
                );
            }
        }

        // 6. SHACL compliance
        let store = spec_to_triples(spec);
        let shapes = get_genie_shacl_shapes();
        let shacl_res = shacl_validate(&store, &shapes);
        if !shacl_res.conforms {
            for violation in shacl_res.violations {
                errors.push(format!(
                    "SHACL Constraint Violation: Node '{:?}', Path '{:?}', Message: '{}'",
                    violation.node, violation.path, violation.message
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Admit<WorldCoherenceLaw> for WorldCoherenceGate {
    type Artifact = WorldSpec;
    type Refusal = Refusal<WorldCoherenceLaw>;

    fn admit(&self, spec: &WorldSpec) -> Result<(), Self::Refusal> {
        self.validate(spec)
            .map_err(|errs| Refusal::new(errs.join("\n")))
    }
}

/// Convert a structured `WorldSpec` into a `TripleStore`.
pub fn spec_to_triples(spec: &WorldSpec) -> TripleStore {
    let mut store = TripleStore::new();
    let genie_ns = "genie:";

    let literal = |val: &str, dt: &str| -> Term {
        Term::Literal {
            value: val.to_string(),
            datatype: Some(dt.to_string()),
            lang: None,
        }
    };

    // 1. Translate Places
    for place in &spec.places {
        let subj = Term::Named(format!("{}place:{}", genie_ns, place.id));
        store.add(Triple::new(
            subj.clone(),
            "rdf:type",
            format!("{}Place", genie_ns),
        ));
        store.add(Triple::new(
            subj.clone(),
            format!("{}name", genie_ns),
            literal(&place.name, "xsd:string"),
        ));
        if let Some(desc) = &place.description {
            store.add(Triple::new(
                subj.clone(),
                format!("{}description", genie_ns),
                literal(desc, "xsd:string"),
            ));
        }
        if let Some(parent) = &place.parent_place_id {
            store.add(Triple::new(
                subj.clone(),
                format!("{}parentPlace", genie_ns),
                Term::Named(format!("{}place:{}", genie_ns, parent)),
            ));
        }

        // Bounds mapping (using Blank Nodes)
        let bounds_bnode = Term::Blank(format!("bounds_{}", place.id));
        store.add(Triple::new(
            subj.clone(),
            format!("{}hasBounds", genie_ns),
            bounds_bnode.clone(),
        ));
        store.add(Triple::new(
            bounds_bnode.clone(),
            "rdf:type",
            format!("{}Bounds3D", genie_ns),
        ));
        store.add(Triple::new(
            bounds_bnode.clone(),
            format!("{}centerX", genie_ns),
            literal(&place.bounds.center.x.to_string(), "xsd:float"),
        ));
        store.add(Triple::new(
            bounds_bnode.clone(),
            format!("{}centerY", genie_ns),
            literal(&place.bounds.center.y.to_string(), "xsd:float"),
        ));
        store.add(Triple::new(
            bounds_bnode.clone(),
            format!("{}centerZ", genie_ns),
            literal(&place.bounds.center.z.to_string(), "xsd:float"),
        ));
        store.add(Triple::new(
            bounds_bnode.clone(),
            format!("{}halfExtentX", genie_ns),
            literal(&place.bounds.half_extents.x.to_string(), "xsd:float"),
        ));
        store.add(Triple::new(
            bounds_bnode.clone(),
            format!("{}halfExtentY", genie_ns),
            literal(&place.bounds.half_extents.y.to_string(), "xsd:float"),
        ));
        store.add(Triple::new(
            bounds_bnode.clone(),
            format!("{}halfExtentZ", genie_ns),
            literal(&place.bounds.half_extents.z.to_string(), "xsd:float"),
        ));
    }

    // 2. Translate Actors
    for actor in &spec.actors {
        let subj = Term::Named(format!("{}actor:{}", genie_ns, actor.id));
        store.add(Triple::new(
            subj.clone(),
            "rdf:type",
            format!("{}Actor", genie_ns),
        ));
        store.add(Triple::new(
            subj.clone(),
            format!("{}name", genie_ns),
            literal(&actor.name, "xsd:string"),
        ));
        store.add(Triple::new(
            subj.clone(),
            format!("{}role", genie_ns),
            literal(&actor.role, "xsd:string"),
        ));
        store.add(Triple::new(
            subj.clone(),
            format!("{}place", genie_ns),
            Term::Named(format!("{}place:{}", genie_ns, actor.place_id)),
        ));
        // Backward compatibility shape matching (inPlace vs place)
        store.add(Triple::new(
            subj.clone(),
            format!("{}inPlace", genie_ns),
            Term::Named(format!("{}place:{}", genie_ns, actor.place_id)),
        ));
    }

    // 3. Translate Objects
    for object in &spec.objects {
        let subj = Term::Named(format!("{}object:{}", genie_ns, object.id));
        store.add(Triple::new(
            subj.clone(),
            "rdf:type",
            format!("{}Object", genie_ns),
        ));
        store.add(Triple::new(
            subj.clone(),
            format!("{}name", genie_ns),
            literal(&object.name, "xsd:string"),
        ));
        store.add(Triple::new(
            subj.clone(),
            format!("{}class", genie_ns),
            literal(&object.class, "xsd:string"),
        ));
        store.add(Triple::new(
            subj.clone(),
            format!("{}place", genie_ns),
            Term::Named(format!("{}place:{}", genie_ns, object.place_id)),
        ));
        store.add(Triple::new(
            subj.clone(),
            format!("{}inPlace", genie_ns),
            Term::Named(format!("{}place:{}", genie_ns, object.place_id)),
        ));
    }

    // 4. Translate Relationships
    for rel in &spec.relationships {
        let subj = Term::Named(format!("{}relationship:{}", genie_ns, rel.id));
        store.add(Triple::new(
            subj.clone(),
            "rdf:type",
            format!("{}Relationship", genie_ns),
        ));

        let type_str = match &rel.rel_type {
            RelationshipType::Connects => "connects",
            RelationshipType::Contains => "contains",
            RelationshipType::Owns => "owns",
            RelationshipType::AdjacentTo => "adjacent_to",
            RelationshipType::Controls => "controls",
            RelationshipType::Custom(s) => s.as_str(),
        };
        store.add(Triple::new(
            subj.clone(),
            format!("{}relType", genie_ns),
            Term::Named(format!("{}relationshipType:{}", genie_ns, type_str)),
        ));
        store.add(Triple::new(
            subj.clone(),
            format!("{}relationshipType", genie_ns),
            literal(type_str, "xsd:string"),
        ));

        // Find correct prefix for source and target
        let resolve_iri = |id: &str| -> Term {
            if spec.places.iter().any(|p| p.id == id) {
                Term::Named(format!("{}place:{}", genie_ns, id))
            } else if spec.actors.iter().any(|a| a.id == id) {
                Term::Named(format!("{}actor:{}", genie_ns, id))
            } else if spec.objects.iter().any(|o| o.id == id) {
                Term::Named(format!("{}object:{}", genie_ns, id))
            } else {
                Term::Named(format!("{}entity:{}", genie_ns, id))
            }
        };

        let source_iri = resolve_iri(&rel.source);
        let target_iri = resolve_iri(&rel.target);
        store.add(Triple::new(
            subj.clone(),
            format!("{}source", genie_ns),
            source_iri.clone(),
        ));
        store.add(Triple::new(
            subj.clone(),
            format!("{}target", genie_ns),
            target_iri.clone(),
        ));

        // Direct statement triple
        let predicate_iri = Term::Named(format!("{}{}", genie_ns, type_str));
        store.add(Triple::new(source_iri, predicate_iri, target_iri));
    }

    // 5. Translate Rules
    for rule in &spec.rules {
        let subj = Term::Named(format!("{}rule:{}", genie_ns, rule.id));
        store.add(Triple::new(
            subj.clone(),
            "rdf:type",
            format!("{}Rule", genie_ns),
        ));
        store.add(Triple::new(
            subj.clone(),
            format!("{}name", genie_ns),
            literal(&rule.name, "xsd:string"),
        ));
        store.add(Triple::new(
            subj.clone(),
            format!("{}expression", genie_ns),
            literal(&rule.expression, "xsd:string"),
        ));
        let sev_str = match &rule.severity {
            RuleSeverity::Info => "info",
            RuleSeverity::Warning => "warning",
            RuleSeverity::Error => "error",
        };
        store.add(Triple::new(
            subj.clone(),
            format!("{}severity", genie_ns),
            Term::Named(format!("{}ruleSeverity:{}", genie_ns, sev_str)),
        ));
    }

    store
}

/// Define custom SHACL shapes and constraints for validation.
pub fn get_genie_shacl_shapes() -> Vec<ShaclShape> {
    let genie_ns = "genie:";
    vec![
        // Shape validating Actor entities
        ShaclShape {
            target_class: Term::Named(format!("{}Actor", genie_ns)),
            constraints: vec![
                ShaclConstraint::MinCount {
                    path: Term::Named(format!("{}name", genie_ns)),
                    count: 1,
                },
                ShaclConstraint::MaxCount {
                    path: Term::Named(format!("{}name", genie_ns)),
                    count: 1,
                },
                ShaclConstraint::Datatype {
                    path: Term::Named(format!("{}name", genie_ns)),
                    datatype: "xsd:string".to_string(),
                },
                ShaclConstraint::MinCount {
                    path: Term::Named(format!("{}role", genie_ns)),
                    count: 1,
                },
                ShaclConstraint::MaxCount {
                    path: Term::Named(format!("{}role", genie_ns)),
                    count: 1,
                },
                ShaclConstraint::Datatype {
                    path: Term::Named(format!("{}role", genie_ns)),
                    datatype: "xsd:string".to_string(),
                },
                ShaclConstraint::MinCount {
                    path: Term::Named(format!("{}place", genie_ns)),
                    count: 1,
                },
                ShaclConstraint::MaxCount {
                    path: Term::Named(format!("{}place", genie_ns)),
                    count: 1,
                },
                ShaclConstraint::NodeKind {
                    path: Term::Named(format!("{}place", genie_ns)),
                    kind: "IRI".to_string(),
                },
            ],
        },
        // Shape validating Place entities
        ShaclShape {
            target_class: Term::Named(format!("{}Place", genie_ns)),
            constraints: vec![
                ShaclConstraint::MinCount {
                    path: Term::Named(format!("{}name", genie_ns)),
                    count: 1,
                },
                ShaclConstraint::MaxCount {
                    path: Term::Named(format!("{}name", genie_ns)),
                    count: 1,
                },
                ShaclConstraint::Datatype {
                    path: Term::Named(format!("{}name", genie_ns)),
                    datatype: "xsd:string".to_string(),
                },
                ShaclConstraint::MaxCount {
                    path: Term::Named(format!("{}parentPlace", genie_ns)),
                    count: 1,
                },
                ShaclConstraint::NodeKind {
                    path: Term::Named(format!("{}parentPlace", genie_ns)),
                    kind: "IRI".to_string(),
                },
                ShaclConstraint::MinCount {
                    path: Term::Named(format!("{}hasBounds", genie_ns)),
                    count: 1,
                },
                ShaclConstraint::MaxCount {
                    path: Term::Named(format!("{}hasBounds", genie_ns)),
                    count: 1,
                },
                ShaclConstraint::NodeKind {
                    path: Term::Named(format!("{}hasBounds", genie_ns)),
                    kind: "BlankNode".to_string(),
                },
            ],
        },
        // Shape validating Object entities
        ShaclShape {
            target_class: Term::Named(format!("{}Object", genie_ns)),
            constraints: vec![
                ShaclConstraint::MinCount {
                    path: Term::Named(format!("{}name", genie_ns)),
                    count: 1,
                },
                ShaclConstraint::MaxCount {
                    path: Term::Named(format!("{}name", genie_ns)),
                    count: 1,
                },
                ShaclConstraint::Datatype {
                    path: Term::Named(format!("{}name", genie_ns)),
                    datatype: "xsd:string".to_string(),
                },
                ShaclConstraint::MinCount {
                    path: Term::Named(format!("{}class", genie_ns)),
                    count: 1,
                },
                ShaclConstraint::MaxCount {
                    path: Term::Named(format!("{}class", genie_ns)),
                    count: 1,
                },
                ShaclConstraint::Datatype {
                    path: Term::Named(format!("{}class", genie_ns)),
                    datatype: "xsd:string".to_string(),
                },
                ShaclConstraint::MinCount {
                    path: Term::Named(format!("{}place", genie_ns)),
                    count: 1,
                },
                ShaclConstraint::MaxCount {
                    path: Term::Named(format!("{}place", genie_ns)),
                    count: 1,
                },
                ShaclConstraint::NodeKind {
                    path: Term::Named(format!("{}place", genie_ns)),
                    kind: "IRI".to_string(),
                },
            ],
        },
        // Shape validating Relationship entities
        ShaclShape {
            target_class: Term::Named(format!("{}Relationship", genie_ns)),
            constraints: vec![
                ShaclConstraint::MinCount {
                    path: Term::Named(format!("{}source", genie_ns)),
                    count: 1,
                },
                ShaclConstraint::MaxCount {
                    path: Term::Named(format!("{}source", genie_ns)),
                    count: 1,
                },
                ShaclConstraint::MinCount {
                    path: Term::Named(format!("{}target", genie_ns)),
                    count: 1,
                },
                ShaclConstraint::MaxCount {
                    path: Term::Named(format!("{}target", genie_ns)),
                    count: 1,
                },
            ],
        },
    ]
}
