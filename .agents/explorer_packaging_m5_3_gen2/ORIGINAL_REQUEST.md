## 2026-06-19T05:46:25Z

You are the Packaging and RHI Target Config Explorer (Explorer 3) for the UE4 Universal RDF Mapping project.
Your working directory is `/Users/sac/rocket-craft/.agents/explorer_packaging_m5_3_gen2`.
Your task is to explore and analyze how to model packaging targets and RHI configurations in `/Users/sac/.ggen/packs/ue4_ontology/typestates.ttl`.
Specifically:
1. Read the current `/Users/sac/.ggen/packs/ue4_ontology/typestates.ttl` and `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl`.
2. Propose concrete RDF classes, properties, and relationships to model:
   - Packaging targets (debug, development, shipping builds)
   - WebGL 2.0 / OpenGL ES3 target RHI profiles
   - The Projection Law constraints (statically baked world structure, preventing generation of WebGL assets directly from ontology, enforcing GGen-only semantic authority, headers/DataTables/BOM/walkthrough/byte-class output paths)
3. Suggest custom SHACL validation shapes or SPARQL rules to validate packaging configurations.
Write your analysis and recommendations to a file named `analysis.md` in your working directory. Then, write a handoff report at `handoff.md` and send a message back to the parent orchestrator (conversation ID: 26a55229-63bc-48fa-bc48-7ec491f0dfa3) with the path to your handoff.md.
