# Research Dossier: `autotel`

**Total Files:** 66 Ontologies (.ttl) | 0 Queries (.rq)
**Total Volume:** 66 files

## 1. Core Vocabularies (Prefixes)
- `(case-insensitive)
PREFIX foaf: <http://xmlns.com/foaf/0.1/>`
- `actor: <http://example.org/actor#>`
- `bit: <http://example.org/bitactor#>`
- `cns: <http://cns.ai/ontology#>`
- `cns: <http://cns.autotel.org/ontology#>`
- `cns: <https://schema.chatman.ai/cns#>`
- `dflss: <http://cns.autotel.org/ontology/dflss#>`
- `dspy: <http://dspy.ai/ontology#>`
- `ex1: <http://example1.org/>`
- `ex2: <http://example2.org/>`
- `ex: <http://example.org/>`
- `ex: <http://example.org/dspy#>`
- `foaf: <http://xmlns.com/foaf/0.1/>`
- `gate: <https://schema.chatman.ai/gatekeeper#>`
- `inst: <http://bitactor.org/instance#>`
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
- `weaver: <https://schema.chatman.ai/weaver#>`
- `xsd: <http://www.w3.org/2001/XMLSchema#>`

## 2. Domain Taxonomy & Entities
### Base Classes & Shapes
- `:Actor`
- `:Appointment`
- `:AppointmentShape`
- `:Behavior`
- `:CNSv8SignatureShape`
- `:Contract`
- `:DoctorShape`
- `:Employee`
- `:Entanglement`
- `:IndividualContributor`
- `:Manager`
- `:Measurement`
- `:Opcode`
- `:PatientShape`
- `:Person`
- `:TelemetrySpan`
- `:TurtleLoopProcessingShape`
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
- `cns:Domain`
- `cns:DomainShape`
- *...and 162 more.*

## 3. Extraction Layer (SPARQL)
- *No queries executed in this project.*

### Projected Variables (SELECT ?var)
- *No specific projection variables identified.*

## 4. File Inventory
<details>
<summary>Click to expand all files</summary>

- `/Users/sac/autotel/autotel/engines/seven_tick/cns/business_rules.ttl` (2281 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/codegen/simple.ttl` (16 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/docs/ontology/cns-architecture.ttl` (20702 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/docs/ontology/cns-build.ttl` (14481 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/docs/ontology/cns-compilers.ttl` (16007 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/docs/ontology/cns-complete.ttl` (17361 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/docs/ontology/cns-core.ttl` (6679 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/docs/ontology/cns-domains.ttl` (15729 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/docs/ontology/cns-engines.ttl` (18769 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/docs/ontology/cns-enhanced.ttl` (33962 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/docs/ontology/cns-gatekeeper.ttl` (20553 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/docs/ontology/cns-master.ttl` (11099 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/docs/ontology/cns-performance.ttl` (13435 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/docs/ontology/cns-seven-t.ttl` (22800 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/docs/ontology/cns-telemetry.ttl` (10497 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/docs/ontology/cns-testing.ttl` (19958 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/docs/ontology/cns-tests.ttl` (13359 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/docs/ontology/cns-weaver.ttl` (17722 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/docs/ontology/dflss-ontology.ttl` (19920 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/simple.ttl` (16 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/src/binary_materializer/test_semantic.ttl` (2621 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/test_debug_simple.ttl` (63 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/test_ontology.ttl` (862 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/ttl-parser/complex_test.ttl` (42 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/ttl-parser/examples/grammar-examples.ttl` (7774 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/ttl-parser/examples/sample_query.ttl` (731 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/ttl-parser/examples/simple.ttl` (1147 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/ttl-parser/network_test.ttl` (92 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/ttl-parser/prefix_test.ttl` (53 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/ttl-parser/rich_test.ttl` (171 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/ttl-parser/simple_test.ttl` (71 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/ttl-parser/test-minimal.ttl` (62 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/ttl-parser/test.ttl` (194 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/ttl-parser/test_complex.ttl` (199 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/ttl-parser/test_debug.ttl` (27 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/ttl-parser/test_debug_simple.ttl` (63 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/ttl-parser/test_minimal.ttl` (63 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/ttl-parser/test_query.ttl` (54 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/ttl-parser/test_query_fixed.ttl` (197 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/ttl-parser/test_query_prefix.ttl` (135 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/ttl-parser/test_sample.ttl` (283 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/ttl-parser/test_sample2.ttl` (437 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/ttl-parser/test_simple.ttl` (171 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/ttl-parser/test_simple_query.ttl` (29 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/ttl-parser/tests/fixtures/complex.ttl` (1766 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/ttl-parser/tests/fixtures/edge_cases.ttl` (2219 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/ttl-parser/tests/fixtures/invalid.ttl` (637 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/ttl-parser/tests/fixtures/large.ttl` (869 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/ttl-parser/tests/fixtures/simple.ttl` (271 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/cns/ttl-parser/tests/fixtures/unicode.ttl` (1135 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/examples/sprint_health/spec/ontology.ttl` (1010 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/examples/sprint_health/spec/shapes.ttl` (1014 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/port/Makefile.ttl` (1140 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/port/bitactor_core_ontology.ttl` (4591 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/port/cns-gatekeeper.ttl` (20553 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/port/cns-master.ttl` (11099 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/port/dspy_owl_shacl_demo.ttl` (9052 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/port/examples/dspy_shapes.ttl` (8775 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/port/examples/dspy_signatures.ttl` (8176 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/port/examples/ontology.ttl` (1010 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/port/examples/shapes.ttl` (1014 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/port/examples/sprint_health/spec/ontology.ttl` (1010 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/port/examples/sprint_health/spec/shapes.ttl` (1014 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/port/my_actor_spec.ttl` (1750 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/port/sample_data.ttl` (5946 bytes)
- `/Users/sac/autotel/autotel/engines/seven_tick/specs/backtest_strategy.ttl` (398 bytes)

</details>
