#!/usr/bin/env python3
import sys
import os
import time
import traceback
from rdflib import Graph, Literal, URIRef, Namespace
from rdflib.plugins.sparql import prepareQuery
from rdflib.namespace import RDF, RDFS, XSD, OWL

# Define namespaces
EDEN = Namespace("https://ggen.io/ontology/eden-server/")
PROV = Namespace("http://www.w3.org/ns/prov#")

def load_original_ontologies():
    print("[*] Loading original ontologies...")
    g = Graph()
    try:
        g.parse("/Users/sac/.ggen/packs/eden_server/ontology/pack.ttl", format="turtle")
        g.parse("/Users/sac/.ggen/packs/eden_server/ontology/deltas.ttl", format="turtle")
        print(f"    [+] Successfully loaded original ontologies: {len(g)} triples.")
        return g
    except Exception as e:
        print(f"    [-] Failed to load original ontologies: {e}")
        sys.exit(1)

def test_datatype_violations():
    print("\n=== TEST 1: DataType Violations (xsd:unsignedByte & xsd:float) ===")
    g = Graph()
    
    # Define Turtle data with out-of-range or invalid literal values
    invalid_ttl = """
    @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
    @prefix eden: <https://ggen.io/ontology/eden-server/> .
    
    eden:part_negative a eden:Part ;
        eden:damageClass "-1"^^xsd:unsignedByte ;
        eden:stressClass "256"^^xsd:unsignedByte ;
        eden:heatClass "9999"^^xsd:unsignedByte ;
        eden:fatigueClass "not-a-byte"^^xsd:unsignedByte .

    eden:receipt_invalid_float a eden:ReceiptDelta ;
        eden:visualDelta "not-a-float"^^xsd:float ;
        eden:verdict "maybe"^^xsd:boolean .
    """
    
    try:
        g.parse(data=invalid_ttl, format="turtle")
        print("    [+] Parsing: RDFLib parsed out-of-range and invalid literals without error.")
    except Exception as e:
        print(f"    [-] Parsing failed: {e}")
        return

    # Check how RDFLib represents these values locally
    print("    [*] Checking Python native conversion (toPython()):")
    for s, p, o in g:
        if isinstance(o, Literal):
            try:
                py_val = o.toPython()
                print(f"        Triple: ({s.split('/')[-1]}, {p.split('/')[-1]}, '{o}') -> toPython(): {py_val} (Type: {type(py_val).__name__})")
            except Exception as e:
                print(f"        Triple: ({s.split('/')[-1]}, {p.split('/')[-1]}, '{o}') -> toPython() FAILED: {e}")

    # Query using SPARQL to see filter behaviors
    query_str = """
    PREFIX eden: <https://ggen.io/ontology/eden-server/>
    PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>
    SELECT ?part ?damage ?stress ?heat ?fatigue
    WHERE {
        ?part a eden:Part .
        OPTIONAL { ?part eden:damageClass ?damage . }
        OPTIONAL { ?part eden:stressClass ?stress . }
        OPTIONAL { ?part eden:heatClass ?heat . }
        OPTIONAL { ?part eden:fatigueClass ?fatigue . }
    }
    """
    q = prepareQuery(query_str)
    res = list(g.query(q))
    print("    [*] SPARQL Query Results for invalid parts:")
    for row in res:
        print(f"        Part: {row.part.split('/')[-1]}")
        print(f"          - damageClass: '{row.damage}'")
        print(f"          - stressClass: '{row.stress}'")
        print(f"          - heatClass: '{row.heat}'")
        print(f"          - fatigueClass: '{row.fatigue}'")

    # What happens when we run a numeric comparison in SPARQL?
    query_filter = """
    PREFIX eden: <https://ggen.io/ontology/eden-server/>
    PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>
    SELECT ?part
    WHERE {
        ?part a eden:Part .
        ?part eden:damageClass ?damage .
        FILTER (xsd:integer(?damage) < 0)
    }
    """
    try:
        res_filter = list(g.query(prepareQuery(query_filter)))
        print(f"    [*] Filter test (xsd:integer(?damage) < 0) returned: {[r.part.split('/')[-1] for r in res_filter]}")
    except Exception as e:
        print(f"    [-] Filter test (xsd:integer(?damage) < 0) raised error: {e}")

    query_filter2 = """
    PREFIX eden: <https://ggen.io/ontology/eden-server/>
    PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>
    SELECT ?part
    WHERE {
        ?part a eden:Part .
        ?part eden:damageClass ?damage .
        FILTER (?damage < 0)
    }
    """
    try:
        res_filter2 = list(g.query(prepareQuery(query_filter2)))
        print(f"    [*] Filter test (?damage < 0) returned: {[r.part.split('/')[-1] for r in res_filter2]}")
    except Exception as e:
        print(f"    [-] Filter test (?damage < 0) raised error: {e}")

