# Research Dossier: `bitactor`

**Total Files:** 52 Ontologies (.ttl) | 0 Queries (.rq)
**Total Volume:** 52 files

## 1. Core Vocabularies (Prefixes)
- `(case-insensitive)
PREFIX foaf: <http://xmlns.com/foaf/0.1/>`
- `bitactor: <http://bitactor.org/ontology#>`
- `cns: <http://cns-forge.org/ontology#>`
- `cns: <http://cns.ai/ontology#>`
- `cns: <http://cns.autotel.org/ontology#>`
- `cns: <https://schema.chatman.ai/cns#>`
- `dflss: <http://cns.autotel.org/ontology/dflss#>`
- `ex1: <http://example1.org/>`
- `ex2: <http://example2.org/>`
- `ex: <http://example.org/>`
- `foaf: <http://xmlns.com/foaf/0.1/>`
- `gate: <https://schema.chatman.ai/gatekeeper#>`
- `ns1: <http://namespace1.org/>`
- `ns2: <http://namespace2.org/>`
- `owl: <http://www.w3.org/2002/07/owl#>`
- `perf: <http://cns.autotel.org/ontology/performance#>`
- `perf: <https://schema.chatman.ai/performance#>`
- `rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>`
- `rdfs: <http://www.w3.org/2000/01/rdf-schema#>`
- `sevenT: <https://schema.chatman.ai/seven-t#>`
- `sh: <http://www.w3.org/ns/shacl#>`
- `telemetry: <http://cns.autotel.org/ontology/telemetry#>`
- `test: <https://schema.chatman.ai/testing#>`
- `time: <http://www.w3.org/2006/time#>`
- `unit: <http://qudt.org/vocab/unit/>`
- `weaver: <https://schema.chatman.ai/weaver#>`
- `xsd: <http://www.w3.org/2001/XMLSchema#>`

## 2. Domain Taxonomy & Entities
### Base Classes & Shapes
- `:Actor`
- `:Behavior`
- `:Contract`
- `:Employee`
- `:Entanglement`
- `:IndividualContributor`
- `:Manager`
- `:Opcode`
- `:TelemetrySpan`
- `bitactor:ActorStatus`
- `bitactor:BitActor`
- `bitactor:Signal`
- `bitactor:SignalPriority`
- `bitactor:SignalType`
- `bitactor:SwarmConfiguration`
- `bitactor:SwarmTopology`
- `bitactor:TTLConstraint`
- `bitactor:TTLViolation`
- `bitactor:TelemetryFrame`
- `cns:7TickCompliant`
- `cns:ArchitecturalPattern`
- `cns:ArchitecturalPatternShape`
- `cns:ArchitectureLayer`
- `cns:ArchitectureLayerShape`
- `cns:Assertion`
- `cns:AssertionShape`
- `cns:Benchmark`
- `cns:BenchmarkShape`
- `cns:BenchmarkType`
- `cns:BuildSystem`
- `cns:CNSBenchmark`
- `cns:CNSComponent`
- `cns:CNSFile`
- `cns:CNSFunction`
- `cns:CNSReport`
- `cns:CodeGenerator`
- `cns:CognitivePattern`
- `cns:Command`
- `cns:CommandShape`
- `cns:Compiler`
- `cns:Component`
- `cns:ComponentInterface`
- `cns:ComponentInterfaceShape`
- `cns:Configuration`
- `cns:CycleMetric`
- `cns:DataFlow`
- `cns:DataFlowShape`
- `cns:Dependency`
- `cns:DeploymentModel`
- `cns:DeploymentModelShape`
- *...and 151 more.*

## 3. Extraction Layer (SPARQL)
- *No queries executed in this project.*

### Projected Variables (SELECT ?var)
- *No specific projection variables identified.*

## 4. File Inventory
<details>
<summary>Click to expand all files</summary>

