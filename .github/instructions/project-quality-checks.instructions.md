---
description: "Use when changing Vue frontend files, Tauri Rust files, or project tooling in Camera Controls. Covers linting, formatting, Rust checks, and updating README developer workflow notes when tooling changes."
name: "Camera Controls Quality Checks"
applyTo: ["src/**", "src-tauri/**", "package.json", "README.md", "eslint.config.js", ".prettierrc.json", ".prettierignore", "vite.config.js"]
---
# Camera Controls Quality Checks

- Run the narrowest available validation after edits instead of relying on inspection alone.
- For Vue and Vite changes, use `npm run lint` to catch real errors and warnings before finishing.
- When Prettier is configured, proactively format touched files or the smallest relevant scope needed to keep edits consistent with the project style.
- For Tauri or Rust changes, use `npm run check:rust` to run `cargo fmt --check` and `cargo clippy` with warnings treated as failures.
- When adding or changing developer tooling, scripts, or workflow expectations, update `README.md` so the project instructions stay accurate.
- Fix errors by default in the touched area. If warnings are pre-existing or unrelated, note them clearly instead of widening the task unless asked.