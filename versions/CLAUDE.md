# versions — CLAUDE.md

## Purpose

Reference UE4 project snapshots used as templates, sample content, and baseline
configs for the Rocket Craft monorepo. Contains 5 Unreal Engine 4 projects (3.2 GB+,
mostly binary `.uasset`/`.umap` files). These are read-only reference projects —
never edit binary assets directly. All asset operations must go through the UE4 Editor
or `RunUAT.sh`.

## Directory Structure

```
versions/
├── 4.24.0/                          # "Brm" vehicle game (primary project)
│   ├── Brm.uproject
│   ├── Brm.png
│   ├── Config/
│   │   ├── DefaultEngine.ini
│   │   ├── DefaultGame.ini
│   │   ├── DefaultInput.ini
│   │   ├── DefaultEditor.ini
│   │   ├── DefaultCrypto.ini
│   │   ├── DefaultDeviceProfiles.ini
│   │   ├── DefaultEditorPerProjectUserSettings.ini
│   │   ├── DefaultGameplayTags.ini
│   │   └── DefaultPixelStreaming.ini
│   ├── Content/
│   │   ├── Environment/             # Landscape, skybox assets
│   │   ├── Fonts/
│   │   ├── Geometry/                # Static mesh assets
│   │   ├── Maps/                    # .umap level files
│   │   ├── VehicleAdv/              # Vehicle Blueprint assets
│   │   └── VehicleAdvBP/
│   ├── Platforms/
│   │   └── HTML5/                   # HTML5 platform overrides
│   ├── Source/
│   │   ├── Brm/                     # C++ game module source
│   │   ├── Brm.Target.cs            # Game target build rules
│   │   ├── BrmEditor.Target.cs      # Editor target build rules
│   │   └── BrmServer.Target.cs      # Server target build rules
│   └── README.md
│
├── 4.24-Shooter/                    # ShooterGame (Epic sample: networked FPS)
│   └── ShooterGame/
│       ├── ShooterGame.uproject
│       ├── Config/
│       ├── Content/                 # Binary .uasset files — do not edit directly
│       ├── Platforms/
│       ├── Source/                  # C++ source for ShooterGame
│       ├── ShooterGame.png
│       └── README.md
│
├── 4.24-Survival/                   # SurvivalGame (Epic sample: survival game)
│   └── EpicSurvivalGameSeries-4.24/
│       ├── LICENSE
│       ├── README.md
│       └── SurvivalGame/
│           └── (standard UE4 layout)
│
├── Realistic/                       # RealisticRendering (Epic showcase)
│   └── RealisticRendering/
│       ├── RealisticRendering.uproject
│       ├── Config/
│       ├── Content/                 # High-fidelity rendering showcase assets
│       ├── Platforms/
│       ├── RealisticRendering.png
│       └── README.md
│
└── Template/                        # FullSpectrum (game template)
    └── FullSpectrum/
        ├── FullSpectrum.uproject
        ├── Config/
        ├── Content/
        └── README.md
```

## UE4 Projects Summary

| Directory          | Project Name        | Engine | Targets                          | Notes                              |
|--------------------|---------------------|--------|----------------------------------|------------------------------------|
| `4.24.0/`          | Brm                 | 4.24   | Brm, BrmEditor, BrmServer        | Vehicle game; HTML5/mobile/WebSocket multiplayer |
| `4.24-Shooter/`    | ShooterGame         | 4.24   | Editor, Client, Game, Server     | Epic networked FPS sample           |
| `4.24-Survival/`   | SurvivalGame        | 4.24   | Editor, Server, Game             | Epic survival RPG sample            |
| `Realistic/`       | RealisticRendering  | 4.24   | (showcase, no game targets)      | Rendering quality showcase          |
| `Template/`        | FullSpectrum        | 4.24   | (template, no game targets)      | Starter template project            |

## Key Commands

All commands require a UE4 installation. Set `UE4_ROOT` to your engine path.

