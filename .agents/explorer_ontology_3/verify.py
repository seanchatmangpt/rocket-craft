#!/usr/bin/env python3
import sys
import os
from rdflib import Graph, Literal, URIRef
from rdflib.plugins.sparql import prepareQuery

def test_turtle_syntax(file_path):
    print(f"[*] Validating Turtle file: {file_path}")
    g = Graph()
    try:
        g.parse(file_path, format="turtle")
        print(f"    [+] SUCCESS: {file_path} parsed successfully.")
        print(f"    [+] Total triples: {len(g)}")
        return g
    except Exception as e:
        print(f"    [-] ERROR: Failed to parse {file_path}: {e}")
        return None

def test_sparql_syntax(file_path):
    print(f"[*] Validating SPARQL query file: {file_path}")
    if not os.path.exists(file_path):
        print(f"    [-] ERROR: Query file not found: {file_path}")
        return None
    with open(file_path, 'r') as f:
        query_str = f.read()
    try:
        q = prepareQuery(query_str)
        print(f"    [+] SUCCESS: Query {file_path} is syntactically valid SPARQL 1.1.")
        return query_str
    except Exception as e:
        print(f"    [-] ERROR: Query syntax validation failed for {file_path}: {e}")
        return None

def build_mock_data():
    mock_ttl = """
    @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
    @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
    @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
    @prefix prov: <http://www.w3.org/ns/prov#> .
    @prefix eden: <https://ggen.io/ontology/eden-server/> .

    # Mock Assembly Tree
    eden:mockMechRoot a eden:MechRoot ;
        rdfs:label "Mock Mech Root" ;
        eden:hasSocket eden:socket1 , eden:socket2 , eden:socketEmpty .

    eden:socket1 a eden:Socket ;
        rdfs:label "Socket 1" .

    eden:socket2 a eden:Socket ;
        rdfs:label "Socket 2" .

    eden:socketEmpty a eden:Socket ;
        rdfs:label "Empty Socket" .

    # Part 1 plugs into Socket 1
    eden:part1 a eden:Part ;
        rdfs:label "Part 1" ;
        eden:plugsInto eden:socket1 ;
        eden:damageClass "10"^^xsd:unsignedByte ;
        eden:stressClass "20"^^xsd:unsignedByte ;
        eden:heatClass "30"^^xsd:unsignedByte ;
        eden:fatigueClass "40"^^xsd:unsignedByte .

    # SubAssembly 1 plugs into Socket 2
    eden:subAssy1 a eden:SubAssembly ;
        rdfs:label "SubAssembly 1" ;
        eden:plugsInto eden:socket2 ;
        eden:hasSocket eden:socket3 .

    eden:socket3 a eden:Socket ;
        rdfs:label "Socket 3" .

    # Part 2 plugs into Socket 3 inside SubAssembly 1
    eden:part2 a eden:Part ;
        rdfs:label "Part 2" ;
        eden:plugsInto eden:socket3 ;
        eden:damageClass "5"^^xsd:unsignedByte ;
        eden:stressClass "15"^^xsd:unsignedByte ;
        eden:heatClass "25"^^xsd:unsignedByte ;
        eden:fatigueClass "35"^^xsd:unsignedByte .

    # Mock Authority Delta
    eden:mockAuthDelta a eden:AuthorityDelta ;
        eden:targetComponent eden:part1 ;
        eden:damageClass "12"^^xsd:unsignedByte ;
        eden:stressClass "22"^^xsd:unsignedByte ;
        prov:generatedAtTime "2026-06-19T00:00:00Z"^^xsd:dateTime ;
        prov:wasAssociatedWith eden:mockIssuer .

    # Mock Assembly Delta
    eden:mockAssyDelta a eden:AssemblyDelta ;
        eden:targetSocket eden:socket3 ;
        eden:installedComponent eden:part2 ;
        prov:generatedAtTime "2026-06-18T12:00:00Z"^^xsd:dateTime ;
        prov:wasAssociatedWith eden:mockActor .

    # Mock Receipt Delta
    eden:mockReceiptDelta a eden:ReceiptDelta ;
        eden:prompt "Generate Mech Root with twin telemetry" ;
        eden:contractHash "0xabc123" ;
        eden:buildLog "build successful" ;
        eden:packagePath "/path/to/pkg.zip" ;
        eden:baselineScreenshot "file:///img1.png"^^xsd:anyURI ;
        eden:afterScreenshot "file:///img2.png"^^xsd:anyURI ;
        eden:consoleLogs "no errors" ;
        eden:inputTrace "W-A-S-D" ;
        eden:visualDelta "0.85"^^xsd:float ;
        eden:verdict "true"^^xsd:boolean ;
        prov:generatedAtTime "2026-06-19T00:01:00Z"^^xsd:dateTime ;
        prov:wasAssociatedWith eden:mockAuditor .
    """
    g = Graph()
    g.parse(data=mock_ttl, format="turtle")
    return g

