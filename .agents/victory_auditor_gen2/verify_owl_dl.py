import os
import sys
from rdflib import Graph, RDF, RDFS, OWL, Namespace, BNode, URIRef

def check_owl_dl(folders):
    g = Graph()
    
    # Load all turtle files
    print("Loading Turtle files...")
    loaded_files = []
    for folder in folders:
        for root, dirs, files in os.walk(folder):
            for file in files:
                if file.endswith('.ttl'):
                    file_path = os.path.join(root, file)
                    # Skip backup files
                    if '.backup' in file or '~' in file:
                        continue
                    print(f"  Loading {file_path}")
                    try:
                        g.parse(file_path, format="turtle")
                        loaded_files.append(file_path)
                    except Exception as e:
                        print(f"Error parsing {file_path}: {e}")
                        return False, f"Syntax error in {file_path}: {e}"

    print(f"Loaded {len(loaded_files)} files. Total triples: {len(g)}")
    
    violations = []
    
    # Check 1: Property Punning (same IRI as ObjectProperty and DatatypeProperty)
    obj_props = set(g.subjects(RDF.type, OWL.ObjectProperty))
    data_props = set(g.subjects(RDF.type, OWL.DatatypeProperty))
    ann_props = set(g.subjects(RDF.type, OWL.AnnotationProperty))
    
    punning_props = obj_props.intersection(data_props)
    if punning_props:
        violations.append(f"Property punning detected (ObjectProperty and DatatypeProperty): {punning_props}")
        
    punning_props2 = obj_props.intersection(ann_props)
    if punning_props2:
        violations.append(f"Property punning detected (ObjectProperty and AnnotationProperty): {punning_props2}")
        
    punning_props3 = data_props.intersection(ann_props)
    if punning_props3:
        violations.append(f"Property punning detected (DatatypeProperty and AnnotationProperty): {punning_props3}")
        
    # Check 2: Class/Property punning (an IRI cannot be both Class and Property)
    classes = set(g.subjects(RDF.type, OWL.Class))
    all_props = obj_props.union(data_props).union(ann_props)
    class_prop_punning = classes.intersection(all_props)
    # Exclude standard properties if any, but none should be punned
    if class_prop_punning:
        violations.append(f"Class/Property punning detected (Class and Property): {class_prop_punning}")

    # Check 3: Every predicate used in triples (excluding standard namespaces) must have a type declaration
    standard_ns_prefixes = [
        "http://www.w3.org/1999/02/22-rdf-syntax-ns#",
        "http://www.w3.org/2000/01/rdf-schema#",
        "http://www.w3.org/2002/07/owl#",
        "http://www.w3.org/2001/XMLSchema#",
        "http://www.w3.org/ns/shacl#",
        "http://www.w3.org/ns/prov#",
        "http://www.w3.org/ns/sosa/",
        "http://qudt.org/schema/qudt/",
        "http://purl.org/dc/terms/",
        "http://www.w3.org/2004/02/skos/core#",
        "https://spec.edmcouncil.org/fibo/"
    ]
    
    def is_standard(uri):
        if isinstance(uri, BNode):
            return True
        uri_str = str(uri)
        return any(uri_str.startswith(pfx) for pfx in standard_ns_prefixes)

    # Check properties used as predicates
    used_predicates = set(g.predicates())
    for pred in used_predicates:
        if not is_standard(pred):
            if pred not in all_props:
                violations.append(f"Property {pred} is used as a predicate but not declared as owl:ObjectProperty, owl:DatatypeProperty, or owl:AnnotationProperty in local ontology files.")

    # Check classes used in subclass relations or domains/ranges
    used_classes = set()
    for s, p, o in g:
        if p == RDFS.subClassOf:
            used_classes.add(s)
            used_classes.add(o)
        elif p == RDFS.domain or p == RDFS.range:
            if isinstance(o, URIRef):
                if not str(o).startswith("http://www.w3.org/2002/07/owl#"):
                    used_classes.add(o)
                    
    for cls in used_classes:
        if not is_standard(cls):
            if cls not in classes:
                violations.append(f"Class {cls} is referenced but not declared as owl:Class in local ontology files.")

    # Check 4: Transitive/Symmetric property restrictions (Transitive properties cannot be functional)
    transitive_props = set(g.subjects(RDF.type, OWL.TransitiveProperty))
    functional_props = set(g.subjects(RDF.type, OWL.FunctionalProperty))
    trans_func = transitive_props.intersection(functional_props)
    if trans_func:
        violations.append(f"OWL 2 DL violation: Transitive property cannot be Functional: {trans_func}")

    # Check 5: Symmetric property domain and range must be identical or compatible
    symmetric_props = set(g.subjects(RDF.type, OWL.SymmetricProperty))
    for sp in symmetric_props:
        domains = list(g.objects(sp, RDFS.domain))
        ranges = list(g.objects(sp, RDFS.range))
        if domains and ranges and domains != ranges:
            violations.append(f"OWL 2 DL warning/violation: Symmetric property {sp} has mismatched domain {domains} and range {ranges}")

    if violations:
        print("\nOWL 2 DL Violations found:")
        for v in violations:
            print(f"  - {v}")
        return False, violations
    else:
        print("\nStrict OWL 2 DL Static Analysis PASS.")
        return True, "No OWL 2 DL violations detected"

if __name__ == "__main__":
    folders = [
        "/Users/sac/.ggen/packs/eden_server/ontology",
        "/Users/sac/.ggen/packs/ue4_ontology"
    ]
    success, result = check_owl_dl(folders)
    if not success:
        sys.exit(1)
    sys.exit(0)