- `/Users/sac/bitactor/Desktop/TAI_GAHI_PACKAGE/config/bitactor_ontology.ttl` (10909 bytes)
- `/Users/sac/bitactor/Desktop/TAI_GAHI_PACKAGE/config/sample_bitactor.ttl` (2081 bytes)
- `/Users/sac/bitactor/bitactor_core_ontology.ttl` (4591 bytes)
- `/Users/sac/bitactor/cns/business_rules.ttl` (2281 bytes)
- `/Users/sac/bitactor/cns/codegen/simple.ttl` (16 bytes)
- `/Users/sac/bitactor/cns/docs/ontology/cns-architecture.ttl` (20702 bytes)
- `/Users/sac/bitactor/cns/docs/ontology/cns-build.ttl` (14481 bytes)
- `/Users/sac/bitactor/cns/docs/ontology/cns-compilers.ttl` (16007 bytes)
- `/Users/sac/bitactor/cns/docs/ontology/cns-complete.ttl` (17361 bytes)
- `/Users/sac/bitactor/cns/docs/ontology/cns-core.ttl` (6679 bytes)
- `/Users/sac/bitactor/cns/docs/ontology/cns-domains.ttl` (15729 bytes)
- `/Users/sac/bitactor/cns/docs/ontology/cns-engines.ttl` (18769 bytes)
- `/Users/sac/bitactor/cns/docs/ontology/cns-enhanced.ttl` (33962 bytes)
- `/Users/sac/bitactor/cns/docs/ontology/cns-gatekeeper.ttl` (20553 bytes)
- `/Users/sac/bitactor/cns/docs/ontology/cns-master.ttl` (11099 bytes)
- `/Users/sac/bitactor/cns/docs/ontology/cns-performance.ttl` (13435 bytes)
- `/Users/sac/bitactor/cns/docs/ontology/cns-seven-t.ttl` (22800 bytes)
- `/Users/sac/bitactor/cns/docs/ontology/cns-telemetry.ttl` (10497 bytes)
- `/Users/sac/bitactor/cns/docs/ontology/cns-testing.ttl` (19958 bytes)
- `/Users/sac/bitactor/cns/docs/ontology/cns-tests.ttl` (13359 bytes)
- `/Users/sac/bitactor/cns/docs/ontology/cns-weaver.ttl` (17722 bytes)
- `/Users/sac/bitactor/cns/docs/ontology/dflss-ontology.ttl` (19920 bytes)
- `/Users/sac/bitactor/cns/src/binary_materializer/test_semantic.ttl` (2621 bytes)
- `/Users/sac/bitactor/cns/test_debug_simple.ttl` (63 bytes)
- `/Users/sac/bitactor/cns/test_ontology.ttl` (862 bytes)
- `/Users/sac/bitactor/cns/ttl-parser/complex_test.ttl` (42 bytes)
- `/Users/sac/bitactor/cns/ttl-parser/examples/grammar-examples.ttl` (7774 bytes)
- `/Users/sac/bitactor/cns/ttl-parser/examples/sample_query.ttl` (731 bytes)
- `/Users/sac/bitactor/cns/ttl-parser/examples/simple.ttl` (1147 bytes)
- `/Users/sac/bitactor/cns/ttl-parser/network_test.ttl` (92 bytes)
- `/Users/sac/bitactor/cns/ttl-parser/prefix_test.ttl` (53 bytes)
- `/Users/sac/bitactor/cns/ttl-parser/rich_test.ttl` (171 bytes)
- `/Users/sac/bitactor/cns/ttl-parser/simple_test.ttl` (71 bytes)
- `/Users/sac/bitactor/cns/ttl-parser/test-minimal.ttl` (62 bytes)
- `/Users/sac/bitactor/cns/ttl-parser/test.ttl` (194 bytes)
- `/Users/sac/bitactor/cns/ttl-parser/test_complex.ttl` (199 bytes)
- `/Users/sac/bitactor/cns/ttl-parser/test_debug.ttl` (27 bytes)
- `/Users/sac/bitactor/cns/ttl-parser/test_debug_simple.ttl` (63 bytes)
- `/Users/sac/bitactor/cns/ttl-parser/test_minimal.ttl` (63 bytes)
- `/Users/sac/bitactor/cns/ttl-parser/test_query.ttl` (54 bytes)
- `/Users/sac/bitactor/cns/ttl-parser/test_query_fixed.ttl` (197 bytes)
- `/Users/sac/bitactor/cns/ttl-parser/test_query_prefix.ttl` (135 bytes)
- `/Users/sac/bitactor/cns/ttl-parser/test_sample.ttl` (283 bytes)
- `/Users/sac/bitactor/cns/ttl-parser/test_sample2.ttl` (437 bytes)
- `/Users/sac/bitactor/cns/ttl-parser/test_simple.ttl` (171 bytes)
- `/Users/sac/bitactor/cns/ttl-parser/test_simple_query.ttl` (29 bytes)
- `/Users/sac/bitactor/cns/ttl-parser/tests/fixtures/complex.ttl` (1766 bytes)
- `/Users/sac/bitactor/cns/ttl-parser/tests/fixtures/edge_cases.ttl` (2219 bytes)
- `/Users/sac/bitactor/cns/ttl-parser/tests/fixtures/invalid.ttl` (637 bytes)
- `/Users/sac/bitactor/cns/ttl-parser/tests/fixtures/large.ttl` (869 bytes)
- `/Users/sac/bitactor/cns/ttl-parser/tests/fixtures/simple.ttl` (271 bytes)
- `/Users/sac/bitactor/cns/ttl-parser/tests/fixtures/unicode.ttl` (1135 bytes)

</details>