```bash
export UE4_ROOT=/path/to/UnrealEngine-4.24

# Open Brm in UE4 Editor
"$UE4_ROOT/Engine/Binaries/Linux/UE4Editor" \
  "$(pwd)/4.24.0/Brm.uproject"

# Open ShooterGame in UE4 Editor
"$UE4_ROOT/Engine/Binaries/Linux/UE4Editor" \
  "$(pwd)/4.24-Shooter/ShooterGame/ShooterGame.uproject"

# Build Brm (Development)
"$UE4_ROOT/Engine/Build/BatchFiles/RunUAT.sh" BuildCookRun \
  -project="$(pwd)/4.24.0/Brm.uproject" \
  -noP4 -platform=Win64 -clientconfig=Development -build

# Cook Brm for HTML5
"$UE4_ROOT/Engine/Build/BatchFiles/RunUAT.sh" BuildCookRun \
  -project="$(pwd)/4.24.0/Brm.uproject" \
  -noP4 -platform=HTML5 -clientconfig=Shipping \
  -build -cook -stage -package

# Cook ShooterGame (Server + Client)
"$UE4_ROOT/Engine/Build/BatchFiles/RunUAT.sh" BuildCookRun \
  -project="$(pwd)/4.24-Shooter/ShooterGame/ShooterGame.uproject" \
  -noP4 -platform=Win64 -serverconfig=Development -build -cook -stage

# Validate assets (Rust command at monorepo root)
cd /home/user/rocket-craft && ./rocket test

# Rebuild C++ (Brm module only, no cook)
"$UE4_ROOT/Engine/Build/BatchFiles/Linux/Build.sh" \
  Brm Linux Development \
  "$(pwd)/4.24.0/Brm.uproject" -waitmutex
```

## Config Files (4.24.0/Config/)

| File                                    | Key Settings                                         |
|-----------------------------------------|------------------------------------------------------|
| `DefaultEngine.ini`                     | Renderer settings, net drivers, plugin enable/disable |
| `DefaultGame.ini`                       | `GameMode`, supported maps, max players               |
| `DefaultInput.ini`                      | Input axis/action mappings                            |
| `DefaultCrypto.ini`                     | Encryption key for cooked content                    |
| `DefaultDeviceProfiles.ini`             | Per-device scalability (mobile LOD, shadows)         |
| `DefaultPixelStreaming.ini`             | Pixel streaming server config (HTML5/WebRTC)         |

Edit `.ini` files with a text editor — they are plain text. Never use a binary editor.

## Relation to the Monorepo

- **`project-manifest.json`** (monorepo root) — lists all these projects with their
  targets and platforms. `tools/rocket-sdk/manifest.rs` reads this file.
- **`pwa-staff/`** — the HTML5 shipping builds (`*-HTML5-Shipping.*`) are produced
  from these projects and hosted there.
- **`tools/rocket-cmd`** — `rocket-cmd build` targets these projects via the manifest.
- **`asset-pipeline/`** — staged FBX assets from the pipeline are imported into
  these projects' `Content/` directories via the UE4 Editor.
- **`unify-rs/unify-rdf`** (`project_bridge.rs`) — maps these projects' manifest
  entries into RDF triples for semantic queries.

## CRITICAL: Binary Asset Rules

**DO NOT:**
- Edit `.uasset` or `.umap` files with a text editor or hex editor
- Run `git add Content/` without checking file sizes first
- Merge binary `.uasset` files manually — they are not diff-mergeable
- Delete `.uasset` files outside the UE4 Editor (breaks cross-references)
- Add large binary files (>50 MB) to git without Git LFS

**DO:**
- Use the UE4 Editor for all Content Browser operations
- Use `RunUAT.sh` for building, cooking, and packaging
- Use Git LFS for `.uasset`, `.umap`, `.udk`, `.upk` files
- Check `git lfs status` before committing any Content changes

## Caveats and Gotchas

- **UE4 4.24 only**: these projects use engine version 4.24. Opening them in 4.25+
  will trigger an auto-upgrade that modifies binary assets — do this intentionally,
  not accidentally.
- **Total size is 3.2 GB+**: `git clone` with `--depth 1` if you only need the
  source code. Full history is very large.
- **Brm HTML5 requires Emscripten**: the HTML5 cooking step needs a compatible
  Emscripten SDK. UE4 4.24 ships with a bundled version at
  `$UE4_ROOT/Engine/Extras/ThirdPartyNotUE/emsdk/`.
- **ShooterGame network**: requires the Unreal Network Stack (OnlineSubsystem NULL
  for LAN). For internet multiplayer, configure `OnlineSubsystemSteam` in
  `DefaultEngine.ini`.
- **SurvivalGame is licensed separately**: `4.24-Survival/EpicSurvivalGameSeries-4.24/LICENSE`
  — check Epic's license before redistributing.
- **RealisticRendering and FullSpectrum have no C++ modules**: they are Blueprint/content-only
  projects. Do not add a `Source/` directory without the corresponding `.Build.cs`.
- **`DefaultCrypto.ini` contains encryption keys**: never commit real encryption keys.
  The file in this repo uses placeholder/development keys only.
