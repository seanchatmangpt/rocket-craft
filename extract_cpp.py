import rdflib
import os
import re

TTL_PATH = "ontology/all_merged.ttl"
OUT_DIR = "generated/mech_assets/reference_fabric_001/cpp_bridge"

os.makedirs(OUT_DIR, exist_ok=True)

g = rdflib.Graph()
g.parse(TTL_PATH, format="turtle")

SH = rdflib.Namespace("http://www.w3.org/ns/shacl#")

# Find all NodeShapes
shapes = list(g.subjects(rdflib.RDF.type, SH.NodeShape))

def clean_name(uri):
    # Get fragment or last part of path
    if "#" in uri:
        name = uri.split("#")[-1]
    else:
        name = uri.split("/")[-1]
    # Remove any non-alphanumeric characters
    name = re.sub(r'\W|^(?=\d)', '_', name)
    return name

cpp_header = """#pragma once

#include "CoreMinimal.h"
#include "Engine/DataTable.h"
#include "OntologyGeneratedTypes.generated.h"

"""

structs_code = ""

for s in shapes:
    struct_name = clean_name(str(s))
    
    properties = list(g.objects(s, SH.property))
    
    struct_code = f"USTRUCT(BlueprintType)\nstruct F{struct_name} : public FTableRowBase\n{{\n\tGENERATED_BODY()\n\n"
    
    if len(properties) == 0:
        struct_code += "\t// No SHACL properties defined\n"
    
    for p in properties:
        path = g.value(p, SH.path)
        datatype = g.value(p, SH.datatype)
        min_inc = g.value(p, SH.minInclusive)
        max_inc = g.value(p, SH.maxInclusive)
        
        prop_name = clean_name(str(path)) if path else "UnknownProp"
        
        # map datatype to C++ type
        cpp_type = "float"
        if datatype == rdflib.XSD.integer:
            cpp_type = "int32"
        elif datatype == rdflib.XSD.boolean:
            cpp_type = "bool"
        elif datatype == rdflib.XSD.string:
            cpp_type = "FString"
            
        struct_code += f"\tUPROPERTY(EditAnywhere, BlueprintReadWrite, Category=\"Ontology\")\n"
        struct_code += f"\t{cpp_type} {prop_name};\n\n"
        
        # Add min/max as comments or metadata if present
        if min_inc is not None or max_inc is not None:
            struct_code += f"\t// Constraints: Min={min_inc}, Max={max_inc}\n"
            
    struct_code += "};\n\n"
    structs_code += struct_code

full_code = cpp_header + structs_code

with open(os.path.join(OUT_DIR, "OntologyGeneratedTypes.h"), "w") as f:
    f.write(full_code)

print(f"Generated {len(shapes)} shapes to OntologyGeneratedTypes.h")
