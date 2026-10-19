# Traypack roadmap

Traypack begins with an original CSV-to-SVG shelf-packing implementation. These
are estimates for future work, not completed sessions or claims of adoption.

| Work | Acceptance criteria | Estimate |
| --- | --- | --- |
| Better packing strategies | Add a second rectangle-packing heuristic; compare tray counts on original synthetic inventories; retain deterministic output | 7–10 hours |
| Adjustable inventory workflow | Named inventories, versioned save/load, reorder and edit operations, clear validation | 5–8 hours |
| Multi-page printing | A4/Letter layouts with per-tray pages, overlap marks, and measured physical output | 6–9 hours |
| Divider and material allowances | Separate component footprint, finger clearance, wall thickness, and usable internal dimensions; verify with physical prototypes | 7–11 hours |
| SVG editing and interchange | Export stable component identifiers and machine-readable placement data; preserve units on round trips | 4–6 hours |
| Release readiness | Cross-platform builds, older-toolchain checks, usability improvements, and documented sample assemblies | 4–6 hours |

**Estimated remaining work: 33–50 hours.** The application uses component names
and dimensions, not customer records or AI model infrastructure. Any performance
comparisons concern the packing algorithms, not AI models.