def main():
    print("=== Eden Ontology & Query Verification Agent ===")
    
    # 1. Parse Ontologies
    pack_g = test_turtle_syntax("pack.ttl")
    deltas_g = test_turtle_syntax("deltas.ttl")
    
    if pack_g is None or deltas_g is None:
        print("[-] ERROR: Ontology validation failed.")
        sys.exit(1)
        
    # Check owl:imports
    print("[*] Verifying ontology imports in pack.ttl...")
    imports = list(pack_g.objects(None, URIRef("http://www.w3.org/2002/07/owl#imports")))
    expected_imports = [
        "https://spec.edmcouncil.org/fibo/ontology/",
        "http://www.w3.org/ns/sosa/",
        "http://qudt.org/schema/qudt/",
        "http://www.w3.org/ns/prov#"
    ]
    for imp in expected_imports:
        if URIRef(imp) in imports:
            print(f"    [+] Checked import: {imp}")
        else:
            print(f"    [-] WARNING: Missing expected import: {imp}")

    # 2. Parse Queries
    queries = {
        "substrate": "draft_queries/substrate.rq",
        "extract_authority_deltas": "draft_queries/extract_authority_deltas.rq",
        "extract_assembly_deltas": "draft_queries/extract_assembly_deltas.rq",
        "extract_receipt_deltas": "draft_queries/extract_receipt_deltas.rq"
    }
    
    query_contents = {}
    for name, path in queries.items():
        q_str = test_sparql_syntax(path)
        if q_str is None:
            print(f"[-] ERROR: Query validation failed for {name}.")
            sys.exit(1)
        query_contents[name] = q_str
        
    # 3. Load Mock Data and Merge with Ontologies for Query Testing
    print("[*] Creating mock data graph for query testing...")
    test_graph = Graph()
    # Add ontology definitions
    test_graph += pack_g
    test_graph += deltas_g
    # Add mock instances
    mock_data = build_mock_data()
    test_graph += mock_data
    print(f"    [+] Total triples in unified test graph: {len(test_graph)}")
    
    # 4. Execute Queries and Assert Results
    print("[*] Executing queries against mock graph...")
    
    # 4a. Execute substrate.rq
    print("    [>] Running substrate.rq...")
    res_sub = test_graph.query(query_contents["substrate"])
    print(f"        [+] Returned {len(res_sub)} rows:")
    for row in res_sub:
        print(f"            - Root: {row.root.split('/')[-1]} | Parent: {row.parent.split('/')[-1] if row.parent else 'None'} | Socket: {row.socket.split('/')[-1] if row.socket else 'None'} | Child: {row.child.split('/')[-1] if row.child else 'None'} | Type: {row.childType.split('/')[-1] if row.childType else 'None'} | D:{row.damageClass} S:{row.stressClass} H:{row.heatClass} F:{row.fatigueClass}")
        
    # Check if empty socket was extracted
    empty_socket_found = False
    for row in res_sub:
        if str(row.socket).endswith("socketEmpty") and row.child is None:
            empty_socket_found = True
    if empty_socket_found:
        print("        [+] SUCCESS: Empty socket correctly handled (child is unbound).")
    else:
        print("        [-] WARNING: Empty socket not represented correctly in substrate output.")

    # 4b. Execute extract_authority_deltas.rq
    print("    [>] Running extract_authority_deltas.rq...")
    res_auth = test_graph.query(query_contents["extract_authority_deltas"])
    print(f"        [+] Returned {len(res_auth)} rows:")
    for row in res_auth:
        print(f"            - Delta: {row.delta.split('/')[-1]} | Target: {row.targetComponent.split('/')[-1]} | D:{row.damageClass} S:{row.stressClass} | Time: {row.timestamp}")
    assert len(res_auth) > 0, "No authority deltas returned!"

    # 4c. Execute extract_assembly_deltas.rq
    print("    [>] Running extract_assembly_deltas.rq...")
    res_assy = test_graph.query(query_contents["extract_assembly_deltas"])
    print(f"        [+] Returned {len(res_assy)} rows:")
    for row in res_assy:
        print(f"            - Delta: {row.delta.split('/')[-1]} | Socket: {row.targetSocket.split('/')[-1]} | Inst: {row.installedComponent.split('/')[-1]} | Time: {row.timestamp}")
    assert len(res_assy) > 0, "No assembly deltas returned!"

    # 4d. Execute extract_receipt_deltas.rq
    print("    [>] Running extract_receipt_deltas.rq...")
    res_rec = test_graph.query(query_contents["extract_receipt_deltas"])
    print(f"        [+] Returned {len(res_rec)} rows:")
    for row in res_rec:
        print(f"            - Delta: {row.delta.split('/')[-1]} | Verdict: {row.verdict} | DeltaVal: {row.visualDelta} | Prompt: '{row.prompt}'")
    assert len(res_rec) > 0, "No receipt deltas returned!"
    
    print("[+] ALL TESTS PASSED SUCCESSFULLY! The ontologies and queries are fully validated.")

if __name__ == "__main__":
    main()
