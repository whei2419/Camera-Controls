---
description: "Use when creating or refactoring Vue frontend files or Tauri Rust files in Camera Controls. Covers splitting large files, extracting focused helpers, and organizing code for readability and maintainability."
name: "Readability And File Organization"
applyTo: ["src/**", "src-tauri/**"]
---
# Readability And File Organization

- Prefer small, focused files over large multi-purpose files.
- When a file starts handling multiple concerns, split by responsibility instead of adding more branches to the same module.
- Treat large files, long components, and modules with unrelated sections as split candidates before adding new behavior.
- Split when a single file is mixing responsibilities such as UI rendering, persistence, network calls, parsing, or orchestration logic.
- Keep UI rendering, state management, side effects, and transport or API logic separated when practical.
- For Vue components, extract repeated logic or long non-visual behavior into smaller components, helper modules, or composables when that improves readability.
- For Rust code, prefer smaller domain-oriented modules such as camera, commands, parsing, configuration, or transport instead of concentrating unrelated behavior in one file.
- Extract helper functions, structs, or modules when a Rust file mixes transport, state, parsing, and command handling in one place.
- Name files and symbols by responsibility, not by vague words like `utils`, `helpers`, or `misc`, unless the module is genuinely narrow and cohesive.
- Prefer colocating related files in a clear folder rather than creating deep folder trees with only one file per level.
- Avoid splitting code so aggressively that the flow becomes hard to trace. Favor one clear responsibility per file, not maximum fragmentation.
- If a refactor changes where behavior lives, keep imports, naming, and nearby documentation aligned with the new structure.