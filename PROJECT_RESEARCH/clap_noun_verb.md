# Research Dossier: `clap-noun-verb`

**Total Files:** 20 Ontologies (.ttl) | 11 Queries (.rq)
**Total Volume:** 31 files

## 1. Core Vocabularies (Prefixes)
- `api: <http://example.org/user-api#>`
- `arch: <http://example.org/architecture/>`
- `bible: <http://purl.org/ontology/bible/>`
- `bibo: <http://purl.org/ontology/bibo/>`
- `bos: <http://builditout.systems/nehemiah-operating-grammar/>`
- `calc: <http://example.org/calculator#>`
- `cicd: <http://cargo-cicd.io/ontology#>`
- `clap: <http://clap-noun-verb.io/ontology#>`
- `clap: <http://clap-noun-verb.org/capability/>`
- `cli: <http://clap-noun-verb.io/ontology#>`
- `cnv: <http://clap-noun-verb.io/ontology#>`
- `cnv: <http://example.org/clap-noun-verb/>`
- `dcat: <http://www.w3.org/ns/dcat#>`
- `dct: <http://purl.org/dc/terms/>`
- `dcterms: <http://purl.org/dc/terms/>`
- `ex: <http://clap-noun-verb.io/examples#>`
- `ex: <http://clap-noun-verb.io/examples/greet-demo#>`
- `fm: <http://example.org/file-manager#>`
- `foaf: <http://xmlns.com/foaf/0.1/>`
- `ggen: <https://chatmangpt.com/ontologies/ggen/command-vocab#>`
- `htf: <http://thesis.hyper/framework/>`
- `ne3: <http://builditout.systems/nehemiah-3/>`
- `ne6: <http://builditout.systems/nehemiah-6/>`
- `nehemiah: <http://example.org/nehemiah/>`
- `owl: <http://www.w3.org/2002/07/owl#>`
- `prov: <http://www.w3.org/ns/prov#>`
- `rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>`
- `rdfs: <http://www.w3.org/2000/01/rdf-schema#>`
- `skos: <http://www.w3.org/2004/02/skos/core#>`
- `spec: <http://cargo-cicd.io/spec#>`
- `spec: <http://clap-noun-verb.io/spec#>`
- `srv: <http://example.org/web-server#>`
- `vt: <http://clap-noun-verb.io/verb-traits#>`
- `wiz: <http://clap-noun-verb.io/wizard#>`
- `xsd: <http://www.w3.org/2001/XMLSchema#>`

## 2. Domain Taxonomy & Entities
### Base Classes & Shapes
- `:BackupVerb`
- `:DatabaseCLI`
- `:DatabaseVerb`
- `:MigrateVerb`
- `:StatusVerb`
- `:backup_output`
- `:migrate_output`
- `:status_output`
- `arch:ArchitectureBoard`
- `arch:ArchitectureBuildingBlock`
- `arch:ArchitectureContract`
- `arch:SolutionBuildingBlock`
- `arch:TOGAFPhase`
- `bible:AuthorialSource`
- `bible:Chapter`
- `bible:Gospel`
- `bible:Pericope`
- `bible:Verse`
- `bos:Builder`
- `bos:Courier`
- `bos:CourierRecord`
- `bos:FalseGate`
- `bos:FalseReport`
- `bos:Gate`
- `bos:GateSwarm`
- `bos:InspectionGate`
- `bos:Mocker`
- `bos:MusterLedger`
- `bos:Prayer`
- `bos:ProphetOffice`
- `bos:Receipt`
- `bos:UsuryLedger`
- `bos:VerdictALIVE`
- `bos:VerdictBLOCKED`
- `bos:VerdictPARTIAL`
- `bos:WallSection`
- `cicd:CoverageReport`
- `cicd:DependencyGraph`
- `cicd:DiagnosticReport`
- `cicd:PublishResults`
- `cicd:TestResults`
- `cnv:Argument`
- `cnv:ArgumentType`
- `cnv:CliBuilder`
- `cnv:CliElement`
- `cnv:Command`
- `cnv:CommandRegistry`
- `cnv:CommandRouter`
- `cnv:ComposableCommand`
- `cnv:CompoundNoun`
- *...and 55 more.*

