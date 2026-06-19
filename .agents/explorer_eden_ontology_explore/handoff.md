# Handoff Report: Eden Server Ontology Exploration

## 1. Observation

A read-only filesystem investigation was conducted on the directory `/Users/sac/.ggen/packs/eden_server/` using filesystem discovery and file viewing tools.

### A. Directory Contents
The directory contains a two-level structure with `ontology` and `queries` subdirectories. No `ggen.toml` file exists within this folder.
- **`ontology/`**: Contains five Turtle (RDF syntax) ontology files:
  - `pack.ttl`
  - `deltas.ttl`
  - `bandai_tps.ttl`
  - `egp_racing.ttl`
  - `mars_market.ttl`
- **`queries/`**: Contains four SPARQL query files (`.rq`):
  - `extract_assembly_deltas.rq`
  - `extract_authority_deltas.rq`
  - `extract_receipt_deltas.rq`
  - `substrate.rq`

---

### B. Ontology Files Detailed Structure & Imports

#### 1. Core Ontology (`ontology/pack.ttl`)
- **Imports**: FIBO, SOSA, QUDT, PROV-O
  ```turtle
  eden:pack a owl:Ontology ;
      rdfs:label "Eden Manufacturing Server Ontology" ;
      owl:imports <https://spec.edmcouncil.org/fibo/ontology/> ,
                  <http://www.w3.org/ns/sosa/> ,
                  <http://qudt.org/schema/qudt/> ,
                  <http://www.w3.org/ns/prov#> .
  ```
- **Key Schema Classes**:
  - `eden:AssemblyComponent` (subclass of `fibo:Asset`, `fibo:Product`, `sosa:FeatureOfInterest`, `prov:Entity`)
  - `eden:MechRoot`, `eden:SubAssembly`, `eden:Part`, and `eden:Socket` (subclass of `eden:AssemblyComponent`, `sosa:Platform`)
- **Key Object Properties**:
  - `eden:hasSocket` (domain: `eden:AssemblyComponent`, range: `eden:Socket`)
  - `eden:plugsInto` (domain: `eden:AssemblyComponent`, range: `eden:Socket`)
- **Key Telemetry Datatype Properties (Byte-Class Authority, range: `xsd:unsignedByte`)**:
  - `eden:damageClass` (Dimensionless Ratio, mapped to `eden:damageProperty`)
  - `eden:stressClass` (Stress Quantity, mapped to `eden:stressProperty`)
  - `eden:heatClass` (Temperature Quantity, mapped to `eden:heatProperty`)
  - `eden:fatigueClass` (Dimensionless Ratio, mapped to `eden:fatigueProperty`)

#### 2. Deltas Ontology (`ontology/deltas.ttl`)
- **Imports**: PROV-O (`http://www.w3.org/ns/prov#`)
- **Key Schema Classes**:
  - `eden:Delta` (subclass of `prov:Entity` representing state modification artifacts)
  - `eden:AuthorityDelta` (subclass of `eden:Delta`)
  - `eden:AssemblyDelta` (subclass of `eden:Delta`)
  - `eden:ProjectionDelta` (subclass of `eden:Delta`)
  - `eden:InterestDelta` (subclass of `eden:Delta`)
  - `eden:ReceiptDelta` (subclass of `eden:Delta` capturing cryptographic and Playwright validation verdicts)
- **Key Properties**:
  - `eden:deltaId` (`xsd:string`), `eden:sequenceNumber` (`xsd:unsignedLong`), `eden:timestamp` (subproperty of `prov:generatedAtTime`), `eden:targetGraph` (`xsd:anyURI`), `eden:deltaPayload` (`xsd:string`)
  - `eden:authorizedBy` (subproperty of `prov:wasAttributedTo`), `eden:appliesTo` (range: `eden:AssemblyComponent`), `eden:receiptFor` (range: `prov:Activity`)
- **Receipt Delta Specific Datatypes**:
  - `eden:prompt` (`xsd:string`), `eden:contractHash` (`xsd:string`), `eden:buildLog` (`xsd:string`), `eden:packagePath` (`xsd:string`), `eden:baselineScreenshot` (`xsd:anyURI`), `eden:afterScreenshot` (`xsd:anyURI`), `eden:consoleLogs` (`xsd:string`), `eden:inputTrace` (`xsd:string`), `eden:visualDelta` (`xsd:float`), `eden:verdict` (`xsd:boolean`)

