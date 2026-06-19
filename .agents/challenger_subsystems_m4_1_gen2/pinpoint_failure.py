import shutil
import os
import subprocess

TOML_PATH = "/Users/sac/rocket-craft/ggen-validation-tests/ggen.toml"
TOML_BACKUP = TOML_PATH + ".debug2.bak"
GGEN_BIN = "/Users/sac/.local/bin/ggen"

PREFIXES = """
PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
"""

QUERIES_TO_TEST = [
    # Test 1: Simple check if gundam:GundamWorld has type ue4:UWorld
    """
    ASK {
      FILTER NOT EXISTS {
        ?world a ue4:UWorld .
      }
    }
    """,
    # Test 2: Check if world has level
    """
    ASK {
      FILTER NOT EXISTS {
        ?world a ue4:UWorld ;
               ue4:hasLevel ?level .
      }
    }
    """,
    # Test 3: Check if world has level and level has actor
    """
    ASK {
      FILTER NOT EXISTS {
        ?world a ue4:UWorld ;
               ue4:hasLevel ?level .
        ?level ue4:hasActor ?actor .
      }
    }
    """,
    # Test 4: Check if actor is replicated
    """
    ASK {
      FILTER NOT EXISTS {
        ?world a ue4:UWorld ;
               ue4:hasLevel ?level .
        ?level ue4:hasActor ?actor .
        ?actor ue4:bReplicates true .
      }
    }
    """,
    # Test 5: Check if world has subsystem (any subsystem)
    """
    ASK {
      FILTER NOT EXISTS {
        ?world a ue4:UWorld ;
               ue4:hasLevel ?level .
        ?level ue4:hasActor ?actor .
        ?actor ue4:bReplicates true .
        FILTER NOT EXISTS {
          ?world ue4:hasSubsystem ?subsystem .
        }
      }
    }
    """,
    # Test 6: Check type of subsystem
    """
    ASK {
      FILTER NOT EXISTS {
        ?world a ue4:UWorld ;
               ue4:hasLevel ?level .
        ?level ue4:hasActor ?actor .
        ?actor ue4:bReplicates true .
        FILTER NOT EXISTS {
          ?world ue4:hasSubsystem ?subsystem .
          ?subsystem a ue4:UNetworkingSubsystem .
        }
      }
    }
    """
]

shutil.copy(TOML_PATH, TOML_BACKUP)

try:
    for idx, q in enumerate(QUERIES_TO_TEST, 1):
        # Read backup
        with open(TOML_BACKUP) as f:
            content = f.read()
            
        # Locate RuleNetWorldSubsystemTopology rule
        # We need to replace the ask field
        # RuleNetWorldSubsystemTopology:
        # ask = \"\"\"...\"\"\"
        # Let's replace the whole ask content
        start_idx = content.find('name = "RuleNetWorldSubsystemTopology"')
        if start_idx == -1:
            print("Could not find rule!")
            break
            
        ask_start = content.find('ask = """', start_idx) + len('ask = """')
        ask_end = content.find('"""', ask_start)
        
        full_query = PREFIXES + q
        content_mod = content[:ask_start] + full_query + content[ask_end:]
        
        with open(TOML_PATH, "w") as f:
            f.write(content_mod)
            
        # Run ggen sync
        res = subprocess.run(
            [GGEN_BIN, "sync", "--manifest", TOML_PATH, "--validate-only", "true"],
            capture_output=True, text=True
        )
        
        failed = "RuleNetWorldSubsystemTopology" in (res.stdout + res.stderr)
        print(f"Test {idx} failed (RuleNetWorldSubsystemTopology is in output): {failed}")
        if failed:
            # Print the relevant failure output
            print("  Output preview:")
            for line in (res.stdout + res.stderr).splitlines():
                if "RuleNetWorldSubsystemTopology" in line or "failed" in line:
                    print(f"    {line}")
        
finally:
    if os.path.exists(TOML_BACKUP):
        shutil.copy(TOML_BACKUP, TOML_PATH)
        os.remove(TOML_BACKUP)
