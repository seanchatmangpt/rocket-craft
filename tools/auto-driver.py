import subprocess
import os

# Ensure we are in the workspace root
os.chdir("/Users/sac/rocket-craft")

print("--- [AUTOMATED DRIVER] Launching WebGL 2.0 Engine (Headed) ---")
# Use the tps-dflss test with --headed to allow visual observation
# We need to make sure playwright is installed and configured
subprocess.run(["npx", "playwright", "test", "pwa-staff/tests-e2e/tps-dflss.spec.ts", "--headed"])
print("--- [AUTOMATED DRIVER] Simulation Complete ---")