def test_assembly_tree_cycles():
    print("\n=== TEST 2: Assembly Tree Cycles ===")
    g = load_original_ontologies()
    
    # Introduce cycle:
    # root -> hasSocket -> socket1 -> plugsInto -> subAssy1
    # subAssy1 -> hasSocket -> socket2 -> plugsInto -> subAssy2
    # subAssy2 -> hasSocket -> socket3 -> plugsInto -> subAssy1 (Cycle!)
    
    cycle_ttl = """
    @prefix eden: <https://ggen.io/ontology/eden-server/> .
    
    eden:mockRoot a eden:MechRoot ;
        eden:hasSocket eden:socket1 .
        
    eden:socket1 a eden:Socket .
    
    eden:subAssy1 a eden:SubAssembly ;
        eden:plugsInto eden:socket1 ;
        eden:hasSocket eden:socket2 .
        
    eden:socket2 a eden:Socket .
    
    eden:subAssy2 a eden:SubAssembly ;
        eden:plugsInto eden:socket2 ;
        eden:hasSocket eden:socket3 .
        
    eden:socket3 a eden:Socket .
    
    # Close the cycle
    eden:subAssy1 eden:plugsInto eden:socket3 .
    """
    
    g.parse(data=cycle_ttl, format="turtle")
    print(f"    [+] Injected cycle. Unified graph size: {len(g)}")
    
    # Read substrate query
    with open("/Users/sac/.ggen/packs/eden_server/queries/substrate.rq", "r") as f:
        query_str = f.read()
        
    print("    [*] Executing substrate query on cyclic assembly...")
    start_time = time.time()
    try:
        res = list(g.query(prepareQuery(query_str)))
        elapsed = time.time() - start_time
        print(f"    [+] Query finished in {elapsed:.4f} seconds.")
        print(f"    [+] Returned {len(res)} rows:")
        for r in res:
            r_str = f"Root: {r.root.split('/')[-1]} | Parent: {r.parent.split('/')[-1] if r.parent else 'None'} | Socket: {r.socket.split('/')[-1] if r.socket else 'None'} | Child: {r.child.split('/')[-1] if r.child else 'None'}"
            print(f"        - {r_str}")
    except Exception as e:
        print(f"    [-] Query execution failed or hung on cyclic data: {e}")
        traceback.print_exc()

def test_namespace_conflicts_and_shadowing():
    print("\n=== TEST 3: Namespace Conflicts and URI Edge Cases ===")
    
    # What if someone uses 'https://ggen.io/ontology/eden-server' (no trailing slash) instead of 'https://ggen.io/ontology/eden-server/' (with trailing slash)
    # What if they query with 'https://ggen.io/ontology/eden-server/' prefix but definitions use 'https://ggen.io/ontology/eden-server'?
    
    g = Graph()
    ttl_no_slash = """
    @prefix eden_noslash: <https://ggen.io/ontology/eden-server> .
    @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
    
    # Note the missing slash in prefix, so resource becomes <https://ggen.io/ontology/eden-serverpart1> or similar if concatenated, 
    # or <https://ggen.io/ontology/eden-server/part1> if defined properly. Let's see what happens.
    # Actually, if the prefix is <https://ggen.io/ontology/eden-server> (without slash) and we write:
    eden_noslash:part1 a eden_noslash:Part .
    # The URI resolves to <https://ggen.io/ontology/eden-serverpart1>
    """
    g.parse(data=ttl_no_slash, format="turtle")
    print("    [*] Listing parsed triples for no-slash prefix:")
    for s, p, o in g:
        print(f"        Subject: {s} | Predicate: {p} | Object: {o}")
        
    # Test query using query with standard trailing slash prefix
    query_str = """
    PREFIX eden: <https://ggen.io/ontology/eden-server/>
    SELECT ?x WHERE { ?x a eden:Part . }
    """
    res = list(g.query(prepareQuery(query_str)))
    print(f"    [+] Query searching for <https://ggen.io/ontology/eden-server/Part> returned: {len(res)} results (expected 0 due to prefix mismatch).")

    # Test with illegal URI character handling (e.g. space in URI)
    print("    [*] Testing spaces in URIs:")
    try:
        # Note: raw spaces are invalid in turtle, but what if they are percent-encoded or parsed?
        invalid_uri_ttl = """
        @prefix eden: <https://ggen.io/ontology/eden-server/> .
        # In RDF/Turtle, space is forbidden in prefix/localname, but can be written in full URI:
        <https://ggen.io/ontology/eden-server/part%20with%20spaces> a eden:Part .
        """
        g2 = Graph()
        g2.parse(data=invalid_uri_ttl, format="turtle")
        print("        [+] Successfully parsed percent-encoded URI with spaces.")
        for s, p, o in g2:
            print(f"            Parsed Subject: {s}")
    except Exception as e:
        print(f"        [-] Failed parsing URI with spaces: {e}")

