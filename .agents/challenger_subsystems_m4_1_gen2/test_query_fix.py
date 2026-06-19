import shutil
import os
import subprocess

TOML_PATH = "/Users/sac/rocket-craft/ggen-validation-tests/ggen.toml"
TOML_BACKUP = TOML_PATH + ".debug.bak"
GGEN_BIN = "/Users/sac/.local/bin/ggen"

shutil.copy(TOML_PATH, TOML_BACKUP)

try:
    # Read TOML
    with open(TOML_PATH) as f:
        content = f.read()
        
    # Replace query in RuleNetWorldSubsystemTopology
    old_query = "?subsystem a/rdfs:subClassOf* ue4:UNetworkingSubsystem ."
    new_query = "?subsystem a ue4:UNetworkingSubsystem ."
    
    if old_query in content:
        print("Found old query. Replacing...")
        content_mod = content.replace(old_query, new_query)
        with open(TOML_PATH, "w") as f:
            f.write(content_mod)
            
        # Run ggen sync
        res = subprocess.run(
            [GGEN_BIN, "sync", "--manifest", TOML_PATH, "--validate-only", "true"],
            capture_output=True, text=True
        )
        print("Modified rule validation output contains RuleNetWorldSubsystemTopology:", "RuleNetWorldSubsystemTopology" in (res.stdout + res.stderr))
    else:
        print("Could not find old query in TOML!")
        
finally:
    if os.path.exists(TOML_BACKUP):
        shutil.copy(TOML_BACKUP, TOML_PATH)
        os.remove(TOML_BACKUP)
