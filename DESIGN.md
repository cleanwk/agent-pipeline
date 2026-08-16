---
name: Agent Pipeline
description: A system drafting table for observable, recoverable, auditable Agent work.
colors:
  canvas: "#f3f0e7"
  canvas-strong: "#ece8dd"
  paper: "#f7f4ec"
  paper-raised: "#fbf8f0"
  ink: "#20211f"
  muted: "#696a65"
  faint: "#a6a399"
  rule: "rgba(50, 50, 45, 0.18)"
  rule-strong: "rgba(45, 45, 40, 0.34)"
  wire: "#236a9b"
  attention: "#d53a2c"
  attention-wash: "#f6dcd4"
  success: "#2b9351"
  night-canvas: "#151b1d"
  night-paper: "#20292c"
  night-ink: "#eef1eb"
  night-wire: "#60a9da"
  night-attention: "#ff6a55"
  warm-canvas: "#eee2c5"
  warm-paper: "#f8edd3"
  warm-ink: "#282117"
typography:
  display:
    fontFamily: "-apple-system, BlinkMacSystemFont, SF Pro Text, PingFang SC, Helvetica Neue, sans-serif"
    fontSize: "30px"
    fontWeight: 650
    lineHeight: 1.15
    letterSpacing: "-0.04em"
  headline:
    fontFamily: "-apple-system, BlinkMacSystemFont, SF Pro Text, PingFang SC, Helvetica Neue, sans-serif"
    fontSize: "24px"
    fontWeight: 650
    lineHeight: 1.2
    letterSpacing: "-0.035em"
  title:
    fontFamily: "-apple-system, BlinkMacSystemFont, SF Pro Text, PingFang SC, Helvetica Neue, sans-serif"
    fontSize: "16px"
    fontWeight: 650
    lineHeight: 1.25
    letterSpacing: "-0.025em"
  body:
    fontFamily: "-apple-system, BlinkMacSystemFont, SF Pro Text, PingFang SC, Helvetica Neue, sans-serif"
    fontSize: "12px"
    fontWeight: 400
    lineHeight: 1.6
  label:
    fontFamily: "-apple-system, BlinkMacSystemFont, SF Pro Text, PingFang SC, Helvetica Neue, sans-serif"
    fontSize: "11px"
    fontWeight: 500
    lineHeight: 1.5
  mono:
    fontFamily: "SFMono-Regular, Consolas, monospace"
    fontSize: "11px"
    fontWeight: 400
    lineHeight: 1.65
rounded:
  status: "4px"
  control: "5px"
  compact: "7px"
  node: "8px"
  menu: "9px"
  panel: "10px"
  circle: "999px"
spacing:
  hairline: "1px"
  xs: "5px"
  sm: "7px"
  md: "9px"
  lg: "14px"
  xl: "16px"
  section: "28px"
  grid: "28px"
components:
  button-primary:
    backgroundColor: "{colors.wire}"
    textColor: "#ffffff"
    typography: "{typography.label}"
    rounded: "{rounded.control}"
    padding: "0 13px"
    height: "34px"
  button-secondary:
    backgroundColor: "transparent"
    textColor: "{colors.ink}"
    typography: "{typography.label}"
    rounded: "{rounded.control}"
    padding: "0 13px"
    height: "34px"
  graph-node:
    backgroundColor: "{colors.paper}"
    textColor: "{colors.ink}"
    rounded: "{rounded.node}"
    width: "140px"
---

# Design System: Agent Pipeline

## Overview

**Creative North Star: "The System Drafting Table"**

Agent Pipeline makes long-running Agent work feel like a durable technical drawing: warm drafting surfaces, graphite partitions, blueprint-blue relationships, and one vermilion intervention signal. The interface is an operating instrument, not a collection of metrics. Users first see what needs them, follow the Graph to the relevant Node, then inspect evidence and act without losing history.

The product is not a generic dashboard, a low-code builder, or a Dify-style drag-and-drop canvas. The model performs configuration labor from natural language; the UI explains, navigates, reviews, and recovers versioned facts. Graph, Attention, Activity, Attempt, Artifact, and Handoff must remain visibly distinct concepts.

**Key Characteristics:**

- Dense but quiet macOS desktop tooling; warm technical paper rather than SaaS chrome.
- Fine rules, restrained radii, compact controls, tabular metadata, and sparse elevation.
- Attention first, Graph dominant, Inspector fixed; Node Focus is the deliberate deep-reading mode.
- Immutable attempts and revisions stay visible. New facts append; old facts fade into history but never disappear.

## Colors