def stress_test_hierarchy():
    print("\n=== TEST 4: Stress Testing Assembly Tree scale ===")
    
    # 4a. Deep Tree: MechRoot -> Socket -> SubAssembly 1 -> Socket -> SubAssembly 2 -> ... -> SubAssembly N -> Socket -> Part
    print("    [*] Stress testing DEEP tree traversal...")
    for depth in [10, 50, 100, 200]:
        g = load_original_ontologies()
        
        # Build chain
        prev_node = EDEN.mockRoot
        g.add((prev_node, RDF.type, EDEN.MechRoot))
        
        for i in range(1, depth + 1):
            socket = EDEN[f"socket_{i}"]
            subassy = EDEN[f"subAssy_{i}"]
            
            g.add((socket, RDF.type, EDEN.Socket))
            g.add((prev_node, EDEN.hasSocket, socket))
            
            if i == depth:
                # Leaf part
                part = EDEN.leafPart
                g.add((part, RDF.type, EDEN.Part))
                g.add((part, EDEN.plugsInto, socket))
                g.add((part, EDEN.damageClass, Literal(50, datatype=XSD.unsignedByte)))
            else:
                g.add((subassy, RDF.type, EDEN.SubAssembly))
                g.add((subassy, EDEN.plugsInto, socket))
                prev_node = subassy
                
        # Run substrate query
        with open("/Users/sac/.ggen/packs/eden_server/queries/substrate.rq", "r") as f:
            query_str = f.read()
            
        start_time = time.time()
        res = list(g.query(prepareQuery(query_str)))
        elapsed = time.time() - start_time
        print(f"        Depth {depth:4d} | Triples {len(g):5d} | Results {len(res):4d} | Time {elapsed:.4f}s")

    # 4b. Wide Tree: MechRoot -> 5000 Sockets, each holding a SubAssembly or Part
    print("    [*] Stress testing WIDE tree...")
    for width in [100, 1000, 5000]:
        g = load_original_ontologies()
        root = EDEN.mockRoot
        g.add((root, RDF.type, EDEN.MechRoot))
        
        for i in range(width):
            socket = EDEN[f"socket_{i}"]
            part = EDEN[f"part_{i}"]
            
            g.add((socket, RDF.type, EDEN.Socket))
            g.add((root, EDEN.hasSocket, socket))
            g.add((part, RDF.type, EDEN.Part))
            g.add((part, EDEN.plugsInto, socket))
            g.add((part, EDEN.damageClass, Literal(i % 256, datatype=XSD.unsignedByte)))
            
        # Run substrate query
        with open("/Users/sac/.ggen/packs/eden_server/queries/substrate.rq", "r") as f:
            query_str = f.read()
            
        start_time = time.time()
        res = list(g.query(prepareQuery(query_str)))
        elapsed = time.time() - start_time
        print(f"        Width {width:4d} | Triples {len(g):5d} | Results {len(res):4d} | Time {elapsed:.4f}s")