## 3. Extraction Layer (SPARQL)
- **Query Types Executed:** {'SELECT': 7, 'ASK': 2, 'CONSTRUCT': 2}

### Projected Variables (SELECT ?var)
This project actively projects the following variables into code/templates:
> `?affectedResource`, `?argument`, `?argumentAbout`, `?argumentName`, `?argumentTypeLabel`, `?defaultValue`, `?description`, `?full_command_name`, `?handlerSignature`, `?handler_name`, `?is_required`, `?issue`, `?nounName`, `?noun_description`, `?noun_name`, `?output_format`, `?required`, `?resource`, `?resourceName`, `?resourceType`, `?returnType`, `?return_type`, `?shortName`, `?traitLabel`, `?traitRequirement`, `?valueType`, `?verb`, `?verbAbout`, `?verbName`, `?verb_about`, `?verb_description`, `?verb_name`

## 4. File Inventory
<details>
<summary>Click to expand all files</summary>

- `/Users/sac/clap-noun-verb/archive/examples-playground/thesis-ontology.ttl` (11061 bytes)
- `/Users/sac/clap-noun-verb/archive/playground/thesis-ontology.ttl` (18283 bytes)
- `/Users/sac/clap-noun-verb/docs/_internal/clap-capabilities.ttl` (18224 bytes)
- `/Users/sac/clap-noun-verb/docs/abb_governance_ontology.ttl` (33203 bytes)
- `/Users/sac/clap-noun-verb/examples/greet-demo/ontology.ttl` (2289 bytes)
- `/Users/sac/clap-noun-verb/examples/specs/database-cli.ttl` (1130 bytes)
- `/Users/sac/clap-noun-verb/examples/turtle-specs/calculator.ttl` (3041 bytes)
- `/Users/sac/clap-noun-verb/examples/turtle-specs/file-manager.ttl` (5769 bytes)
- `/Users/sac/clap-noun-verb/examples/turtle-specs/user-api.ttl` (11576 bytes)
- `/Users/sac/clap-noun-verb/examples/turtle-specs/web-server.ttl` (11082 bytes)
- `/Users/sac/clap-noun-verb/examples/wizard-specs.ttl` (8753 bytes)
- `/Users/sac/clap-noun-verb/ontology/cargo-cicd.ttl` (10838 bytes)
- `/Users/sac/clap-noun-verb/ontology/clap-noun-verb-ontology.ttl` (19693 bytes)
- `/Users/sac/clap-noun-verb/ontology/cli-pattern.ttl` (16624 bytes)
- `/Users/sac/clap-noun-verb/ontology/cli_schema.ttl` (1728 bytes)
- `/Users/sac/clap-noun-verb/ontology/gospel-passage-pattern.ttl` (8787 bytes)
- `/Users/sac/clap-noun-verb/ontology/nehemiah-operating-grammar.ttl` (19209 bytes)
- `/Users/sac/clap-noun-verb/ontology/oshb-morphology-source.ttl` (7916 bytes)
- `/Users/sac/clap-noun-verb/ontology/oshb-reference.ttl` (2065 bytes)
- `/Users/sac/clap-noun-verb/ontology/queries/conformance-check.rq` (2078 bytes)
- `/Users/sac/clap-noun-verb/ontology/queries/extract-command-specs.rq` (2215 bytes)
- `/Users/sac/clap-noun-verb/ontology/queries/fieldname-collision.rq` (2448 bytes)
- `/Users/sac/clap-noun-verb/ontology/verb-traits.ttl` (22343 bytes)
- `/Users/sac/clap-noun-verb/queries/cargo-cicd-commands.rq` (1891 bytes)
- `/Users/sac/clap-noun-verb/queries/extract-arguments.rq` (1348 bytes)
- `/Users/sac/clap-noun-verb/queries/find-all-verbs.rq` (937 bytes)
- `/Users/sac/clap-noun-verb/queries/generate-cli-spec.rq` (1361 bytes)
- `/Users/sac/clap-noun-verb/queries/generate-trait-impls.rq` (1758 bytes)
- `/Users/sac/clap-noun-verb/queries/validate-cli-structure.rq` (3012 bytes)
- `/Users/sac/clap-noun-verb/queries/verb-signatures.rq` (3388 bytes)
- `/Users/sac/clap-noun-verb/queries/verbs-mod.rq` (609 bytes)

</details>
