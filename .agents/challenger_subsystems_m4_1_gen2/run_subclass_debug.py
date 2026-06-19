import subprocess
import shutil
import os

CORE_TTL_PATH = "/Users/sac/rocket-craft/ggen-validation-tests/core.ttl"
BACKUP_PATH = CORE_TTL_PATH + ".debug.bak"
GGEN_BIN = "/Users/sac/.local/bin/ggen"
MANIFEST_PATH = "/Users/sac/rocket-craft/ggen-validation-tests/ggen.toml"

shutil.copy(CORE_TTL_PATH, BACKUP_PATH)

def run_val(extra=""):
    shutil.copy(BACKUP_PATH, CORE_TTL_PATH)
    if extra:
        with open(CORE_TTL_PATH, "a") as f:
            f.write(extra)
    res = subprocess.run(
        [GGEN_BIN, "sync", "--manifest", MANIFEST_PATH, "--validate-only", "true"],
        capture_output=True, text=True
    )
    return res.returncode, res.stdout + "\n" + res.stderr

try:
    # 1. Run baseline
    code, out = run_val()
    print("Baseline result:")
    print("RuleNetWorldSubsystemTopology in output:", "RuleNetWorldSubsystemTopology" in out)
    
    # 2. Add self-subclass for UNetworkingSubsystem
    code, out = run_val("\nue4:UNetworkingSubsystem rdfs:subClassOf ue4:UNetworkingSubsystem .\n")
    print("Self-subclass result:")
    print("RuleNetWorldSubsystemTopology in output:", "RuleNetWorldSubsystemTopology" in out)

    # 3. Add explicit type: UNetworkingSubsystem a owl:Class
    # (it is already in subsystems.ttl)
    
    # 4. What if we just change the query? But we can't easily change query without editing ggen.toml.
    # Let's see what happens if we make GundamNetworkingHandler have type UNetworkingSubsystem,
    # and also hasSubsystem in GundamWorld.
    
finally:
    if os.path.exists(BACKUP_PATH):
        shutil.copy(BACKUP_PATH, CORE_TTL_PATH)
        os.remove(BACKUP_PATH)
