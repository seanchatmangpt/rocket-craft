#!/usr/bin/env python3
import os
import sys

def check_plugin(engine_path, plugin_name, possible_rel_paths):
    """
    Checks if a plugin exists in any of the possible relative paths within the engine.
    Verifies the existence of the .uplugin file.
    """
    found = False
    for rel_path in possible_rel_paths:
        plugin_dir = os.path.join(engine_path, "Engine", "Plugins", rel_path)
        uplugin_file = os.path.join(plugin_dir, f"{plugin_name}.uplugin")
        
        if os.path.exists(uplugin_file):
            print(f"[OK] Found {plugin_name} at: {plugin_dir}")
            found = True
            break
        elif os.path.exists(plugin_dir):
            # If directory exists but .uplugin is missing or differently named
            print(f"[WARN] Directory exists but {plugin_name}.uplugin not found in: {plugin_dir}")
    
    if not found:
        expected_locations = [os.path.join(engine_path, "Engine", "Plugins", p) for p in possible_rel_paths]
        print(f"[ERROR] {plugin_name} NOT found or invalid. Checked locations:")
        for loc in expected_locations:
            print(f"  - {loc}")
    
    return found

def main():
    # Priority: 1. Command line argument, 2. UE_ROOT env var, 3. Current directory
    if len(sys.argv) > 1:
        engine_path = sys.argv[1]
    else:
        engine_path = os.environ.get("UE_ROOT", os.getcwd())
    
    engine_path = os.path.abspath(engine_path)
    print(f"--- Rocket Craft Dependency Checker ---")
    print(f"Target Engine Path: {engine_path}\n")

    # Common locations for these plugins in UE4.24
    dependencies = {
        "WebSocketNetworking": [
            "Runtime/Networking/WebSocketNetworking",
            "Networking/WebSocketNetworking",
            "WebSocketNetworking"
        ],
        "VaRest": [
            "Marketplace/VaRest",
            "VaRest"
        ]
    }

    all_ok = True
    for plugin_name, paths in dependencies.items():
        if not check_plugin(engine_path, plugin_name, paths):
            all_ok = False
        print()

    if all_ok:
        print("Status: SUCCESS - All dependencies present.")
        sys.exit(0)
    else:
        print("Status: FAILED - Some dependencies are missing.")
        print("Please ensure you have the UE4.24.3-HTML5 engine build with these plugins installed.")
        sys.exit(1)

if __name__ == "__main__":
    main()
