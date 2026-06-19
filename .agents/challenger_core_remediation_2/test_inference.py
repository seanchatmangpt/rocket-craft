import os
import sys
from rdflib import Graph, Namespace, URIRef
from rdflib.namespace import RDF, RDFS, OWL

def main():
    ontology_dir = "/Users/sac/.ggen/packs/ue4_ontology"
    ttl_files = [
        "core.ttl",
        "reflection.ttl",
        "blueprints.ttl",
        "subsystems.ttl",
        "typestates.ttl"
    ]
    
    g = Graph()
    
    # Load all turtle files
    for ttl in ttl_files:
        path = os.path.join(ontology_dir, ttl)
        if not os.path.exists(path):
            print(f"Error: {path} does not exist.")
            sys.exit(1)
        g.parse(path, format="turtle")
        
    print(f"Loaded base ontology. Total triples: {len(g)}")
    
    # Define Namespace
    UE4 = Namespace("https://rocket-craft.io/ontology/ue4/")
    EX = Namespace("https://rocket-craft.io/example/")
    g.bind("ue4", UE4)
    g.bind("ex", EX)
    g.bind("rdfs", RDFS)
    g.bind("owl", OWL)
    g.bind("rdf", RDF)
    
    # Add instances
    print("\n--- 1. Testing component inference (hasComponent -> isComponentOf) ---")
    actor_uri = EX.MyActor
    comp_uri = EX.MyComponent
    g.add((actor_uri, RDF.type, UE4.AActor))
    g.add((comp_uri, RDF.type, UE4.UActorComponent))
    g.add((actor_uri, UE4.hasComponent, comp_uri))
    
    print("Added: ex:MyActor a ue4:AActor .")
    print("Added: ex:MyComponent a ue4:UActorComponent .")
    print("Added: ex:MyActor ue4:hasComponent ex:MyComponent .")
    
    # Run ask condition
    ask_comp = """
    PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
    ASK {
      ?actor ue4:hasComponent ?component .
    }
    """
    ask_result = g.query(ask_comp)
    print(f"Inference ASK condition result: {bool(ask_result)}")
    
    # Run construct rule
    construct_comp = """
    PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
    CONSTRUCT {
      ?component ue4:isComponentOf ?actor .
    } WHERE {
      ?actor ue4:hasComponent ?component .
    } ORDER BY ?actor ?component
    """
    constructed_triples = g.query(construct_comp)
    print("Constructed triples:")
    for triple in constructed_triples:
        print(f"  {triple[0].n3(g.namespace_manager)} {triple[1].n3(g.namespace_manager)} {triple[2].n3(g.namespace_manager)}")
        # Add to graph
        g.add(triple)
        
    # Verify isComponentOf exists
    has_inferred = (comp_uri, UE4.isComponentOf, actor_uri) in g
    print(f"Verification of ex:MyComponent ue4:isComponentOf ex:MyActor: {'PASSED' if has_inferred else 'FAILED'}")
    
    # Add level instances
    print("\n--- 2. Testing level inference (hasLevel -> isLevelOf) ---")
    world_uri = EX.MyWorld
    level_uri = EX.MyLevel
    g.add((world_uri, RDF.type, UE4.UWorld))
    g.add((level_uri, RDF.type, UE4.ULevel))
    g.add((world_uri, UE4.hasLevel, level_uri))
    
    print("Added: ex:MyWorld a ue4:UWorld .")
    print("Added: ex:MyLevel a ue4:ULevel .")
    print("Added: ex:MyWorld ue4:hasLevel ex:MyLevel .")
    
    # Run ask condition
    ask_level = """
    PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
    ASK {
      ?world ue4:hasLevel ?level .
    }
    """
    ask_result_level = g.query(ask_level)
    print(f"Inference ASK condition result: {bool(ask_result_level)}")
    
    # Run construct rule
    construct_level = """
    PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
    CONSTRUCT {
      ?level ue4:isLevelOf ?world .
    } WHERE {
      ?world ue4:hasLevel ?level .
    } ORDER BY ?world ?level
    """
    constructed_triples_level = g.query(construct_level)
    print("Constructed triples:")
    for triple in constructed_triples_level:
        print(f"  {triple[0].n3(g.namespace_manager)} {triple[1].n3(g.namespace_manager)} {triple[2].n3(g.namespace_manager)}")
        # Add to graph
        g.add(triple)
        
    # Verify isLevelOf exists
    has_inferred_level = (level_uri, UE4.isLevelOf, world_uri) in g
    print(f"Verification of ex:MyLevel ue4:isLevelOf ex:MyWorld: {'PASSED' if has_inferred_level else 'FAILED'}")

if __name__ == "__main__":
    main()