Color is semantic before it is atmospheric. Components consume role tokens (`canvas`, `paper`, `ink`, `rule`, `wire`, `attention`, `success`), never theme-specific literals.

| Role | Meaning |
| --- | --- |
| Canvas / paper | Drafting field, partition, normal surface, raised surface—in that depth order. |
| Ink / muted / faint | Primary content, secondary context, and de-emphasized history or unavailable state. |
| Rule / rule-strong | Structural separation and interactive enclosure; use lines before shadows. |
| Wire | Graph causality, selection, running state, focus, and primary action. |
| Attention / wash | Human intervention, rejection, failure, or blocked continuation. It is not a brand accent. |
| Success | Completed, ready, validated, or locally healthy. |

### Theme contract

- **System:** follow macOS. Light uses the canonical tokens in frontmatter; dark uses the full Night Ops semantic mapping.
- **Draft Light:** cooler, cleaner paper (`canvas #f4f2eb`, `paper #f9f7f1`) while retaining the canonical ink, wire, attention, and success meanings.
- **Night Ops:** charcoal canvas/paper with pale ink, brighter blueprint wire, coral attention, and mint success. Increase shadow opacity, not border weight.
- **Warm Paper:** ochre canvas/paper and brown-black ink; keep blueprint wire, vermilion attention, and green success unchanged so operational meaning survives the atmosphere.

**The Semantic Swap Rule.** A Theme Pack may remap semantic tokens only. It may not change hierarchy, layout, component structure, status meaning, or inject package-specific UI.

**The Vermilion Scarcity Rule.** In the Graph, the active Review Gate and its blocked/feedback path are the sole vermilion focal point. Do not use attention color for neutral emphasis or decoration.

Status must never rely on color alone: pair color with an icon, label, stroke style, or both.

## Typography

Use the native macOS sans stack for the application and SF Mono-compatible faces for logs, diffs, package paths, IDs, and protocol facts. The character is precise and compact, with large type reserved for onboarding and top-level authoring/deliverable headings.

- **Display:** onboarding milestones only; tight and confident (`30px`, `-0.04em`).
- **Headline:** top-level non-Graph views (`24px`, `-0.035em`).
- **Title:** Run and Node names (`16–18px`, tight tracking).
- **Body:** explanatory copy (`12–13px`, `1.6–1.75` line height).
- **Label:** operational UI, metadata, tabs, statuses, and table copy (`11–13px`). Operational text never drops below `11px`.
- **Mono:** diffs, logs, file manifests, paths, IDs, timestamps, and protocol output (`11px`, tabular numerals where applicable).

**The Labels Carry Facts Rule.** Avoid marketing-like display hierarchy inside the run workspace. Let alignment, rules, state marks, and concise labels establish hierarchy.

## Layout

The Run first viewport is invariant: a `42px` native titlebar, a `72px` command bar, then three full-height columns—Attention (`246px`), Graph (`minmax(600px, 1fr)`), and fixed Inspector (`300px`). The center Graph owns the visual field; side rails support it and scroll independently. The Graph uses a `28px` drafting grid and a stable stage, so causality does not jump as events arrive.

Node Focus is a state change, not a modal: hide Attention and Graph, then center the Inspector as a paper sheet up to `920px` wide with `28px` vertical breathing room. Preserve its tabs, facts, and actions so focus feels like zooming into the same Node.

Deliverables use a restrained two-column index + revision list. Authoring uses intent + proposal preview. Neither should devolve into cards: rows, manifests, permissions, validation output, provenance, and revisions are the content.

At widths up to `1220px`, compress Attention/Inspector to `210px/280px`, collapse command labels to icons, and truncate the Run brief; retain the three-column causal model. This is a macOS desktop product: do not invent a phone dashboard. If a narrower adaptive surface is introduced, make Attention and Inspector explicit switchable modes and keep Graph geometry horizontally scrollable rather than stacking unrelated panels.

Use a compact `5–16px` control rhythm, `28px` section/grid rhythm, and `30–52px` padding only for focused onboarding or authoring canvases. Every scroll region needs `min-width/min-height: 0`; application chrome itself does not scroll.

## Elevation & Depth

Depth comes from tonal layers and one-pixel rules. Default rails, nodes, tables, and inspection blocks are flat. Shadows are reserved for floating menus, generated proposal sheets, selected Graph nodes, and the centered Node Focus sheet. A normal node gets only a faint ambient shadow; waiting and historical nodes get none.

Theme menus use the raised-surface shadow (`0 12px 32px rgba(57,51,39,.12)` light; `0 16px 36px rgba(0,0,0,.34)` night). Primary actions may carry a small wire-colored glow; selection rings are state feedback, not general elevation.

