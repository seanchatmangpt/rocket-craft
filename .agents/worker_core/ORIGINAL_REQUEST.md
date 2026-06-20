## 2026-06-19T00:44:43Z

Implement Milestone 2 (Core C++ Backbone ontology) by writing core.ttl and stub files for reflection.ttl, blueprints.ttl, subsystems.ttl, and typestates.ttl, and fixing ggen.toml validation rules.

## Task Details
1. **Modify ggen.toml**: Update `/Users/sac/.ggen/packs/ue4_ontology/ggen.toml` with inference rules, a dummy generation rule, and ensure all SPARQL queries have ORDER BY determinism:
   ```toml
   [inference]
   [[inference.rules]]
   name = "infer-is-component-of"
   construct = """
   PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
   CONSTRUCT {
     ?component ue4:isComponentOf ?actor .
   } WHERE {
     ?actor ue4:hasComponent ?component .
   } ORDER BY ?actor ?component
   """

   [[inference.rules]]
   name = "infer-is-level-of"
   construct = """
   PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
   CONSTRUCT {
     ?level ue4:isLevelOf ?world .
   } WHERE {
     ?world ue4:hasLevel ?level .
   } ORDER BY ?world ?level
   """

   [generation]
   rules = [
     { name = "readme", query = { inline = "SELECT * WHERE { ?s ?p ?o } ORDER BY ?s LIMIT 1" }, template = { inline = "# UE4 Ontology\n" }, output_file = "README.md", mode = "Overwrite" }
   ]
   ```
2. **Author core.ttl**: Write the complete static class hierarchy ontology at `/Users/sac/.ggen/packs/ue4_ontology/core.ttl` including UObject, AActor, APawn, ACharacter, UActorComponent, USceneComponent, UWorld, and ULevel, with hasComponent, hasRootComponent, hasOwner/owner, hasLevel, persistentLevel, hasActor, bReplicates, bIsActive, bHidden properties. Make sure all classes and properties have labels and comments, and namespace resolves to `https://rocket-craft.io/ontology/ue4/`.
3. **Author Stubs**: Write stub ontologies for `reflection.ttl`, `blueprints.ttl`, `subsystems.ttl`, and `typestates.ttl` in `/Users/sac/.ggen/packs/ue4_ontology/` using the designs from `/Users/sac/rocket-craft/.agents/explorer_core_1/analysis.md` (or explorer_core_3).
4. **Verify**: Run `/Users/sac/rocket-craft/validate_ontology.sh` and ensure validation succeeds with exit code 0.
