# Engineering Instructions

This is production software for a real motorcycle repair workshop.

## Non-negotiable development process

Use TDD for business behavior:

1. Write a test describing the required behavior.
2. Confirm the test fails for the expected reason.
3. Implement the smallest correct change.
4. Confirm the test passes.
5. Refactor while preserving behavior.
6. Run the relevant broader test suite.

Do not write implementation first and manufacture tests afterward unless explicitly dealing with infrastructure that cannot reasonably be test-first.

Tests must represent real application behavior.

Do not:

* add meaningless tests for coverage numbers;
* excessively mock internal implementation;
* test private implementation details;
* change tests simply to make incorrect implementation pass;
* silently weaken assertions;
* silently change established business rules.

Use clear Arrange / Act / Assert structure.

## Architecture

Keep these concerns separated:

* presentation
* application/use cases
* domain
* infrastructure/persistence

Domain code must remain independent of React, Tauri and SQLite wherever reasonably possible.

UI components must not directly contain persistence logic.

SQLite access belongs behind repositories/infrastructure adapters.

Use transactions whenever one business operation changes multiple related pieces of persisted state.

## Data integrity

Workshop records and inventory history are business data.

Never perform destructive migrations without an explicit migration strategy.

Inventory changes must be represented by auditable stock movements rather than unexplained quantity mutations.

Tests involving persistence should use isolated temporary databases.

## Scope

Implement one requested slice at a time.

Do not opportunistically add unrelated features.

Before declaring work complete:

* tests pass;
* type checking passes;
* lint passes;
* production build passes;
* relevant integration tests pass;
* no known regression has been introduced.

Prefer simple code and explicit domain concepts over speculative abstractions.
