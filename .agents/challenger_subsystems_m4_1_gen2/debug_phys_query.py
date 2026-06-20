import subprocess
import shutil

CORE_TTL_PATH = "/Users/sac/rocket-craft/ggen-validation-tests/core.ttl"
BACKUP_PATH = CORE_TTL_PATH + ".phys.bak"
GGEN_BIN = "/Users/sac/.local/bin/ggen"

shutil.copy(CORE_TTL_PATH, BACKUP_PATH)

extra_ttl = """
gundam:SimBodyZeroMass a ue4:URigidBody ;
    rdfs:label "SimBodyZeroMass" ;
    ue4:physicsType ue4:PhysType_Simulated ;
    ue4:massKg 0.0 .
"""

with open(CORE_TTL_PATH, "a") as f:
    f.write(extra_ttl)

# Let's test filter NOT EXISTS
q1 = """
PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
SELECT ?this
WHERE {
    ?this a ue4:URigidBody .
    ?this ue4:physicsType ue4:PhysType_Simulated .
    FILTER NOT EXISTS {
        ?this ue4:massKg ?mass .
    }
}
"""

q2 = """
PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
SELECT ?this
WHERE {
    ?this a ue4:URigidBody .
    ?this ue4:physicsType ue4:PhysType_Simulated .
    FILTER NOT EXISTS {
        ?this ue4:massKg ?mass .
        FILTER (?mass > 0.0)
    }
}
"""

try:
    res1 = subprocess.run([GGEN_BIN, "graph", "query", "--graph-file", CORE_TTL_PATH, "--sparql-query", q1], capture_output=True, text=True)
    res2 = subprocess.run([GGEN_BIN, "graph", "query", "--graph-file", CORE_TTL_PATH, "--sparql-query", q2], capture_output=True, text=True)
    print("Q1 (FILTER NOT EXISTS { ?this ue4:massKg ?mass }):")
    print(res1.stdout)
    print("Q2 (FILTER NOT EXISTS { ?this ue4:massKg ?mass . FILTER (?mass > 0.0) }):")
    print(res2.stdout)
    
finally:
    shutil.copy(BACKUP_PATH, CORE_TTL_PATH)
    import os
    os.remove(BACKUP_PATH)
