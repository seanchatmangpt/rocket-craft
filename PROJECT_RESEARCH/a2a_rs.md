# Research Dossier: `a2a-rs`

**Total Files:** 8 Ontologies (.ttl) | 3 Queries (.rq)
**Total Volume:** 11 files

## 1. Core Vocabularies (Prefixes)
- `a2a: <http://example.org/a2a#>`
- `a2a: <https://ggen.io/ontology/a2a/>`
- `owl: <http://www.w3.org/2002/07/owl#>`
- `rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>`
- `rdfs: <http://www.w3.org/2000/01/rdf-schema#>`
- `xsd: <http://www.w3.org/2001/XMLSchema#>`

## 2. Domain Taxonomy & Entities
### Base Classes & Shapes
- `a2a:A2AError`
- `a2a:APIKeySecurityScheme`
- `a2a:Agent`
- `a2a:AgentCapabilities`
- `a2a:AgentCard`
- `a2a:AgentCardSignature`
- `a2a:AgentExtension`
- `a2a:AgentInterface`
- `a2a:AgentProvider`
- `a2a:AgentSkill`
- `a2a:Artifact`
- `a2a:AuthenticatedExtendedCardNotConfiguredError`
- `a2a:CancelTaskRequest`
- `a2a:ContentTypeNotSupportedError`
- `a2a:DataPart`
- `a2a:DebateAgent`
- `a2a:DeleteTaskPushNotificationConfigRequest`
- `a2a:DialogueAgent`
- `a2a:FileContent`
- `a2a:FilePart`
- `a2a:FileWithBytes`
- `a2a:FileWithUri`
- `a2a:GetAuthenticatedExtendedCardRequest`
- `a2a:GetTaskPushNotificationConfigRequest`
- `a2a:GetTaskRequest`
- `a2a:HTTPAuthSecurityScheme`
- `a2a:InternalError`
- `a2a:InvalidAgentResponseError`
- `a2a:InvalidParamsError`
- `a2a:InvalidRequestError`
- `a2a:JSONParseError`
- `a2a:JSONRPCError`
- `a2a:JSONRPCErrorResponse`
- `a2a:JSONRPCMessage`
- `a2a:JSONRPCRequest`
- `a2a:JSONRPCResponse`
- `a2a:JSONRPCSuccessResponse`
- `a2a:ListTaskPushNotificationConfigRequest`
- `a2a:ListTasksParams`
- `a2a:ListTasksRequest`
- `a2a:Message`
- `a2a:MessageSendConfiguration`
- `a2a:MessageSendParams`
- `a2a:MethodNotFoundError`
- `a2a:MutualTLSSecurityScheme`
- `a2a:OAuth2SecurityScheme`
- `a2a:OpenIdConnectSecurityScheme`
- `a2a:Part`
- `a2a:PushNotificationAuthenticationInfo`
- `a2a:PushNotificationConfig`
- *...and 19 more.*

## 3. Extraction Layer (SPARQL)
- **Query Types Executed:** {'SELECT': 3}

### Projected Variables (SELECT ?var)
This project actively projects the following variables into code/templates:
> `?agent`, `?agent_id`, `?description`, `?label`, `?max_chars`, `?model`, `?output_format`, `?role`, `?sentence_count`

## 4. File Inventory
<details>
<summary>Click to expand all files</summary>

- `/Users/sac/a2a-rs/ggen/ontology/a2a-agent.ttl` (17696 bytes)
- `/Users/sac/a2a-rs/ggen/ontology/a2a-events-errors.ttl` (12612 bytes)
- `/Users/sac/a2a-rs/ggen/ontology/a2a-message.ttl` (16330 bytes)
- `/Users/sac/a2a-rs/ggen/ontology/a2a-requests.ttl` (35252 bytes)
- `/Users/sac/a2a-rs/ggen/ontology/a2a-schema.ttl` (37501 bytes)
- `/Users/sac/a2a-rs/ggen/ontology/a2a-task.ttl` (11969 bytes)
- `/Users/sac/a2a-rs/test-zai/ontologies/agents.ttl` (7658 bytes)
- `/Users/sac/a2a-rs/test-zai/ontologies/tools.ttl` (2358 bytes)
- `/Users/sac/a2a-rs/test-zai/queries/all_agents.rq` (1129 bytes)
- `/Users/sac/a2a-rs/test-zai/queries/dialogue_agents.rq` (902 bytes)
- `/Users/sac/a2a-rs/test-zai/queries/tool_agents.rq` (1038 bytes)

</details>
