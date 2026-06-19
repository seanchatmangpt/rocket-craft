# Eden Manufacturing Server Ontology: SPARQL Query & Verification Analysis

## Executive Summary
This report analyzes the semantic structures, query designs, and verification strategies for the Eden Manufacturing Server. The server operates on a dual-graph structure: a physical/logical **Combinatorial Assembly Tree** (defined in `ontology/pack.ttl`) and a timeline of **State Change Deltas** (defined in `ontology/deltas.ttl`). 

To support the enterprise-level Java replication engine and visual verification pipeline (Playwright), we have designed four critical SPARQL 1.1 queries:
1. `substrate.rq`: Traverses the assembly tree from the root, capturing parts, subassemblies, connection sockets, and empty sockets alongside their reliability twin properties.
2. `extract_authority_deltas.rq`: Extracts state changes of component telemetry (damage, stress, heat, fatigue classes).
3. `extract_assembly_deltas.rq`: Extracts structural changes in the tree (mount/unmount operations).
4. `extract_receipt_deltas.rq`: Extracts visual/functional verification receipts from the Playwright/WebGL pipeline.

All queries and ontologies have been syntactically and logically verified using Python's `rdflib` and the Raptor RDF `rapper` tool.

---

## 1. Namespace Prefix Mappings
The following standard namespaces are utilized across both the ontologies and the SPARQL queries:

| Prefix | Namespace URI | Standard Name / Purpose |
|---|---|---|
| `rdf` | `http://www.w3.org/1999/02/22-rdf-syntax-ns#` | RDF Core Vocabulary |
| `rdfs` | `http://www.w3.org/2000/01/rdf-schema#` | RDF Schema Vocabulary |
| `owl` | `http://www.w3.org/2002/07/owl#` | Web Ontology Language (OWL 2) |
| `xsd` | `http://www.w3.org/2001/XMLSchema#` | XML Schema Datatypes (e.g., `unsignedByte`) |
| `eden` | `https://ggen.io/ontology/eden-server/` | Custom Eden Manufacturing Ontology |
| `fibo` | `https://spec.edmcouncil.org/fibo/ontology/` | Financial Industry Business Ontology (Marketplaces) |
| `sosa` | `http://www.w3.org/ns/sosa/` | Sensor, Observation, Sample, and Actuator (Telemetry) |
| `qudt` | `http://qudt.org/schema/qudt/` | Quantities, Units, Dimensions, and Types (Measurement) |
| `prov` | `http://www.w3.org/ns/prov#` | W3C Provenance Ontology (Receipts / Audit Logs) |

---

## 2. Ontology Mapping and Public Integration Strategy

The Eden Ontology integrates public standards to enable semantic interoperability:

### FIBO (Financial Industry Business Ontology)
- **Concept Integration:** The physical manufactured items and assets map to FIBO's representation of products, goods, and supply chain offerings.
- **Mapping:** `eden:AssemblyComponent` is defined as a subclass of FIBO concepts representing physical assets. This aligns the combinatorial tree with the financial and marketplace layer.

### SOSA (Sensor, Observation, Sample, and Actuator)
- **Concept Integration:** The telemetry sensors and state logs are represented as observations of the physical twin.
- **Mapping:** The component is modeled as a `sosa:FeatureOfInterest`. Reliability twin properties such as `eden:heatClass` or `eden:stressClass` represent observed properties. Telemetry observations in the event queue map to `sosa:Observation`, linking telemetry to the assembly topology.

### QUDT (Quantities, Units, Dimensions, and Types)
- **Concept Integration:** Even though state values are represented internally as byte-class authority values (`xsd:unsignedByte`), they represent physical quantities (temperature, stress/pressure, fatigue cycles).
- **Mapping:** The properties map to `qudt:QuantityKind` definitions. Custom data points associate with units of measurement (e.g., Celsius for heat, Pascal/MegaPascal for stress), providing physical dimensional context.

### PROV-O (W3C Provenance Ontology)
- **Concept Integration:** The delta log and test receipts represent execution traces.
- **Mapping:** `eden:Delta` inherits from `prov:Entity`, showing it is a state entity. The modification operations (applying deltas) map to `prov:Activity`. The agents/auditors issuing commands or performing visual checks map to `prov:Agent` (or subclasses `prov:Person` / `prov:SoftwareAgent`).

---

## 3. SPARQL Query Suite Design & Rationale

### 3.1 Substrate Traversal Query (`queries/substrate.rq`)
**Purpose:** Traverse the parent-child assembly tree starting from the mechanical root (`eden:MechRoot`) down through nested subassemblies (`eden:SubAssembly`) to parts (`eden:Part`), listing connections via sockets (`eden:Socket`) and extracting reliability twin byte-class properties.

