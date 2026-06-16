# Realistic Rendering - RocketCraft

> **Note:** This project is built and managed using the `./rocket` CLI tool.

## Purpose
An architectural visualization demo titled **Full power of ue4 html5 extension**. It is designed to showcase the high-fidelity rendering capabilities of Unreal Engine 4, specifically optimized for realistic environments and potentially for web-based delivery via HTML5.

## Target Platforms
- **Windows / Mac**: For high-end desktop rendering.
- **HTML5**: Specifically mentioned in the project description as a target for extension.

## Unique Config Settings
- **Rendering Quality**: 
  - **Temporal AA** and **SSR** (Screen Space Reflections) are explicitly enabled in system settings.
  - Custom `LightComplexityColors` and `ShaderComplexityColors` for refined editor visualization.
- **Near Clip Plane**: Set to **2.0** to allow for very close inspection of high-detail assets.
- **Packaging**: Configured for **Shipping** and **For Distribution**, indicating it is ready for end-user deployment.
- **Maps**: 
  - Primary Scene: `Room`
- **UI**: Slate settings configured with explicit canvas child Z-order disabled.
