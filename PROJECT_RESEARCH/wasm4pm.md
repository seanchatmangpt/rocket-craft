# Research Dossier: `wasm4pm`

**Total Files:** 33 Ontologies (.ttl) | 9 Queries (.rq)
**Total Volume:** 42 files

## 1. Core Vocabularies (Prefixes)
- `adms: <http://www.w3.org/ns/adms#>`
- `aux: <https://w3id.org/ocedo/auxiliary#>`
- `bibo: <http://purl.org/ontology/bibo/>`
- `cmns-cls: <https://www.omg.org/spec/Commons/Classifiers/>`
- `cmns-col: <https://www.omg.org/spec/Commons/Collections/>`
- `cmns-dt: <https://www.omg.org/spec/Commons/DatesAndTimes/>`
- `cmns-ge: <https://www.omg.org/spec/Commons/GeopoliticalEntities/>`
- `cmns-id: <https://www.omg.org/spec/Commons/Identifiers/>`
- `cmns-loc: <https://www.omg.org/spec/Commons/Locations/>`
- `cmns-q: <https://www.omg.org/spec/Commons/Quantities/>`
- `cmns-txt: <https://www.omg.org/spec/Commons/Text/>`
- `compat: <https://wasm4pm-compat.rs/ontology#>`
- `compat: <https://wasm4pm.dev/ns#>`
- `cr: <http://mlcommons.org/croissant/>`
- `dc: <http://purl.org/dc/elements/1.1/>`
- `dc: <http://purl.org/dc/terms/>`
- `dcam: <http://purl.org/dc/dcam/>`
- `dcat: <http://www.w3.org/ns/dcat#>`
- `dcmitype: <http://purl.org/dc/dcmitype/>`
- `dct: <http://purl.org/dc/terms/>`
- `dcterms: <http://purl.org/dc/terms/>`
- `dctype: <http://purl.org/dc/dcmitype/>`
- `dmop: <http://www.e-lico.eu/ontologies/dmo/DMOP/DMOP.owl#>`
- `doap: <http://usefulinc.com/ns/doap#>`
- `fibo-be-corp-corp: <https://spec.edmcouncil.org/fibo/ontology/BE/Corporations/Corporations/>`
- `fibo-be-ge-ge: <https://spec.edmcouncil.org/fibo/ontology/BE/GovernmentEntities/GovernmentEntities/>`
- `fibo-be-le-cb: <https://spec.edmcouncil.org/fibo/ontology/BE/LegalEntities/CorporateBodies/>`
- `fibo-be-le-lp: <https://spec.edmcouncil.org/fibo/ontology/BE/LegalEntities/LegalPersons/>`
- `fibo-be-nfp-nfp: <https://spec.edmcouncil.org/fibo/ontology/BE/NotForProfitOrganizations/NotForProfitOrganizations/>`
- `fibo-be-oac-cctl: <https://spec.edmcouncil.org/fibo/ontology/BE/OwnershipAndControl/CorporateControl/>`
- `fibo-fbc-dae-dbt: <https://spec.edmcouncil.org/fibo/ontology/FBC/DebtAndEquities/Debt/>`
- `fibo-fbc-pas-fpas: <https://spec.edmcouncil.org/fibo/ontology/FBC/ProductsAndServices/FinancialProductsAndServices/>`
- `fibo-fnd-acc-cur: <https://spec.edmcouncil.org/fibo/ontology/FND/Accounting/CurrencyAmount/>`
- `fibo-fnd-agr-ctr: <https://spec.edmcouncil.org/fibo/ontology/FND/Agreements/Contracts/>`
- `fibo-fnd-arr-doc: <https://spec.edmcouncil.org/fibo/ontology/FND/Arrangements/Documents/>`
- `fibo-fnd-arr-lif: <https://spec.edmcouncil.org/fibo/ontology/FND/Arrangements/Lifecycles/>`
- `fibo-fnd-dt-oc: <https://spec.edmcouncil.org/fibo/ontology/FND/DatesAndTimes/Occurrences/>`
- `fibo-fnd-org-org: <https://spec.edmcouncil.org/fibo/ontology/FND/Organizations/Organizations/>`
- `fibo-fnd-pas-pas: <https://spec.edmcouncil.org/fibo/ontology/FND/ProductsAndServices/ProductsAndServices/>`
- `fibo-fnd-plc-adr: <https://spec.edmcouncil.org/fibo/ontology/FND/Places/Addresses/>`
- `fibo-fnd-plc-fac: <https://spec.edmcouncil.org/fibo/ontology/FND/Places/Facilities/>`
- `fibo-fnd-plc-loc: <https://spec.edmcouncil.org/fibo/ontology/FND/Places/Locations/>`
- `fibo-fnd-pty-pty: <https://spec.edmcouncil.org/fibo/ontology/FND/Parties/Parties/>`
- `fibo-fnd-rel-rel: <https://spec.edmcouncil.org/fibo/ontology/FND/Relations/Relations/>`
- `fibo-pay-ps-ps: <https://spec.edmcouncil.org/fibo/ontology/PAY/PaymentServices/PaymentServices/>`
- `foaf: <http://xmlns.com/foaf/0.1/>`
- `gleif-L1: <https://www.gleif.org/ontology/L1/>`
- `grddl: <http://www.w3.org/2003/g/data-view#>`
- `gs1: <https://ref.gs1.org/voc/>`
- `lcc-cr: <https://www.omg.org/spec/LCC/Countries/CountryRepresentation/>`
- *...and 33 more.*