#### 3. GMF TPS & Bandai Manufacturing Ontology (`ontology/bandai_tps.ttl`)
- **Imports**: `eden-server` (`https://ggen.io/ontology/eden-server/`), SOSA, QUDT
- **Key Materials Taxonomies (disjoint polymers)**:
  - `tps:ManufacturingMaterial` with disjoint subclasses: `tps:PS`, `tps:ABS`, `tps:KPS`, `tps:PE`
  - Functional property: `tps:hasMaterial` maps an `eden:AssemblyComponent` to exactly one `tps:ManufacturingMaterial`
- **Jidoka Quality Gate**:
  - Defined via owl:equivalentClass:
    ```turtle
    tps:JidokaGate a owl:Class ;
        rdfs:subClassOf sosa:Actuator ;
        owl:equivalentClass [
            a owl:Restriction ;
            owl:onProperty sosa:hosts ;
            owl:someValuesFrom tps:FoundryProcess
        ] .
    ```
- **Scale and Origin Constraints (Gunpla Grade Disjointness)**:
  - Subclasses of `tps:ScaleGrade`: `tps:GradeHG`, `tps:GradeRG`, `tps:GradeMG`, `tps:GradePG`
  - Cardinality Constraint on `eden:MechRoot`:
    ```turtle
    eden:MechRoot rdfs:subClassOf [
        a owl:Restriction ;
        owl:onProperty tps:hasGrade ;
        owl:cardinality "1"^^xsd:nonNegativeInteger
    ] .
    ```

#### 4. Eden Grand Prix Racing Ontology (`ontology/egp_racing.ttl`)
- **Imports**: `eden-server` (`https://ggen.io/ontology/eden-server/`)
- **Key Classes**:
  - `egp:VehicleRoot` (subclass of `eden:AssemblyComponent`)
  - Mutual disjoint subclasses: `egp:Tire`, `egp:Engine`, and `egp:Chassis` (subclasses of `eden:Part`)
- **Telemetry Classes (Byte-Class Authority, Functional, 0-255)**:
  - `egp:gripClass` (domain: `egp:Tire`, range: `xsd:unsignedByte`)
  - `egp:heatClass` (domain: `egp:Engine`, range: `xsd:unsignedByte`)
- **Pit Strategy Events**:
  - `egp:PitStrategy` (OCEL-compatible assembly delta request) and `egp:SectorTime` (PROV-O activity backed by receipt)

#### 5. Mars Dimensional Marketplace Ontology (`ontology/mars_market.ttl`)
- **Imports**: `eden-server` (`https://ggen.io/ontology/eden-server/`), FIBO
- **Key Classes**:
  - `mars:DimensionalAsset` (subclass of `fibo:Asset` and `eden:AssemblyComponent`)
  - `mars:OwnershipRecord` (subclass of `fibo:Ownership` binding PROV-O Agent to asset via BLAKE3 receipt delta)
- **Functional Datatypes**:
  - `mars:riskClass` (unsignedByte telemetry history) and `mars:proofClass` (unsignedByte cryptographic receipt chain integrity)
  - Existential validation rule on `mars:DimensionalAsset`:
    ```turtle
    mars:DimensionalAsset rdfs:subClassOf [
        a owl:Restriction ;
        owl:onProperty mars:proofClass ;
        owl:someValuesFrom xsd:unsignedByte
    ] .
    ```

---

### C. SPARQL Query Files Detailed Structure

1. **`extract_assembly_deltas.rq`**:
   Extracts physical changes to the assembly tree (target sockets, component mounts/unmounts, sequence/actor provenance).
   - Order: `ORDER BY DESC(?timestamp) ?delta`
2. **`extract_authority_deltas.rq`**:
   Extracts authority updates on component telemetry classes (damage, stress, heat, fatigue classes, issuer).
   - Order: `ORDER BY DESC(?timestamp) ?delta`
3. **`extract_receipt_deltas.rq`**:
   Extracts cryptographic and Playwright validation deltas (baseline/after screenshots, visual delta, verdict, logs).
   - Order: `ORDER BY DESC(?timestamp) ?delta`
4. **`substrate.rq`**:
   Performs a property path traversal of the entire tree (`(eden:hasSocket/^eden:plugsInto)*`) starting at `eden:MechRoot` down to parents, sockets, and child parts, fetching their respective byte-class telemetry states.
   - Order: `ORDER BY ?root ?parent ?socket ?child`

---

