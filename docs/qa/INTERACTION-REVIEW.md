# Agent Pipeline interaction review

Date: 2026-08-16

## Scope

The arm64 app was rebuilt, ad-hoc signed, verified with `codesign --deep --strict`, and installed at `/Applications/Agent Pipeline.app`. The interaction matrix below was exercised against the same compiled Vue surface on `127.0.0.1:1420` because the macOS Computer Use capture service returned `cgWindowNotFound` for Agent Pipeline, Finder, and Chrome during the installed-app evidence pass. These screenshots are therefore useful UI regression evidence, but they are not represented as installed-app proof.

## Passed interactions

- Five-step onboarding and Environment Doctor.
- Graph selection for all node kinds; selected-node Activity lane no longer intersects history.
- Inspector Overview, Activity, Artifacts, Logs and Node Focus.
- Review feedback creates immutable Implement Attempt 2; approval advances Deploy, Smoke Test, and 7/7 completion.
- Definition map, source/protocol concepts, execution contract, inputs, outputs, context, sandbox, policy, Skills, exact MCP tool grants, and capabilities.
- Natural-language “让模型修改此 Node” route, Package Proposal, permission expansion, source reveal feedback, validation/install states.
- Deliverable slots, empty release slot, provenance, revision metadata and JSON snapshot export.
- Attention critical-only filter, Help, More menu, Mission Control, reset, Doctor re-entry.
- System, Draft Light, Night Ops and Warm Paper theme switching.
- Exact 1100×720 viewport: Inspector remains readable and Graph uses intentional horizontal scrolling.
- Run identity resolves the bundled `seven-stage-product-delivery@0.2.0` from the immutable local package store and verifies its SHA-256 before rendering.
- Definition topology is projected from declared edges (including `when` and bounded loop policy), rather than inferred from node order.

## Screenshot index

1. `01–05`: onboarding stages.
2. `06`: Review Attention baseline.
3. `07–08`: Definition view and Deploy MCP grants.
4. `09–10`: natural-language customization and permission review.
5. `11–13`: Help and theme variants.
6. `14–15`: readable Activity Inspector and Node Focus.
7. `16–18`: feedback loop, Deploy, final completion.
8. `19–20`: Graph and Definition at exact 1100×720.
9. `21–24`: Deliverable empty state, More menu, Attention filter and Doctor.
10. `25`: data-driven Graph regression at 1100×720.
11. `26–27`: real edge ledger and a Deploy node with exact Skill/MCP/tool/capability grants.

## Remaining evidence gate

Before calling the native interaction review complete, recapture the same matrix from `/Applications/Agent Pipeline.app`, including titlebar drag, minimize/fullscreen/restore, close/reopen with runner persistence, Dock icon, and exact 1100×720 native window. The current macOS capture-channel failure prevents that evidence from being produced automatically in this run.