def stress_test_delta_volume():
    print("\n=== TEST 5: Stress Testing Delta Volume ===")
    
    for count in [100, 1000, 5000]:
        g = load_original_ontologies()
        
        # Add 'count' authority deltas, 'count' assembly deltas, and 'count' receipt deltas
        print(f"    [*] Generating {count} of each Delta type...")
        for i in range(count):
            auth_delta = EDEN[f"auth_delta_{i}"]
            g.add((auth_delta, RDF.type, EDEN.AuthorityDelta))
            g.add((auth_delta, EDEN.targetComponent, EDEN.mockPart))
            g.add((auth_delta, EDEN.damageClass, Literal(i % 256, datatype=XSD.unsignedByte)))
            g.add((auth_delta, PROV.generatedAtTime, Literal(f"2026-06-18T12:00:{i%60:02d}Z", datatype=XSD.dateTime)))
            
            assy_delta = EDEN[f"assy_delta_{i}"]
            g.add((assy_delta, RDF.type, EDEN.AssemblyDelta))
            g.add((assy_delta, EDEN.targetSocket, EDEN.mockSocket))
            g.add((assy_delta, EDEN.installedComponent, EDEN.mockPart))
            g.add((assy_delta, PROV.generatedAtTime, Literal(f"2026-06-18T12:00:{i%60:02d}Z", datatype=XSD.dateTime)))
            
            rec_delta = EDEN[f"rec_delta_{i}"]
            g.add((rec_delta, RDF.type, EDEN.ReceiptDelta))
            g.add((rec_delta, EDEN.prompt, Literal(f"Prompt {i}")))
            g.add((rec_delta, EDEN.visualDelta, Literal(0.5 + (i % 50) / 100.0, datatype=XSD.float)))
            g.add((rec_delta, PROV.generatedAtTime, Literal(f"2026-06-18T12:00:{i%60:02d}Z", datatype=XSD.dateTime)))

        # Benchmark authority deltas query
        with open("/Users/sac/.ggen/packs/eden_server/queries/extract_authority_deltas.rq", "r") as f:
            q_auth = f.read()
        start = time.time()
        res_auth = list(g.query(prepareQuery(q_auth)))
        el_auth = time.time() - start
        
        # Benchmark assembly deltas query
        with open("/Users/sac/.ggen/packs/eden_server/queries/extract_assembly_deltas.rq", "r") as f:
            q_assy = f.read()
        start = time.time()
        res_assy = list(g.query(prepareQuery(q_assy)))
        el_assy = time.time() - start
        
        # Benchmark receipt deltas query
        with open("/Users/sac/.ggen/packs/eden_server/queries/extract_receipt_deltas.rq", "r") as f:
            q_rec = f.read()
        start = time.time()
        res_rec = list(g.query(prepareQuery(q_rec)))
        el_rec = time.time() - start
        
        print(f"        Count {count:4d} | Triples {len(g):5d}")
        print(f"          - Auth Deltas Query: {len(res_auth):5d} results in {el_auth:.4f}s")
        print(f"          - Assy Deltas Query: {len(res_assy):5d} results in {el_assy:.4f}s")
        print(f"          - Rec  Deltas Query: {len(res_rec):5d} results in {el_rec:.4f}s")

def test_query_parsing_overhead():
    print("\n=== TEST 6: Query Parsing Overhead ===")
    queries = {
        "substrate": "/Users/sac/.ggen/packs/eden_server/queries/substrate.rq",
        "extract_authority_deltas": "/Users/sac/.ggen/packs/eden_server/queries/extract_authority_deltas.rq",
        "extract_assembly_deltas": "/Users/sac/.ggen/packs/eden_server/queries/extract_assembly_deltas.rq",
        "extract_receipt_deltas": "/Users/sac/.ggen/packs/eden_server/queries/extract_receipt_deltas.rq"
    }
    
    for name, path in queries.items():
        with open(path, "r") as f:
            q_str = f.read()
        
        start = time.time()
        iterations = 100
        for _ in range(iterations):
            q = prepareQuery(q_str)
        el = time.time() - start
        avg_time_ms = (el / iterations) * 1000
        print(f"        Query: {name:<25} | Compilations: {iterations} | Avg Compile Time: {avg_time_ms:.3f} ms")

def main():
    print("======================================================================")
    print("            EDEN SERVER ONTOLOGY & QUERY ADVERSARIAL HARNESS          ")
    print("======================================================================")
    test_datatype_violations()
    test_assembly_tree_cycles()
    test_namespace_conflicts_and_shadowing()
    stress_test_hierarchy()
    stress_test_volume()
    test_query_parsing_overhead()

if __name__ == "__main__":
    # Workaround: map stress_test_volume to stress_test_delta_volume
    stress_test_volume = stress_test_delta_volume
    main()
