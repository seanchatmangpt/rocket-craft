# Research Dossier: `sos`

**Total Files:** 3 Ontologies (.ttl) | 3 Queries (.rq)
**Total Volume:** 6 files

## 1. Core Vocabularies (Prefixes)
- `ex: <http://example.org/>`
- `owl: <http://www.w3.org/2002/07/owl#>`
- `rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>`
- `rdfs: <http://www.w3.org/2000/01/rdf-schema#>`
- `schema: <https://schema.org/>`
- `sos: <https://ggen.io/semantic-os/>`

## 2. Domain Taxonomy & Entities
### Base Classes & Shapes
- `ex:TestClass`
- `schema:Person`
- `sos:ArchitectureLevel`
- `sos:Artifact`
- `sos:CompoundMachine`
- `sos:Crate`
- `sos:ExitField`
- `sos:Field8`
- `sos:Law`
- `sos:LifecycleMachine`
- `sos:OperatingDiscipline`
- `sos:OperationalDomain`
- `sos:Playbook`
- `sos:Receipt`
- `sos:Recipe`
- `sos:Replay`
- `sos:RootField`
- `sos:Runbook`

## 3. Extraction Layer (SPARQL)
- **Query Types Executed:** {'SELECT': 2, 'CONSTRUCT': 1}

### Projected Variables (SELECT ?var)
This project actively projects the following variables into code/templates:
> `?condition_name`, `?condition_type`, `?description`, `?field_type`, `?input_value`, `?law_id`, `?name`

## 4. File Inventory
<details>
<summary>Click to expand all files</summary>

- `/Users/sac/sos/diagnostic_test.ttl` (173 bytes)
- `/Users/sac/sos/queries/semantic_os/construct-laws.rq` (296 bytes)
- `/Users/sac/sos/queries/semantic_os/extract-laws.rq` (767 bytes)
- `/Users/sac/sos/queries/semantic_os/extract-transitions.rq` (349 bytes)
- `/Users/sac/sos/schema/domain.ttl` (6587 bytes)
- `/Users/sac/sos/test_init/schema/domain.ttl` (1559 bytes)

</details>