## 2. Domain Taxonomy & Entities
### Base Classes & Shapes
- `:AMRadioChannel`
- `:APIReference`
- `:AboutPage`
- `:AcceptAction`
- `:Accommodation`
- `:AccountingService`
- `:AchieveAction`
- `:Action`
- `:ActionAccessSpecification`
- `:ActionStatusType`
- `:ActivateAction`
- `:Activity`
- `:ActivityInfluence`
- `:AddAction`
- `:AdministrativeArea`
- `:AdultEntertainment`
- `:Agent`
- `:AgentInfluence`
- `:AggregateOffer`
- `:AggregateRating`
- `:AgreeAction`
- `:Airline`
- `:Airport`
- `:AlignmentObject`
- `:AllocateAction`
- `:AmusementPark`
- `:AnimalShelter`
- `:Answer`
- `:Apartment`
- `:ApartmentComplex`
- `:AppendAction`
- `:ApplyAction`
- `:Aquarium`
- `:ArriveAction`
- `:ArtGallery`
- `:Article`
- `:AskAction`
- `:AssessAction`
- `:AssignAction`
- `:Association`
- `:Attorney`
- `:Attribution`
- `:Audience`
- `:AudioObject`
- `:AuthorizeAction`
- `:AutoBodyShop`
- `:AutoDealer`
- `:AutoPartsStore`
- `:AutoRental`
- `:AutoRepair`
- *...and 750 more.*

## 3. Extraction Layer (SPARQL)
- **Query Types Executed:** {'SELECT': 3, 'CONSTRUCT': 6}

### Projected Variables (SELECT ?var)
This project actively projects the following variables into code/templates:
> `?algorithm_doc`, `?algorithm_id`, `?algorithm_label`, `?algorithm_status`, `?category`, `?citation`, `?cli_alias`, `?decoy_reason`, `?description`, `?hardcode_lockable`, `?input_format`, `?module_file`, `?output_type`, `?paperReference`, `?pi_standing`, `?pointer_derivation`, `?pointer_kind`, `?pointer_locus`, `?pointer_value`, `?quality_tier`, `?rustType`, `?speed_tier`, `?vda_fitness`, `?vda_generalization`, `?vda_precision`, `?vda_simplicity`, `?wasm_export`, `?witnessFamily`, `?witnessKey`, `?witnessTitle`, `?witnessYear`

## 4. File Inventory
<details>
<summary>Click to expand all files</summary>

