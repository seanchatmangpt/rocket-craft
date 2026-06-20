# Research Dossier: `erlmcp`

**Total Files:** 26 Ontologies (.ttl) | 19 Queries (.rq)
**Total Volume:** 45 files

## 1. Core Vocabularies (Prefixes)
- `dc: <http://purl.org/dc/elements/1.1/>`
- `dcterms: <http://purl.org/dc/terms/>`
- `dist: <http://erlmcp.org/schema/distributed/>`
- `dlock: <http://erlmcp.org/schema/dlock/>`
- `erlmcp: <http://erlmcp.org/schema/>`
- `events: <http://erlmcp.org/schema/events/>`
- `ex: <http://example.org/erlmcp/production#>`
- `mcp: <http://erlmcp.org/schema/mcp/>`
- `mcp: <http://modelcontextprotocol.io/2025-11-25/>`
- `mt: <http://erlmcp.org/schema/multitenant/>`
- `otel: <http://erlmcp.org/schema/otel/>`
- `otp: <http://erlang.org/otp/>`
- `owl: <http://www.w3.org/2002/07/owl#>`
- `protocol: <http://erlmcp.org/schema/protocol/>`
- `proxy: <http://erlmcp.org/schema/proxy/>`
- `queue: <http://erlmcp.org/schema/queue/>`
- `rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>`
- `rdfs: <http://www.w3.org/2000/01/rdf-schema#>`
- `sandbox: <http://erlmcp.org/schema/sandbox/>`
- `sh: <http://www.w3.org/ns/shacl#>`
- `skos: <http://www.w3.org/2004/02/skos/core#>`
- `taiea: <http://taiea.io/ontology/taiea#>`
- `tcps-core: <http://erlmcp.org/ontology/tcps/core#>`
- `tcps-flow: <http://erlmcp.org/ontology/tcps/flow#>`
- `tcps-quality: <http://erlmcp.org/ontology/tcps/quality#>`
- `tcps: <http://erlmcp.org/ontology/tcps#>`
- `tcps: <http://example.org/tcps#>`
- `tcps: <http://taiea.ai/ontology/tcps#>`
- `tcps: <http://taiea.io/ontology/tcps#>`
- `transport: <http://erlmcp.org/schema/transport/>`
- `watcher: <http://erlmcp.org/schema/watcher/>`
- `wf: <http://erlmcp.org/schema/workflow/>`
- `xsd: <http://www.w3.org/2001/XMLSchema#>`

## 2. Domain Taxonomy & Entities
### Base Classes & Shapes
- `dist:ClusterMembership`
- `dist:Connection`
- `dist:ConnectionState`
- `dist:Consensus`
- `dist:ConsensusStrategy`
- `dist:ElectionStrategy`
- `dist:LeaderElection`
- `dist:MembershipStrategy`
- `dist:Node`
- `dist:NodeCapabilities`
- `dist:NodeMetric`
- `dist:NodeStatus`
- `dist:Partition`
- `dist:PartitionTolerance`
- `dist:RegisteredProcess`
- `dist:RegistrationStrategy`
- `dist:Registry`
- `dist:Replication`
- `dist:ReplicationStrategy`
- `dlock:DistributedLock`
- `dlock:FastFail`
- `dlock:Lock`
- `dlock:LockAcquired`
- `dlock:LockExpired`
- `dlock:LockOptions`
- `dlock:LockReleased`
- `dlock:LockRequest`
- `dlock:LockStolen`
- `dlock:Queue`
- `dlock:Wait`
- `erlmcp:Behavior`
- `erlmcp:CheckType`
- `erlmcp:Client`
- `erlmcp:Connection`
- `erlmcp:ConnectionState`
- `erlmcp:Dependency`
- `erlmcp:HealthCheck`
- `erlmcp:HealthStatus`
- `erlmcp:JsonRpcCodec`
- `erlmcp:Metadata`
- `erlmcp:Server`
- `erlmcp:Transport`
- `events:Aggregate`
- `events:DiskStore`
- `events:ETSStore`
- `events:Event`
- `events:EventStore`
- `events:IntervalSnapshot`
- `events:ManualSnapshot`
- `events:MemoryStore`
- *...and 219 more.*

## 3. Extraction Layer (SPARQL)
- **Query Types Executed:** {'CONSTRUCT': 8, 'SELECT': 11}

### Projected Variables (SELECT ?var)
This project actively projects the following variables into code/templates:
> `?andon_id`, `?bucket`, `?completed`, `?created`, `?created_timestamp`, `?demand_signal`, `?elapsed_hours`, `?failed_receipts`, `?failure_reason`, `?is_ready`, `?missing_receipts`, `?open_andon_count`, `?priority`, `?rank_in_bucket`, `?receipt_id`, `?receipt_timestamp`, `?severity`, `?sku_id`, `?stage`, `?stage_sequence`, `?status`, `?time_period`, `?triggered_timestamp`, `?validation_data`, `?validator`, `?work_order_id`

## 4. File Inventory
<details>
<summary>Click to expand all files</summary>

