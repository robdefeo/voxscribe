
## Principles

Design, engineering, leadership, and communication principles for different professional contexts.

---

### Design & Product Management

- **Principle of Least Astonishment (POLA)**: A system component should behave as users expect, not surprising them. Design interfaces and workflows to match mental models and established conventions. Use status indicators, clear feedback, visible affordances.
- **Visibility**: System state should be immediately observable. Users shouldn't guess what's happening. Provide status indicators, clear feedback, and visible affordances.
- **Progressive Disclosure**: Reveal complexity only when needed. Show basics by default, hide advanced options until relevant. Scaffold learning progressively.
- **Consistency**: Apply the same rules and patterns across similar situations. Use uniform naming, replicate patterns across interfaces, document deviations explicitly.

### Engineering & Development

- **Keep It Simple, Stupid (KISS)**: Simpler solutions are preferable to complex ones. Resist over-engineering. Choose straightforward approaches. Minimize moving parts. Document why complexity was necessary when it is.
- **Don't Repeat Yourself (DRY)**: Eliminate redundancy; maintain a single source of truth. Duplication creates divergence. Extract shared logic, use libraries, refactor repeated patterns.
- **Separation of Concerns**: Each module should have a single, well-defined responsibility. Single-responsibility modules are predictable. Design clear interfaces, isolate concerns, test independently.
- **Convention over Configuration**: Provide sensible defaults and standard patterns. Define framework defaults, use standard naming, minimize configuration surfaces.

### Leadership & Management

- **Transparency**: Keep decision-making processes and reasoning visible. Teams are less surprised when they understand the 'why.' Share rationale, document trade-offs, explain constraints.
- **Explicit Constraints**: Clearly communicate boundaries and scope upfront. Define scope, communicate resource limits, state decision boundaries, document non-negotiables.

---

### Writing & Communication

- **Simple is Better Than Complex**: Clear, accessible expression outweighs sophisticated or dense writing. Use short sentences and common words. Remove jargon unless necessary. Value clarity over cleverness. Make complexity visible rather than hidden.
- **Information Scent**: Links, headings, and titles should accurately signal what's inside. Readers shouldn't be surprised by content. Use descriptive links, specific headings, preview scope, fulfill promises.
- **Structural Parallelism**: Parallel sentence structure creates pattern recognition. Parallel structures set expectations that are then met. Use consistent lists, matching syntax, parallel emphasis.

---

## Build Commands

```bash
just build              # build all crates
just test               # cargo test + bun test
just lint               # cargo fmt --check + cargo clippy -- -D warnings + biome check
just fmt                # cargo fmt + biome format --write
just coverage-check     # grcov coverage check
```

## Code Style

- Group by responsibility, not type
- Avoid large modules, split early

### Rust

- Architect to prefer immutability: avoid `&mut self` and `let mut` where a value can be constructed or transformed instead; use interior mutability (`AtomicT`, `Mutex`) only when the trait or concurrency boundary requires it.
- No `unwrap()` in library code — use `?` and `anyhow`
- Tests in `tests/` (integration) or inline `#[cfg(test)]` (unit)
- Prefer `tests/` for integration tests; add `benches/` or `examples/` only when they add value.
- `lib.rs` defines public API and high-level module structure, not implementation details.


## Commit and PR Rules

- Commit message format and PR title must follow `.config/commitlint.config.mjs`