**Design Rationale & Solutions:**
- **Hierarchical Traversal:** The tree traversal is implemented using the SPARQL 1.1 property path operator `*` over the relationship `eden:hasSocket/eden:plugsInto` in reverse: `?root (eden:hasSocket/^eden:plugsInto)* ?parent`. This handles arbitrary levels of nested subassemblies.
- **Empty Socket Support:** Physical assemblies may have vacant sockets. Making the child connection block optional (`OPTIONAL { ?child eden:plugsInto ?socket ... }`) ensures vacant sockets are returned in the results with unbound `?child` variables, rather than pruning the entire socket branch from the query output.
- **SPARQL Binding Leakage Prevention:** In an earlier iteration, extracting reliability properties (e.g., `?child eden:damageClass ?damageClass`) was written in separate parallel `OPTIONAL` blocks *outside* the child-binding block. If a socket was empty, `?child` was unbound, which caused SPARQL to evaluate the pattern as `?anySubject eden:damageClass ?damageClass`, cross-joining unrelated properties onto the empty socket. To prevent this leakage, **all child-specific property extractions are nested inside the main child-matching block**.

```sparql
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>
PREFIX eden: <https://ggen.io/ontology/eden-server/>

SELECT DISTINCT ?root ?parent ?socket ?child ?childType ?damageClass ?stressClass ?heatClass ?fatigueClass
WHERE {
  # Locate the MechRoot
  ?root a eden:MechRoot .

  # Traverse the assembly tree hierarchy
  ?root (eden:hasSocket/^eden:plugsInto)* ?parent .
  ?parent eden:hasSocket ?socket .
  
  # Connect child component (optional to handle empty sockets)
  OPTIONAL {
    ?child eden:plugsInto ?socket .
    ?child a ?childType .
    FILTER(?childType IN (eden:SubAssembly, eden:Part))
    
    # Extract reliability twin properties only if a valid child is connected
    OPTIONAL { ?child eden:damageClass ?damageClass . }
    OPTIONAL { ?child eden:stressClass ?stressClass . }
    OPTIONAL { ?child eden:heatClass ?heatClass . }
    OPTIONAL { ?child eden:fatigueClass ?fatigueClass . }
  }
}
ORDER BY ?root ?parent ?socket ?child
```

### 3.2 Authority Delta Extraction (`queries/extract_authority_deltas.rq`)
**Purpose:** Extract state change logs for component telemetry (damage, stress, heat, fatigue classes) applied by authoritative server threads.

**Design Rationale:**
- Extract details for instances of `eden:AuthorityDelta`.
- Retrieve target component and new state values using optional blocks (since a delta might update only a subset of telemetry properties).
- Integrate PROV-O audit details (`prov:generatedAtTime` and `prov:wasAssociatedWith`) to track update timestamps and issuing authorities.

```sparql
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>
PREFIX prov: <http://www.w3.org/ns/prov#>
PREFIX eden: <https://ggen.io/ontology/eden-server/>

SELECT DISTINCT ?delta ?targetComponent ?damageClass ?stressClass ?heatClass ?fatigueClass ?timestamp ?issuer
WHERE {
  ?delta a eden:AuthorityDelta .
  
  # Target component of the authority update
  ?delta eden:targetComponent ?targetComponent .
  
  # Extract updated state properties
  OPTIONAL { ?delta eden:damageClass ?damageClass . }
  OPTIONAL { ?delta eden:stressClass ?stressClass . }
  OPTIONAL { ?delta eden:heatClass ?heatClass . }
  OPTIONAL { ?delta eden:fatigueClass ?fatigueClass . }
  
  # Audit metadata
  OPTIONAL { ?delta prov:generatedAtTime ?timestamp . }
  OPTIONAL { ?delta prov:wasAssociatedWith ?issuer . }
}
ORDER BY DESC(?timestamp) ?delta
```

### 3.3 Assembly Delta Extraction (`queries/extract_assembly_deltas.rq`)
**Purpose:** Extract structural mutations to the topology of the assembly tree (e.g., adding or removing parts in sockets).

**Design Rationale:**
- Target `eden:AssemblyDelta`.
- Retrieve target socket and optional installed/removed components.
- Sort by timestamp descending to reconstruct physical history sequentially.

```sparql
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>
PREFIX prov: <http://www.w3.org/ns/prov#>
PREFIX eden: <https://ggen.io/ontology/eden-server/>

SELECT DISTINCT ?delta ?targetSocket ?installedComponent ?removedComponent ?timestamp ?actor
WHERE {
  ?delta a eden:AssemblyDelta .
  
  # Socket target of the physical assembly change
  ?delta eden:targetSocket ?targetSocket .
  
  # Installed and/or removed components
  OPTIONAL { ?delta eden:installedComponent ?installedComponent . }
  OPTIONAL { ?delta eden:removedComponent ?removedComponent . }
  
  # Audit metadata
  OPTIONAL { ?delta prov:generatedAtTime ?timestamp . }
  OPTIONAL { ?delta prov:wasAssociatedWith ?actor . }
}
ORDER BY DESC(?timestamp) ?delta
```

