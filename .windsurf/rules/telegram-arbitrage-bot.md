---
trigger: model_decision
---

## 1. Foundational Tech: Clarity and Consistency
   - **Rule 1.1: Stick to the Stack.** Our core stack is TypeScript, Cloudflare Workers, `ccxt` for exchange interactions, `itty-router` for routing, `grammy` for Telegram notifications, and Vitest for testing. Adhere to this stack unless a well-researched, compelling reason justifies a deviation. This aligns with picking a mainstream tech stack for clarity and broad support.
   - **Rule 1.2: Configuration as Code.** All project configurations, including Cloudflare Worker settings (`wrangler.toml`), build scripts (`package.json`), and linting/formatting rules, shall be version-controlled.

## 2. Planning & Execution: Precision and Incrementality
   - **Rule 2.1: Simple PRD (Product Requirement Document) First.** Before writing code for any new feature or significant change, create a concise PRD. This document must:
      - Clearly define what you want to achieve.
      - Break the work into small, clear, testable steps.
   - **Rule 2.2: One Step at a Time.** Whether instructing an AI or coding manually, implement and test one small, well-defined step at a time. Avoid "doing everything at once."

## 3. Testing: The Cornerstone of Quality
   - **Rule 3.1: Test-Driven Development (TDD) is Mandatory.**
      - Always write a failing test *before* writing implementation code for new functionality or bug fixes.
      - Red -> Green -> Refactor is our TDD cycle.
   - **Rule 3.2: Vitest is Our Sole Testing Framework.** All automated tests (unit, integration, etc.) must be written using Vitest. Leverage its features like mocking, spies, and worker environment testing effectively.
   - **Rule 3.3: >95% Test Coverage is the Goal.** Strive for and maintain test coverage above 95%. Regularly run `npm run test:coverage` and address gaps. Coverage is a means to an end: ensuring robust, well-tested code.
   - **Rule 3.4: Comprehensive Tests.** Tests must cover:
      - Happy paths.
      - Edge cases.
      - Error handling and failure scenarios.
      - Interactions with external services (e.g., exchanges via `ccxt`) should be mocked appropriately for unit/integration tests.

## 4. Version Control: Track Progress, Enable Reversion
   - **Rule 4.1: Git Discipline is Key.**
      - Commit frequently with clear, descriptive messages adhering to conventional commit formats.
        - **Examples of good commit messages:**
          - `feat: add Binance Futures integration for position tracking`
          - `fix: correct USDT balance calculation on Kraken`
          - `docs: update README with setup instructions for Telegram bot`
          - `style: apply Biome formatter to all .ts files`
          - `refactor: simplify order book parsing logic in exchangeService`
          - `test: add unit tests for funding rate APR calculation`
          - `chore: upgrade vitest to v1.5.0`
      - Use feature branches for all new development and bug fixes (e.g., `feature/add-new-exchange`, `fix/incorrect-fee-calculation`).
      - Regularly push branches to the remote repository.
      - Merge branches via Pull Requests (even if working solo, for process consistency).

## 5. AI Collaboration: Smart and Effective
   - **Rule 5.1: Provide Clear Context and Working Samples.** When using AI for assistance (e.g., Cursor, GitHub Copilot):
      - Don't assume AI knows about third-party libraries or specific APIs from documentation alone.
      - If modifying existing code or adding related functionality, provide the AI with relevant, *working* code samples from the project.
   - **Rule 5.2: Break the "Stuck Loop" with Fresh Context.** If AI assistance leads to a frustrating "copy error -> paste to chat -> fix -> new error -> repeat" cycle:
      - **Stop.**
      - Open a fresh chat/prompt with the AI.
      - Clearly state:
         - What's broken or not working.
         - What you expected to happen.
         - What you've already tried (briefly).
         - Include essential logs, error messages, and even screenshots if they help clarify.
      - A clean context and clear input are vital for effective AI collaboration. The longer the chat history, often the "dumber" the AI gets.

## 6. Code Quality and Maintenance
   - **Rule 6.1: Lint and Format Consistently.** Adhere to the project's linting (`biome lint ./src`) and formatting (`biome --write '**/*.{ts,js,json,md}'`) rules. Integrate these into your pre-commit hooks if possible.
   - **Rule 6.2: Write Readable and Maintainable Code.** Focus on clarity. Refactor code when TDD cycles allow, to improve structure and readability without changing behavior (tests must always pass after refactoring).
   - **Rule 6.3: Document Wisely.** Document complex logic, architectural decisions, and any non-obvious parts of the code.

## 7. Dependency Management
   - **Rule 7.1: Keep Dependencies Updated.** Regularly review and update dependencies using `npm update` or specific version bumps. Test thoroughly after updates.
   - **Rule 7.2: Scrutinize New Dependencies.** Before adding a new dependency, evaluate its necessity, maintenance status, and potential alternatives.

## 8. CI/CD Integration: Automate for Quality and Speed
   - **Rule 8.1: Automate Testing.**
      - All pushes to feature branches and pull requests to the main branch must trigger the full test suite (`npm test` or `yarn test`).
      - Ensure that tests run in an environment that closely mimics production (e.g., using appropriate mocks for Cloudflare services if not testing directly against them).
      - Test failures must block merges to the main branch.
   - **Rule 8.2: Automate Linting and Formatting Checks.**
      - All pushes and pull requests should trigger linting (`npm run lint` or `yarn run lint`) and formatting checks (e.g., `biome check .` or a dry-run formatting command).
      - Linting errors or formatting inconsistencies should ideally block merges or at least be clearly flagged in PRs.
   - **Rule 8.3: Build Verification.**
      - The CI pipeline should attempt to build the project (`npm run build` or `yarn run build`) to catch any build-time errors early.
   - **Rule 8.4: (Future) Automated Deployment.**
      - Once stable, configure CI/CD to automatically deploy successful builds of the main branch to a staging or production environment using Wrangler (`npm run deploy` or `yarn run deploy`).
      - Implement appropriate safeguards and manual approval steps for production deployments if necessary.
   - **Rule 8.5: Secure Secrets Management.**
      - **Storage:** Never hardcode API keys, tokens, or other sensitive environment secrets directly in your codebase or commit them to version control.
         - **Recommended Tools:** Utilize secure secret management solutions.
            - **GitHub Secrets:** For storing secrets used in GitHub Actions workflows. These are encrypted and can be exposed to workflows as environment variables.
            - **HashiCorp Vault:** A powerful, centralized secrets management tool suitable for more complex or self-hosted environments.
            - **Cloudflare Worker Secrets:** Use Wrangler to upload secrets that will be available to your deployed Worker functions.
      - **Rotation:** Implement a policy for regular rotation of API keys and other sensitive credentials. The frequency should depend on the sensitivity of the secret and the risk tolerance.
      - **Access Control:** Limit access to secrets on a need-to-know basis. Configure your CI/CD pipeline and secret stores to grant only the necessary permissions to specific workflows or deployment stages.
      - **Audit Trails:** Ensure your secret management solution provides audit trails to track access and modifications to secrets.
      - **Local Development:** For local development, use `.env` files (which should be in `.gitignore`) to store secrets. Provide a `.env.example` file in your repository with placeholder values to guide developers.

This is a living document. Revisit and refine these rules as the project evolves.
