import rdflib
import sys

def main():
    g = rdflib.Graph()
    files = [
        "/Users/sac/rocket-craft/ggen-validation-tests/core.ttl",
        "/Users/sac/rocket-craft/ggen-validation-tests/blueprints.ttl",
        "/Users/sac/rocket-craft/ggen-validation-tests/subsystems.ttl",
        "/Users/sac/rocket-craft/ggen-validation-tests/reflection.ttl",
        "/Users/sac/rocket-craft/ggen-validation-tests/typestates.ttl"
    ]
    for f in files:
        g.parse(f, format="turtle")
    
    print(f"Loaded {len(g)} triples.")
    
    # Let's run a query to find UK2Node subclasses and their input exec pins
    q = """
    PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
    PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
    
    SELECT ?node ?pin ?dir ?cat WHERE {
        ?pin ue4:pinOf ?node .
        ?node a/rdfs:subClassOf* ue4:UK2Node .
        ?pin ue4:pinDirection ?dir .
        ?pin ue4:pinCategory ?cat .
    }
    """
    print("All input/output pins on UK2Node nodes:")
    for r in g.query(q):
        print(f"Node: {r.node} | Pin: {r.pin} | Dir: {r.dir} | Cat: {r.cat}")
        
    # Let's see which pins have connectedTo
    q_connected = """
    PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
    SELECT ?pin ?other WHERE {
        ?pin ue4:connectedTo ?other .
    }
    """
    print("\nAll connections:")
    for r in g.query(q_connected):
        print(f"Pin: {r.pin} -> Other: {r.other}")
        
    # Let's run the exact RuleH check
    q_ruleh = """
    PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
    PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
    
    SELECT ?pin ?node WHERE {
        ?pin ue4:pinOf ?node .
        ?node a/rdfs:subClassOf* ue4:UK2Node .
        ?pin ue4:pinDirection ue4:Input .
        ?pin ue4:pinCategory "exec" .
        FILTER NOT EXISTS { ?pin (ue4:connectedTo|^ue4:connectedTo|ue4:linkedTo|^ue4:linkedTo) ?other }
    }
    """
    print("\nUnconnected input exec pins matching RuleH:")
    results = list(g.query(q_ruleh))
    if not results:
        print("None! Graph is valid according to rdflib.")
    for r in results:
        print(f"Pin: {r.pin} on Node: {r.node}")

if __name__ == "__main__":
    main()
