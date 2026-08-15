# UI Architecture & Agent Guidelines (`ui/AGENTS.md`)

> **Purpose:** This file defines the frontend stack, design system, build workflow, and rules for AI agents modifying the Warmplane Control Deck web UI.

---

## 🎨 Tech Stack & Architecture

- **Runtime & Bundler:** TypeScript + [Bun](https://bun.sh) (`bun build ./src/main.ts --outfile=./dist/bundle.js --minify`).
- **Styling Paradigm:** **Vanilla CSS & Custom Properties only**.
  - **🚫 NO Tailwind CSS:** Do NOT use Tailwind utility classes (e.g. `p-5`, `bg-gray-900`, `flex`, `space-y-4`). Tailwind is NOT installed.
  - **Single HTML Distribution:** [`ui/build.ts`](file://./ui/build.ts) inlines CSS and JS into a standalone `ui/dist/index.html` bundled into the Rust daemon binary via `include_str!`.

---

## 📐 Design Tokens & CSS Classes (`ui/src/styles/theme.css`)

Always use the established design tokens and CSS utility classes:

### CSS Custom Properties
| Variable | Value / Purpose |
| :--- | :--- |
| `var(--bg-app)` | Application background (`#0c0d10`) |
| `var(--surface)` | Dark surface (`#13151a`) |
| `var(--surface-card)` | Bento card surface (`#181b22`) |
| `var(--surface-hover)` | Hover card background (`#1f232c`) |
| `var(--border)` | Default border (`#262b36`) |
| `var(--border-subtle)` | Subtle border (`#1c202a`) |
| `var(--amber-300)` / `var(--amber-400)` / `var(--amber-500)` | Warmplane amber branding & accents |
| `var(--green-400)` | Success / connected state |
| `var(--red-400)` | Danger / denied / rejected state |
| `var(--cyan-400)` | Info / transport / server accents |
| `var(--text-main)` / `var(--text-muted)` / `var(--text-dim)` | Typography color hierarchy |
| `var(--ff-sans)` / `var(--ff-mono)` | System Sans / JetBrains & Monospace fonts |

### Reusable UI Classes
- `.bento-grid`, `.bento-card`, `.col-3`, `.col-4`, `.col-6`, `.col-12`
- `.btn`, `.btn-primary`, `.btn-ghost`, `.btn-danger`
- `.brand-badge`
- `.form-group`, `.form-label`, `.form-input`, `.form-textarea`
- `.modal-backdrop`, `.modal-box`, `.modal-header`, `.modal-title`

---

## 🛠️ Build & Recompilation Workflow

Whenever changes are made to files in `ui/`:

1. **Rebuild UI Bundle:**
   ```bash
   cd ui && bun run build && bun run ./build.ts
   ```
2. **Recompile Rust Daemon:**
   The Rust binary embeds `ui/dist/index.html` at compile time. Rebuild to test:
   ```bash
   cargo build
   ```
