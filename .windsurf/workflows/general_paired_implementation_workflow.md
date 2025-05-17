---
description: TDD-Implementation
---



## 1. General Paired Implementation Workflow

> **Goal:** Ship high-quality, test-backed changes in small increments by pairing source ↔ test (or doc) files.

### Principles

* **Focused Scope:** Tackle one logical change at a time, working on a primary file and its counterpart (test, doc, type, etc.).
* **Red-Green-Refactor:** Prefer TDD—write a failing spec first.
* **Immutable CI Pipeline:** Every push triggers tests, lint, formatting, type-checks, and coverage.

---

### 1.1. Pre-Work

1. **Task Intake**

   * Link to ticket/PRD and relevant spec in `/Docs`.
   * Confirm acceptance criteria, mocks, edge cases.
2. **Branch Naming**

   ```
   feature/<short-desc>  
   fix/<short-desc>  
   chore/update-<pkg>  
   ```
3. **Commit Guidelines**

   * Use [Conventional Commits](https://www.conventionalcommits.org/).
   * One logical change per commit.

---

### 1.2. Setup Boilerplate (if needed)

* Create skeleton in both files (`X.ts` + `X.test.ts` or `X.spec.ts`).
* Import relevant modules, helpers, fixtures, and testing utilities.

---

### 1.3. Iterative TDD Cycle

Repeat until the sub-task is complete:

|  Step | Who | Action                                                                        |
| :---: | :-: | :---------------------------------------------------------------------------- |
| **A** | Dev | RED: Add a single failing test in `*.test.ts` (Vitest) covering one behavior. |
| **B** | Dev | GREEN: Implement minimal code in `service.ts` to pass that test.              |
| **C** | Dev | REFRACTOR: Tidy up code and tests; ensure readability.                        |
| **D** | Dev | VERIFY:                                                                       |

* `pnpm run test` (all tests pass)
* `pnpm run lint` & `pnpm run format` (no errors)
* Local manual sanity check  |

---

### 1.4. Quality Gates

* **Coverage:** New or changed code must have at least 90% test coverage. Overall project coverage should not decrease. (`pnpm run test:coverage`).
* **Pre-commit Hooks:**

  * `husky`: block bad commits (lint, tests).
  * `commitlint`: enforce commit style.
* **CI Checks:** Green build + coverage badge.

---

### 1.5. Merge & Next Steps

1. **PR Template:** Fill description, attach screen recordings or logs.
2. **Peer Review:** Address comments → ✅ All checks pass.
3. **Squash & Merge:** Preserve clean history.
4. **Post-Merge:**

   * Delete branch.
   * Document in changelog if needed.
   * Identify next feature/bug pair.

---

## 2. Feature-Driven Paired Workflow (TDD Emphasis)

> **Enhancements for new functionality.**

1. **Define Requirements**

   * Small, atomic user stories/behaviors.
   * Write mini-PRDs: input, output, side-effects, error cases.

2. **Branch & Draft PR**

   ```bash
   git fetch && git checkout main && git pull
   git checkout -b feature/<short-name>
   ```

   * Draft PR early with “WIP” tag to gather feedback.

3. **TDD Loop (per behavior)**

   * RED: `it('…', () => expect(…).to…);`
   * GREEN: Minimal implementation.
   * REFACTOR: Extract, rename, optimize.
   * Commit each cycle with clear message.

4. **End-to-End Validation**

   * Add integration/UI tests if applicable.
   * Manual exploratory testing.

5. **Final Checks**

   * Coverage, lint, format.
   * Remove `WIP` tag, update description, request review.

---

## 3. Bug-Fix Paired Workflow (TDD Emphasis)

> **Quick reproduction → fail test → fix → verify.**

1. **Reproduce & Document**

   * Step-by-step reproduction in ticket.
   * Attach logs/screenshots.

2. **Branch**

   ```bash
   git checkout main && git pull
   git checkout -b fix/<short-bug-desc>
   ```

3. **TDD Fix Cycle**

   * RED: Write a test case that exposes the bug.
   * GREEN: Correct code path.
   * VERIFY: All tests pass, including regression.

4. **QA & Merge**

   * Ensure no new warnings, coverage holds.
   * Follow PR template → review → merge.

---

## 4. AI-Assisted Coding Workflow

> **Guided prompts + critical validation.**

1. **Task Prep**

   * Isolate smallest, testable chunk.
   * Collect relevant code context.

2. **Prompt Sequence**

   * **Test Generation:** “Write a Vitest test for …”
   * **Implementation:** “Implement minimal code to satisfy above.”
   * **Edge Cases:** “Add tests for … , refactor code accordingly.”

3. **Review & Validate**

   * Code walkthrough + run tests locally.
   * Spot-check performance/security implications.

4. **Iteration**

   * If AI output is off-mark, refine prompt or switch context.

5. **Finalize**

   * Integrate code, run full suite, commit, and push.

---

## 5. Dependency Update Workflow

> **Safe, traceable upgrades.**

1. **Audit**

   ```bash
   pnpm outdated
   ```

2. **Branch**

   ```bash
   git checkout -b chore/update-<pkg-name>
   ```

3. **Upgrade**

   ```bash
   pnpm add <pkg>@<version>
   ```

   * Review CHANGELOG/SECURITY notices.

4. **Test & Validate**

   * Full test suite + coverage.
   * Manual smoke tests for critical flows.

5. **Commit & Merge**

   * Bump `package.json`/`lockfile`.
   * PR → review → merge → monitor.

---

## 6. Tooling & Environment

* **Node.js & pnpm**
* **Scripts**

  * `pnpm run dev` (local)
  * `pnpm run build` (CI)
  * `pnpm run deploy` (CF Workers via Wrangler)
* **Testing**

  * `pnpm run test` / `pnpm run test:watch` / `pnpm run test:coverage`
* **Lint & Format**

  * `pnpm run lint` (Biome)
  * `pnpm run format` (Biome)
* **CI/CD**

  * GitHub Actions → run lint, tests, coverage, and deploy on merge

---
