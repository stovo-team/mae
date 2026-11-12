# Repository Instructions

## OKF Repository Protocol

Before planning or editing, run `okf-agent index .` when the index is absent, then retrieve repository context. After source changes, run `okf-agent update . --changed-files <changed paths>`. Before completion, run `okf-agent validate .`. Preserve curated content outside OKF-generated blocks.

<!-- okf:workflow:start -->
## OKF Workflow Routing

Read `.okf/workflows/active.md` when present. Classify the task and assess risk, then load only the selected workflow profile and assigned role instructions. Use independent GTDD only when separate role contexts and controlled handoffs are available; otherwise report a single-agent adversarial review.
<!-- okf:workflow:end -->