- `/Users/sac/wasm4pm/crates/prolog8/ontology/prolog8.ttl` (3873 bytes)
- `/Users/sac/wasm4pm/ggen/ontology-algorithms/algorithm-pointers.ttl` (9596 bytes)
- `/Users/sac/wasm4pm/ggen/ontology-algorithms/algorithm-vocabulary.ttl` (7219 bytes)
- `/Users/sac/wasm4pm/ggen/ontology/algorithms.ttl` (44955 bytes)
- `/Users/sac/wasm4pm/ggen/ontology/breeds.ttl` (24603 bytes)
- `/Users/sac/wasm4pm/ggen/queries-algorithms/extract-algorithm-pointers.rq` (1001 bytes)
- `/Users/sac/wasm4pm/ggen/queries-algorithms/extract-algorithms.rq` (1492 bytes)
- `/Users/sac/wasm4pm/ggen/queries/extract-witnesses-full.rq` (909 bytes)
- `/Users/sac/wasm4pm/ocel/reports/evidence.ttl` (9163 bytes)
- `/Users/sac/wasm4pm/ocel/reports/pi_evidence.ttl` (9732 bytes)
- `/Users/sac/wasm4pm/ontology/public-alignment.ttl` (1497 bytes)
- `/Users/sac/wasm4pm/ontology/public-shapes.ttl` (1071 bytes)
- `/Users/sac/wasm4pm/ontology/standards/croissant.ttl` (478 bytes)
- `/Users/sac/wasm4pm/ontology/standards/dcat3.ttl` (1964 bytes)
- `/Users/sac/wasm4pm/ontology/standards/dcterms.ttl` (47834 bytes)
- `/Users/sac/wasm4pm/ontology/standards/dmop.ttl` (649 bytes)
- `/Users/sac/wasm4pm/ontology/standards/doap.ttl` (3535 bytes)
- `/Users/sac/wasm4pm/ontology/standards/linkml.ttl` (380 bytes)
- `/Users/sac/wasm4pm/ontology/standards/mex-algo.ttl` (578 bytes)
- `/Users/sac/wasm4pm/ontology/standards/mex-core.ttl` (782 bytes)
- `/Users/sac/wasm4pm/ontology/standards/mex-perf.ttl` (591 bytes)
- `/Users/sac/wasm4pm/ontology/standards/mls.ttl` (50079 bytes)
- `/Users/sac/wasm4pm/ontology/standards/ocel20.ttl` (3017 bytes)
- `/Users/sac/wasm4pm/ontology/standards/odrl22.ttl` (3060 bytes)
- `/Users/sac/wasm4pm/ontology/standards/ontodm.ttl` (1070046 bytes)
- `/Users/sac/wasm4pm/ontology/standards/owl.ttl` (23964 bytes)
- `/Users/sac/wasm4pm/ontology/standards/prov-o.ttl` (68795 bytes)
- `/Users/sac/wasm4pm/ontology/standards/rdf.ttl` (6004 bytes)
- `/Users/sac/wasm4pm/ontology/standards/rdfs.ttl` (3790 bytes)
- `/Users/sac/wasm4pm/ontology/standards/schema.ttl` (464683 bytes)
- `/Users/sac/wasm4pm/ontology/standards/shacl.ttl` (52899 bytes)
- `/Users/sac/wasm4pm/ontology/standards/skos.ttl` (28966 bytes)
- `/Users/sac/wasm4pm/ontology/standards/spdx.ttl` (359 bytes)
- `/Users/sac/wasm4pm/ontology/standards/time.ttl` (21261 bytes)
- `/Users/sac/wasm4pm/semconv/sparql-proofs/conformance-check.rq` (2443 bytes)
- `/Users/sac/wasm4pm/semconv/sparql-proofs/detect-drift.rq` (4567 bytes)
- `/Users/sac/wasm4pm/semconv/sparql-proofs/discover-dfg.rq` (2254 bytes)
- `/Users/sac/wasm4pm/semconv/sparql-proofs/ml-classify.rq` (6131 bytes)
- `/Users/sac/wasm4pm/semconv/sparql-proofs/ocel-load.rq` (5472 bytes)
- `/Users/sac/wasm4pm/semconv/sparql-proofs/predict-activity.rq` (4655 bytes)
- `/Users/sac/wasm4pm/semconv/wasm4pm-ontology.ttl` (10419 bytes)
- `/Users/sac/wasm4pm/semconv/wasm4pm-shapes.ttl` (3905 bytes)

</details>