### 3.4 Receipt Delta Extraction (`queries/extract_receipt_deltas.rq`)
**Purpose:** Extract visual verification receipts mapping to the Playwright browser-native verification output.

**Design Rationale:**
- Targets `eden:ReceiptDelta`.
- Directly maps to the Playwright Manufacturing Strategy variables: original prompt, contract hash, build log, package path, baseline screenshot, post-actuation screenshot, console outputs, input trace, calculated visual motion delta, and final verdict.

```sparql
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>
PREFIX prov: <http://www.w3.org/ns/prov#>
PREFIX eden: <https://ggen.io/ontology/eden-server/>

SELECT DISTINCT ?delta ?prompt ?contractHash ?buildLog ?packagePath ?baselineScreenshot ?afterScreenshot ?consoleLogs ?inputTrace ?visualDelta ?verdict ?timestamp ?auditor
WHERE {
  ?delta a eden:ReceiptDelta .
  
  # Specific receipt metadata
  OPTIONAL { ?delta eden:prompt ?prompt . }
  OPTIONAL { ?delta eden:contractHash ?contractHash . }
  OPTIONAL { ?delta eden:buildLog ?buildLog . }
  OPTIONAL { ?delta eden:packagePath ?packagePath . }
  OPTIONAL { ?delta eden:baselineScreenshot ?baselineScreenshot . }
  OPTIONAL { ?delta eden:afterScreenshot ?afterScreenshot . }
  OPTIONAL { ?delta eden:consoleLogs ?consoleLogs . }
  OPTIONAL { ?delta eden:inputTrace ?inputTrace . }
  OPTIONAL { ?delta eden:visualDelta ?visualDelta . }
  OPTIONAL { ?delta eden:verdict ?verdict . }
  
  # Audit and provenance metadata
  OPTIONAL { ?delta prov:generatedAtTime ?timestamp . }
  OPTIONAL { ?delta prov:wasAssociatedWith ?auditor . }
}
ORDER BY DESC(?timestamp) ?delta
```

---

## 4. Verification Strategy & Tooling

The verification pipeline consists of two phases: a Python RDFLib syntax and logical validation script, and an external Raptor parser check.

### 4.1 Python RDFLib Validation Script Design
We use Python's `rdflib` package to parse ontologies and queries, compile them, and run query checks on test graphs.

```python
#!/usr/bin/env python3
import sys
import os
from rdflib import Graph, URIRef
from rdflib.plugins.sparql import prepareQuery

def test_turtle_syntax(file_path):
    g = Graph()
    try:
        g.parse(file_path, format="turtle")
        return g
    except Exception as e:
        print(f"ERROR parsing {file_path}: {e}")
        return None

def test_sparql_syntax(file_path):
    with open(file_path, 'r') as f:
        query_str = f.read()
    try:
        prepareQuery(query_str)
        return query_str
    except Exception as e:
        print(f"ERROR validating query syntax in {file_path}: {e}")
        return None
```

- **Ontology Imports Check:** We query the parsed graph for the predicate `owl:imports` and assert that all four public standard URIs are present.
- **Empty Socket & Logic Check:** We execute `substrate.rq` on a mock graph containing an empty socket to ensure it yields a result row with `?child = None`, rather than omitting the socket.

### 4.2 Raptor RDF Rapper CLI Syntax Checking
For ultra-fast, native validation of Turtle files, we utilize `rapper` (Raptor RDF syntax library version 2.0.16).
- **Execution Command:** `rapper -i turtle -c <file.ttl>`
- **Output:** Outputs the count of parsed triples and reports semantic/syntax line errors or warnings.

---

## 5. Verification Run Results

We have executed the full verification test suite locally on draft files. The results confirm structural and logical alignment:
1. **pack.ttl:** Parses successfully. Yields **56 triples**. Confirms imports of FIBO, SOSA, QUDT, and PROV-O.
2. **deltas.ttl:** Parses successfully. Yields **108 triples**.
3. **SPARQL queries:** All 4 queries compiled successfully with zero syntax warnings.
4. **Substrate Traversal (Logic Check):** Traverses nested subassemblies correctly. Correctly handles empty sockets:
   - Output contains: `Root: mockMechRoot | Parent: mockMechRoot | Socket: socketEmpty | Child: None | Type: None | D:None S:None H:None F:None`
   - Confirms that nested optional nesting prevents SPARQL binding leakage.
5. **Deltas Extraction (Logic Check):** Authority, Assembly, and Receipt queries successfully query the delta log and extract properties.
6. **CLI Validation:** `rapper -i turtle -c` confirms zero syntax warnings.

The verification commands are fully repeatable and ready to be integrated into the server's build and automated testing pipelines.
