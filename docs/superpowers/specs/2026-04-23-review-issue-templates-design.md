# Review Issue Templates — Design Spec

**Date:** 2026-04-23
**Status:** Approved (brainstorming session)
**Scope:** Create GitHub issue-form templates for the upcoming second codebase review (clean architecture, best practice, docs).

---

## Context

YoYoPod just completed a "COMPLETE ARCH REWORK" (commit `8f624ed`, PR #317). A prior review on 2026-04-18 produced 12 architecture findings (`#224`–`#235`), all of which were closed as `obsolete` on 2026-04-23 because every code path they cited was renamed or deleted by the rework. Key failure modes of the prior review template:

1. **No commit pin** — findings didn't record the reviewed snapshot, so after a large refactor there was no way to retrace what the reviewer actually saw.
2. **SLOC-driven findings** — several issues led with line-count thresholds the user doesn't care about.
3. **Prescriptive "Proposed fix"** — e.g. "split `loop.py` → `loop_cadence.py + loop_dispatch.py`". When the file was restructured entirely, the prescription became meaningless even where the underlying concern might still apply.
4. **Ad-hoc body format** — no enforced schema; fields drifted between findings.

The second review needs templates that are (a) machine-fillable by a reviewing agent, (b) resistant to code reorganization, (c) narrow enough that each finding has one clear lens.

---

## Goals

- Standardize the shape of review findings so they remain triage-able after future refactors.
- Cover the three stated review concerns (clean architecture, best practice, docs) with enough granularity that each finding lands in a purpose-built form.
- Keep the template count low — four, not seven — so signal isn't diluted across too many buckets.
- Make commit-SHA pinning a required field so findings are frozen against a reviewed snapshot.
- Replace prescriptive "Proposed fix" framing with observational "Suggested direction" framing.

## Non-goals

- Human-filed bug reports / feature requests (left to blank issues; a separate bug template could be added later but is out of scope here).
- Automating review execution. The templates only define how findings land — the reviewing agent is a separate concern.
- Tracking SLOC or any line-count-based heuristic.

---

## Design

### File layout

```
.github/ISSUE_TEMPLATE/
  review-architecture.yml
  review-code-quality.yml
  review-testing.yml
  review-docs.yml
  config.yml            # leaves blank-issue filing enabled
```

### Common fields (present in all four templates)

Ordered as they appear in the form:

| Order | Field | Control | Required | Purpose |
|-------|-------|---------|----------|---------|
| 1 | Finding ID | `input` | no | e.g. `A01`, `CQ03`. Cross-refs a review summary issue. |
| 2 | Source commit SHA | `input` | **yes** | Pins the finding to the reviewed snapshot. Survives later refactors. |
| 3 | Severity | `dropdown` | yes | `critical` / `high` / `medium` / `low`. |
| 4 | Effort | `dropdown` | yes | `small` / `medium` / `large`. |
| 5 | Files affected | `textarea` | yes | Code paths, one per line. Line numbers optional — paths are primary. |
| 6 | Finding | `textarea` | yes | What's wrong, observed at the pinned SHA. |
| 7 | Impact | `textarea` | yes | What breaks / what rule or doc it violates. |
| 8 | Suggested direction | `textarea` | no | Observational, not prescriptive. Principles over file names. |
| 9 | References | `textarea` | no | `rules/*.md`, `docs/*.md`, external links. |

### Per-template specialization

#### `review-architecture.yml`
- **Auto-labels:** `architecture`, `review`
- **Title prefix:** `[Arch] `
- **Extra fields:**
  - `Concern type` (dropdown, required): `boundary` / `coupling` / `state-ownership` / `threading` / `layering` / `dependency-direction`
  - `Invariant violated` (input, optional): free text pointing at a rule or doc invariant

#### `review-code-quality.yml`
- **Auto-labels:** `code-quality`, `review`
- **Title prefix:** `[CQ] `
- **Extra fields:**
  - `Concern type` (dropdown, required): `error-handling` / `logging` / `dead-code` / `naming` / `idioms` / `perf-hotpath` / `security`
- **Note:** perf and security ride on this template via the dropdown until finding volume justifies their own templates.

#### `review-testing.yml`
- **Auto-labels:** `testing`, `review`
- **Title prefix:** `[Test] `
- **Extra fields:**
  - `Gap type` (dropdown, required): `missing-coverage` / `brittle-test` / `mock-vs-reality` / `CI-gate-gap` / `integration-gap`
  - `Test gap or failing assertion` (textarea, required): the concrete test shape or assertion that's missing/broken

#### `review-docs.yml`
- **Auto-labels:** `docs`, `review`
- **Title prefix:** `[Docs] `
- **Extra fields:**
  - `Doc problem` (dropdown, required): `stale` / `missing` / `inaccurate` / `contradictory`
  - `Doc location` (input, required): path to the affected doc (or "N/A — doc missing")
  - `What it should say` (textarea, optional): the corrected or intended content

### `config.yml`

```yaml
blank_issues_enabled: true
```

Leaves blank issues on — review forms are additive, not exclusive. A future bug-report template can slot in without blocking ad-hoc human reports.

---

## Labels to create in the repo

New labels required (the `obsolete` label is already present from the prior cleanup):

| Label | Color (suggested) | Purpose |
|---|---|---|
| `code-quality` | `#fbca04` | Applied by `review-code-quality.yml`. |
| `testing` | `#0e8a16` | Applied by `review-testing.yml`. |
| `docs` | `#0075ca` | Applied by `review-docs.yml`. Create new rather than reusing existing `documentation` so the `review-*` label family is consistently named. |
| `review` | `#5319e7` | Stable parent label applied by all four templates. |
| `review:2026-04-23` | `#5319e7` | Per-review-round child label, created fresh each review. |

Existing usable labels (no change): `architecture`, `severity:{critical,high,medium,low}`, `effort:{small,medium,large}`, `perf`, `tech-debt`, `pi-zero`, `obsolete`.

Note: YAML issue forms can only apply **static** labels via the `labels:` frontmatter field. Severity and Effort dropdowns are form-body fields, not labels — the reviewing agent applies matching `severity:*` / `effort:*` labels via `gh issue edit --add-label` after creation (or the web submitter adds them manually).

---

## Obsolescence-prevention contract

Baked into every template:

1. **Commit SHA required.** The pinned snapshot is the authoritative reference — not the current HEAD, not the PR at triage time.
2. **No SLOC fields.** Arch cleanliness is the concern, not line counts. (Confirmed with user during brainstorming.)
3. **"Suggested direction" not "Proposed fix".** Prior review prescribed exact file splits that rotted when the codebase reorganized. Observational framing survives reorganization even when the specific mechanism no longer applies.
4. **Paths are primary, line numbers are optional.** Paths are stabler than line numbers across refactors.
5. **Finding + Impact separated.** The prior template collapsed "current behavior" and "why it violates X" into a single prose paragraph. Splitting them makes each finding easier to skim and reason about during triage.

---

## Reviewer workflow

The reviewing agent is expected to:

1. Run a structured audit against a pinned commit (typically `main` HEAD at review start).
2. For each finding, generate a title, pick the right template, fill the form fields (body).
3. Submit via `gh issue create --body <rendered-body> --label "<static-labels-from-template>" --label "severity:<x>" --label "effort:<y>" --label "review:YYYY-MM-DD"`.
4. Optionally create a tracking issue that cross-references all findings by Finding ID.

Because `gh issue create` doesn't enforce YAML form structure from the CLI, the template file is a **schema contract** the agent reads to know which fields to render. The form's enforcement kicks in only for web submissions — which is acceptable given this is review-only.

---

## Open questions

None at the time of design approval. The user confirmed Option 2 (four templates) and YAML form format.

---

## Next steps

1. Transition to `writing-plans` skill to produce an implementation plan covering:
   - Four YAML form files with exact field schemas.
   - `config.yml`.
   - Label creation via `gh label create` (three or four new labels).
   - A brief README or entry in the repo root explaining the review workflow.
   - Verification: a dry-run of the forms by opening each in GitHub's web UI.