- `/Users/sac/erlmcp/ontology/erlmcp_new/distributed.ttl` (14187 bytes)
- `/Users/sac/erlmcp/ontology/erlmcp_new/dlock.ttl` (4804 bytes)
- `/Users/sac/erlmcp/ontology/erlmcp_new/domain.ttl` (8314 bytes)
- `/Users/sac/erlmcp/ontology/erlmcp_new/events.ttl` (6000 bytes)
- `/Users/sac/erlmcp/ontology/erlmcp_new/instances.ttl` (4558 bytes)
- `/Users/sac/erlmcp/ontology/erlmcp_new/mcp.ttl` (12361 bytes)
- `/Users/sac/erlmcp/ontology/erlmcp_new/multitenant.ttl` (6617 bytes)
- `/Users/sac/erlmcp/ontology/erlmcp_new/observability.ttl` (14506 bytes)
- `/Users/sac/erlmcp/ontology/erlmcp_new/otp.ttl` (13163 bytes)
- `/Users/sac/erlmcp/ontology/erlmcp_new/protocols.ttl` (2138 bytes)
- `/Users/sac/erlmcp/ontology/erlmcp_new/proxy.ttl` (5357 bytes)
- `/Users/sac/erlmcp/ontology/erlmcp_new/queue.ttl` (6221 bytes)
- `/Users/sac/erlmcp/ontology/erlmcp_new/sandbox.ttl` (5618 bytes)
- `/Users/sac/erlmcp/ontology/erlmcp_new/transports.ttl` (2160 bytes)
- `/Users/sac/erlmcp/ontology/erlmcp_new/watcher.ttl` (5780 bytes)
- `/Users/sac/erlmcp/ontology/erlmcp_new/workflow.ttl` (6449 bytes)
- `/Users/sac/erlmcp/ontology/example_instance_data.ttl` (13752 bytes)
- `/Users/sac/erlmcp/ontology/mcp.ttl` (49736 bytes)
- `/Users/sac/erlmcp/ontology/tcps_core.ttl` (23470 bytes)
- `/Users/sac/erlmcp/ontology/tcps_flow.ttl` (28618 bytes)
- `/Users/sac/erlmcp/ontology/tcps_quality.ttl` (24444 bytes)
- `/Users/sac/erlmcp/ontology/tcps_root_cause.ttl` (15691 bytes)
- `/Users/sac/erlmcp/ontology/work_orders.ttl` (21085 bytes)
- `/Users/sac/erlmcp/shapes/tcps_shapes.ttl` (19905 bytes)
- `/Users/sac/erlmcp/sparql/erlmcp_queries/extract_dlock.rq` (880 bytes)
- `/Users/sac/erlmcp/sparql/erlmcp_queries/extract_events.rq` (870 bytes)
- `/Users/sac/erlmcp/sparql/erlmcp_queries/extract_multitenant.rq` (1612 bytes)
- `/Users/sac/erlmcp/sparql/erlmcp_queries/extract_proxy.rq` (1932 bytes)
- `/Users/sac/erlmcp/sparql/erlmcp_queries/extract_queue.rq` (1442 bytes)
- `/Users/sac/erlmcp/sparql/erlmcp_queries/extract_sandbox.rq` (1645 bytes)
- `/Users/sac/erlmcp/sparql/erlmcp_queries/extract_watcher.rq` (1090 bytes)
- `/Users/sac/erlmcp/sparql/erlmcp_queries/extract_workflow.rq` (1403 bytes)
- `/Users/sac/erlmcp/sparql/tcps_queries/andon_active.rq` (2812 bytes)
- `/Users/sac/erlmcp/sparql/tcps_queries/heijunka_schedule.rq` (2531 bytes)
- `/Users/sac/erlmcp/sparql/tcps_queries/quality_metrics.rq` (4080 bytes)
- `/Users/sac/erlmcp/sparql/tcps_queries/receipts_by_stage.rq` (2918 bytes)
- `/Users/sac/erlmcp/sparql/tcps_queries/sku_readiness.rq` (3548 bytes)
- `/Users/sac/erlmcp/sparql/tcps_queries/work_orders_pending.rq` (1767 bytes)
- `/Users/sac/erlmcp/sparql/tcps_queries_optimized/andon_active_optimized.rq` (1487 bytes)
- `/Users/sac/erlmcp/sparql/tcps_queries_optimized/heijunka_schedule_optimized.rq` (1398 bytes)
- `/Users/sac/erlmcp/sparql/tcps_queries_optimized/quality_metrics_optimized.rq` (1621 bytes)
- `/Users/sac/erlmcp/sparql/tcps_queries_optimized/receipts_by_stage_optimized.rq` (1249 bytes)
- `/Users/sac/erlmcp/sparql/tcps_queries_optimized/sku_readiness_optimized.rq` (1546 bytes)
- `/Users/sac/erlmcp/tests/shacl/test_data_invalid.ttl` (10546 bytes)
- `/Users/sac/erlmcp/tests/shacl/test_data_valid.ttl` (4229 bytes)

</details>