### D. Configuration (`ggen.toml`)
- No `ggen.toml` file exists directly in `/Users/sac/.ggen/packs/eden_server/`. 
- By looking at other packs (such as `sosa_pack/ggen.toml` and `ue4_ontology/ggen.toml`), we observe that `ggen.toml` is used to declare project metadata, primary schema files, imported packages, inference rules (`CONSTRUCT` queries), code generation rules (SPARQL + Tera templates), and SHACL validation files/rules.

---

## 2. Logic Chain

1. **Absence of `ggen.toml`**: Since `/Users/sac/.ggen/packs/eden_server/` contains only raw `.ttl` and `.rq` files and lacks a `ggen.toml` configuration, the pack operates as a raw schema/query provider rather than a standalone `ggen` workspace project. It relies on parent packages/engines (like `nexus-engine` or `rocket-craft`) to import its TTL schemas into their compilation pipeline.
2. **Import Hierarchy**:
   - `pack.ttl` is the root core. It imports standard external ontologies: FIBO, SOSA, QUDT, and PROV-O.
   - `deltas.ttl` imports PROV-O directly.
   - Domain-specific extensions (`bandai_tps.ttl`, `egp_racing.ttl`, `mars_market.ttl`) import `https://ggen.io/ontology/eden-server/` (the core namespaces defined by `pack.ttl`).
   - Additionally, `bandai_tps.ttl` imports SOSA and QUDT. `mars_market.ttl` imports FIBO.
3. **Enforcement of Typestate/Validation Rules**:
   - Instead of SHACL shapes, validation rules in this pack are embedded as OWL 2 DL logical axioms.
   - High-stress sockets/components are restricted by polymer disjointness axioms (e.g., disjointness among `tps:PS`, `tps:ABS`, `tps:KPS`, `tps:PE`).
   - The functional nature (`owl:FunctionalProperty`) of `tps:hasMaterial`, `egp:gripClass`, `egp:heatClass`, `mars:riskClass`, and `mars:proofClass` guarantees that an asset/component has at most one material, grade, grip, heat, risk, or proof state.
   - Cardinality constraints (such as `owl:cardinality "1"` for `tps:hasGrade` on `eden:MechRoot`) ensure that a mechanical root is properly registered under exactly one Scale Grade.
4. **SPARQL Projection Paths**:
   - The queries in `queries/` act as the data extraction projection layer.
   - Specifically, `substrate.rq` uses the property path `(eden:hasSocket/^eden:plugsInto)*` to traverse structural tree parent-child layers dynamically, allowing a generator tool to reconstruct the server's byte-class state in a flat array structure.

---

## 3. Caveats

- **No Standalone `ggen.toml`**: Because there is no `ggen.toml` configuration inside the `eden_server` folder, we assume that any compilation or generation utilizing this pack is orchestrated by an external workspace or CLI command referencing these TTL files directly.
- **In-Memory/Logical Validation Only**: The validation rules observed are purely declarative logical restrictions within the OWL Turtle documents. They require a semantic reasoner (such as Pellet, HermiT, or standard OWL RL rule expansion) to enforce consistency. There are no SHACL files present in this specific folder to run target-driven structural syntax checks.

---

## 4. Conclusion

The `eden_server` pack provides a complete domain model for the TPS/DfLSS Playwright Manufacturing Strategy, organizing the server into:
1. An **Assembly Tree Topology** mapped to SOSA Platforms and Parts.
2. **Reliability Twin properties** representing physical attributes (damage, stress, heat, fatigue) as functional `xsd:unsignedByte` (0-255) telemetry classes.
3. **Transaction Deltas** (including physical Assembly updates, telemetry Authority updates, and cryptographic/visual Receipt verification metrics matching Playwright test execution data).
4. **Disjoint taxonomies and restrictions** enforcing strict polymer/material type-safety, scales, and cryptographic proofs at the OWL reasoning level.

---

## 5. Verification Method

To verify these observations:
1. View files inside `/Users/sac/.ggen/packs/eden_server/` using standard read/view commands:
   - Check file existence: `ls -la /Users/sac/.ggen/packs/eden_server/ontology`
   - Check file contents: `cat /Users/sac/.ggen/packs/eden_server/ontology/pack.ttl`
2. Compare the import declarations in the `.ttl` files to ensure they match the dependencies documented above.
3. Validate the absence of `ggen.toml` in `/Users/sac/.ggen/packs/eden_server/` by performing a search command (e.g., `find /Users/sac/.ggen/packs/eden_server/ -name ggen.toml`).