**The Lines Before Lift Rule.** Add a rule, tone change, or spacing break before adding a shadow. Persistent dashboard-card shadows flatten the drafting-table hierarchy.

Motion is functional and brief: theme color/background transitions use `180ms ease-out`; running marks spin at `1.4s linear`; live work pulses at `1.8s ease-out`. No entrance choreography, graph-node drifting, or decorative parallax. Respect `prefers-reduced-motion` by reducing animation and transition duration to effectively zero.

## Shapes

The system is rectilinear with gently eased corners. Use `4–5px` for statuses, fields, buttons, and compact chips; `7–8px` for icons and Graph nodes; `9–10px` only for menus and large proposal sheets. Circles are reserved for status dots, checks, and step indices. Avoid pill-heavy styling.

Borders are normally `1px`; an active attention node may strengthen to `1.5px`. Waiting nodes and historical attempts use dashed borders. Feedback and attention edges are dashed; historical edges are thinner, lower-contrast, and partially transparent. Do not round or decorate wires.

## Components

- **Titlebar and command bar:** native drag surface above a paper command strip. Keep Runner health visible, theme selection compact, and icon buttons labeled. Active view uses a subtle ink wash, not a saturated tab.
- **Attention Rail:** fixed intervention queue with icon, title, evidence summary, and time. Selected rows gain an attention wash plus a `2px` leading rule. The empty state says execution can continue; it is not a celebration card.
- **Run Graph:** the primary navigation and explanation surface. Nodes are fixed-position technical plates with index, status mark, title, state, and time. Edges express causality; activity expands beneath the owning Node without changing the Graph's meaning.
- **Graph state grammar:** completed = green filled check; running = blue spinner and blue border; waiting = muted dashed enclosure; attention/failed = vermilion alert or failure mark; selected = semantic blue ring unless attention; historical Attempt = faint dashed plate in a dedicated lower band. A feedback edge is stronger than an ordinary edge. Never erase or overwrite a prior Attempt.
- **Node Inspector:** fixed right rail with Overview, Activity, Artifacts, and Logs. It owns evidence, runtime facts, diff review, recovery/advance actions, and the Node Focus affordance. Logs remain an explicit raw-session escape hatch, not the default story.
- **StatusMark:** one reusable icon grammar across Graph, Attention, timelines, Doctor, and artifacts. Always expose an accessible status name.
- **Buttons and fields:** primary is blueprint blue with white text; secondary is transparent with a strong rule; text action is muted. All controls use visible hover and `2px` focus-visible outlines with `2px` offset. Disabled async actions keep their label and show waiting through copy plus reduced opacity.
- **Deliverables:** formal Artifact revisions only—title, revision, summary, media/size/time, producer Node, and Attempt. Group by delivery slot and keep export secondary. Logs, temporary files, and full transcripts do not masquerade as deliverables.
- **Authoring:** natural-language intent on the left; inspectable Package Proposal on the right. Show the proposed mini-Graph, bounded loops, files, permissions, validation, source, and install state. The result is a real text package, never private canvas state.
- **Onboarding and Doctor:** a five-step, light drafting-table sequence: local boundary, host check, Agent discovery, Workspace, ready. Maintain the persistent step index and local-data promise. Doctor re-enters the Agent discovery/check flow and reports installation, version, authentication, and protocol capability separately. Checks are read-only, make no model request, and state external Runtime/network boundaries plainly.

## Do's and Don'ts

### Do

- **Do** lead every Run view with intervention, causality, evidence, and a clear next action.
- **Do** preserve immutable Attempt, Artifact revision, event, and provenance facts in both layout and copy.
- **Do** keep Review rejection visually connected to the new Implement Attempt through a bounded feedback edge.
- **Do** use semantic tokens and verify every new component in System light/dark, Draft Light, Night Ops, and Warm Paper.
- **Do** preserve keyboard focus, meaningful labels, non-color state cues, `11px` minimum operational text, and reduced-motion behavior.

### Don't

- **Don't** build a generic dashboard, KPI-card grid, command center of interchangeable widgets, or gratuitous glassmorphism.
- **Don't** add a drag-and-drop workflow canvas, node handles, freeform placement, or visual configuration forms. Author through natural language and review the generated protocol.
- **Don't** use Graph as decoration or make Activity/log streams more prominent than Node state, Attention, Artifact, and recovery boundaries.
- **Don't** recolor individual packages or statuses with arbitrary accents; extend the semantic Theme Pack contract.
- **Don't** hide prior attempts, collapse formal deliverables into file clutter, or overwrite history to make the current state look simpler.
