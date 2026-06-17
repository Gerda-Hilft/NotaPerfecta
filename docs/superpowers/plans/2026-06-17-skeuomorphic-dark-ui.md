# Skeuomorphic Dark-Mode UI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reskin NotaPerfecta from a flat Tokyo Night aesthetic to a skeuomorphic dark-mode UI with physical materials (brushed aluminum, aged leather, cast resin, typewriter paper).

**Architecture:** Pure CSS + SVG reskin. Rewrite `tokens.css` (palette, fonts, radii, shadows) and `App.css` (all component styles). Replace `Background.tsx` orbs with brushed aluminum + screw SVGs. Bundle three font families locally. No changes to React component logic, hooks, types, or Tauri backend.

**Tech Stack:** CSS custom properties, CSS pseudo-elements, inline SVG (in Background.tsx), `@font-face` declarations, Vite asset pipeline.

## Global Constraints

- Dark mode only. No `prefers-color-scheme` media query. No light mode fallback.
- No pure black (`#000`). No neon glows. No glassmorphism blur.
- Every shadow warm-tinted (`rgba(20,14,10,...)`) — never cold grey.
- Radii 6-12px (machined edges, not pill shapes). Exception: `--radius-full` stays `9999px` for badge pills.
- All fonts bundled locally in `src/assets/fonts/` via `@font-face`. No Google Fonts CDN imports.
- Upper-left lighting direction for all specular highlights and shadows.

---

### Task 1: Bundle Fonts Locally & Rewrite tokens.css

**Files:**
- Create: `src/assets/fonts/PlayfairDisplay-Regular.woff2`
- Create: `src/assets/fonts/PlayfairDisplay-Bold.woff2`
- Create: `src/assets/fonts/PlayfairDisplay-Italic.woff2`
- Create: `src/assets/fonts/AtkinsonHyperlegible-Regular.woff2`
- Create: `src/assets/fonts/AtkinsonHyperlegible-Bold.woff2`
- Create: `src/assets/fonts/AtkinsonHyperlegible-Italic.woff2`
- Create: `src/assets/fonts/AtkinsonHyperlegible-BoldItalic.woff2`
- Create: `src/assets/fonts/SpecialElite-Regular.woff2`
- Create: `src/assets/fonts/CourierPrime-Regular.woff2`
- Create: `src/assets/fonts/CourierPrime-Bold.woff2`
- Create: `src/styles/fonts.css`
- Modify: `src/styles/tokens.css` (full rewrite)

**Interfaces:**
- Consumes: nothing (foundational task)
- Produces: all CSS custom properties used by every other task. `--color-*`, `--font-*`, `--radius-*`, `--shadow-*`, `--btn-3d-*`, `--space-*`, `--transition-*`, `--text-*`, `--font-*` weight tokens.

- [ ] **Step 1: Download font files**

Download woff2 files from Google Fonts. Use the google-webfonts-helper API or direct download from fonts.google.com. Place each file in `src/assets/fonts/`.

```bash
mkdir -p src/assets/fonts
cd src/assets/fonts

# Playfair Display (Regular 400, Bold 700, Italic 400i)
curl -L -o PlayfairDisplay-Regular.woff2 "https://fonts.gstatic.com/s/playfairdisplay/v37/nuFvD-vYSZviVYUb_rj3ij__anPXJzDwcbmjWBN2PKdFvXDXbtM.woff2"
curl -L -o PlayfairDisplay-Bold.woff2 "https://fonts.gstatic.com/s/playfairdisplay/v37/nuFvD-vYSZviVYUb_rj3ij__anPXJzDwcbmjWBN2PKeiu3DXbtM.woff2"
curl -L -o PlayfairDisplay-Italic.woff2 "https://fonts.gstatic.com/s/playfairdisplay/v37/nuFRD-vYSZviVYUb_rj3ij__anPXDTnCjmHKM4nYO7KN_qiTbtbK-F2rA0s.woff2"

# Atkinson Hyperlegible (Regular, Bold, Italic, BoldItalic)
curl -L -o AtkinsonHyperlegible-Regular.woff2 "https://fonts.gstatic.com/s/atkinsonhyperlegible/v11/9Bt23C1KxNDXMspQ1lPyU89-1h6ONRlW45GE5AI5gFCifQ.woff2"
curl -L -o AtkinsonHyperlegible-Bold.woff2 "https://fonts.gstatic.com/s/atkinsonhyperlegible/v11/9Bt73C1KxNDXMspQ1lPyU89-1h6ONRlW45G055ItWQGCbg.woff2"
curl -L -o AtkinsonHyperlegible-Italic.woff2 "https://fonts.gstatic.com/s/atkinsonhyperlegible/v11/9Bt43C1KxNDXMspQ1lPyU89-1h6ONRlW45GE349wTCWYQ0.woff2"
curl -L -o AtkinsonHyperlegible-BoldItalic.woff2 "https://fonts.gstatic.com/s/atkinsonhyperlegible/v11/9Bt93C1KxNDXMspQ1lPyU89-1h6ONRlW45G038cIw3SCbg.woff2"

# Special Elite
curl -L -o SpecialElite-Regular.woff2 "https://fonts.gstatic.com/s/specialelite/v18/XLYgIZbkc4JPUL5CVArUVL0ntnAOSA.woff2"

# Courier Prime (Regular, Bold)
curl -L -o CourierPrime-Regular.woff2 "https://fonts.gstatic.com/s/courierprime/v9/u-450q2lgwslOqpF_6gQ8kELWwZjyFE.woff2"
curl -L -o CourierPrime-Bold.woff2 "https://fonts.gstatic.com/s/courierprime/v9/u-4k0q2lgwslOqpF_6gQ8kELY7pMf-fVqvHo.woff2"
```

Verify all files downloaded and are non-empty:
```bash
ls -la src/assets/fonts/*.woff2
```

- [ ] **Step 2: Create src/styles/fonts.css with @font-face declarations**

```css
/* src/styles/fonts.css */

/* Playfair Display — display/headings */
@font-face {
  font-family: 'Playfair Display';
  font-style: normal;
  font-weight: 400;
  font-display: swap;
  src: url('../assets/fonts/PlayfairDisplay-Regular.woff2') format('woff2');
}
@font-face {
  font-family: 'Playfair Display';
  font-style: normal;
  font-weight: 700;
  font-display: swap;
  src: url('../assets/fonts/PlayfairDisplay-Bold.woff2') format('woff2');
}
@font-face {
  font-family: 'Playfair Display';
  font-style: italic;
  font-weight: 400;
  font-display: swap;
  src: url('../assets/fonts/PlayfairDisplay-Italic.woff2') format('woff2');
}

/* Atkinson Hyperlegible — body/labels */
@font-face {
  font-family: 'Atkinson Hyperlegible';
  font-style: normal;
  font-weight: 400;
  font-display: swap;
  src: url('../assets/fonts/AtkinsonHyperlegible-Regular.woff2') format('woff2');
}
@font-face {
  font-family: 'Atkinson Hyperlegible';
  font-style: normal;
  font-weight: 700;
  font-display: swap;
  src: url('../assets/fonts/AtkinsonHyperlegible-Bold.woff2') format('woff2');
}
@font-face {
  font-family: 'Atkinson Hyperlegible';
  font-style: italic;
  font-weight: 400;
  font-display: swap;
  src: url('../assets/fonts/AtkinsonHyperlegible-Italic.woff2') format('woff2');
}
@font-face {
  font-family: 'Atkinson Hyperlegible';
  font-style: italic;
  font-weight: 700;
  font-display: swap;
  src: url('../assets/fonts/AtkinsonHyperlegible-BoldItalic.woff2') format('woff2');
}

/* Special Elite — typewriter mono for text pane */
@font-face {
  font-family: 'Special Elite';
  font-style: normal;
  font-weight: 400;
  font-display: swap;
  src: url('../assets/fonts/SpecialElite-Regular.woff2') format('woff2');
}

/* Courier Prime — mono fallback */
@font-face {
  font-family: 'Courier Prime';
  font-style: normal;
  font-weight: 400;
  font-display: swap;
  src: url('../assets/fonts/CourierPrime-Regular.woff2') format('woff2');
}
@font-face {
  font-family: 'Courier Prime';
  font-style: normal;
  font-weight: 700;
  font-display: swap;
  src: url('../assets/fonts/CourierPrime-Bold.woff2') format('woff2');
}
```

- [ ] **Step 3: Rewrite src/styles/tokens.css**

Replace the entire file. Remove the Google Fonts `@import`, remove the `prefers-color-scheme: dark` media query, remove `@keyframes` (those move to App.css).

```css
/* src/styles/tokens.css — Skeuomorphic dark-mode design tokens */
@import "./fonts.css";

:root {
  color-scheme: dark;

  /* -- Fonts ------------------------------------------------- */
  --font-sans: 'Atkinson Hyperlegible', ui-sans-serif, system-ui, sans-serif;
  --font-display: 'Playfair Display', ui-serif, Georgia, 'Times New Roman', serif;
  --font-mono: 'Special Elite', 'Courier Prime', ui-monospace, Menlo, monospace;

  /* -- Palette ----------------------------------------------- */
  --color-bg:              #1A1612;
  --color-surface:         #2A2420;
  --color-surface-raised:  #342E29;
  --color-surface-inset:   #1E1A15;
  --color-border:          #4A4035;
  --color-border-subtle:   #3A3228;

  --color-text:            #EDE8DF;
  --color-text-secondary:  #B8B0A4;
  --color-text-muted:      #8C8278;

  --color-primary:         #C9A84C;
  --color-primary-hover:   #D4B85C;
  --color-primary-light:   rgba(201, 168, 76, 0.12);
  --color-primary-border:  #8A7430;
  --color-accent:          #C9A84C;
  --color-accent-soft:     rgba(201, 168, 76, 0.08);

  --color-danger:          #C0392B;
  --color-danger-bg:       rgba(192, 57, 43, 0.12);
  --color-danger-border:   rgba(192, 57, 43, 0.25);
  --color-warning:         #D4A84C;
  --color-warning-bg:      rgba(212, 168, 76, 0.12);
  --color-warning-border:  rgba(212, 168, 76, 0.25);
  --color-success:         #27AE60;
  --color-success-bg:      rgba(39, 174, 96, 0.10);

  /* -- Typography scale -------------------------------------- */
  --text-xs:    0.75rem;
  --text-sm:    0.875rem;
  --text-base:  1rem;
  --text-lg:    1.125rem;
  --text-xl:    1.25rem;
  --text-2xl:   1.5rem;
  --text-3xl:   1.875rem;

  --leading-tight:    1.25;
  --leading-normal:   1.5;
  --leading-relaxed:  1.7;

  --font-normal:    400;
  --font-medium:    500;
  --font-semibold:  600;
  --font-bold:      700;

  --tracking-tight:   -0.01em;
  --tracking-wide:     0.05em;
  --tracking-widest:   0.16em;

  --body-size:         17px;
  --body-line-height:  1.7;

  /* -- Radii (machined edges, 6-12px) ----------------------- */
  --radius-sm:    6px;
  --radius-md:    8px;
  --radius-lg:    10px;
  --radius-xl:    12px;
  --radius-2xl:   12px;
  --radius-full:  9999px;

  /* -- Shadows (warm-tinted, never cold grey) ---------------- */
  --shadow-sm:  0 2px 4px rgba(20, 14, 10, 0.5);
  --shadow-md:  0 6px 16px rgba(20, 14, 10, 0.6);
  --shadow-lg:  0 12px 32px rgba(20, 14, 10, 0.7);

  /* -- 3D button system (dark resin) ------------------------- */
  --btn-3d-top:     #3E3630;
  --btn-3d-bottom:  #2A2420;
  --btn-3d-border:  #4A4035;
  --btn-3d-shadow:  #1A1612;

  /* -- Spacing ----------------------------------------------- */
  --space-1: 0.25rem;
  --space-2: 0.5rem;
  --space-3: 0.75rem;
  --space-4: 1rem;
  --space-5: 1.25rem;
  --space-6: 1.5rem;
  --space-8: 2rem;

  /* -- Motion ------------------------------------------------ */
  --transition-fast: 160ms ease;
  --transition-base: 220ms ease;
  --pointer-x: 0;
  --pointer-y: 0;
}

@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
```

- [ ] **Step 4: Verify fonts load**

Run: `cd "/home/me/Projects/Multi lang/NotaPerfecta" && pnpm dev`

Open browser dev tools, check Network tab for `.woff2` requests. Verify no 404s. Check Elements > Computed tab for body `font-family` resolves to Atkinson Hyperlegible, h1 resolves to Playfair Display.

- [ ] **Step 5: Commit**

```bash
git add src/assets/fonts/ src/styles/fonts.css src/styles/tokens.css
git commit -m "feat(ui): bundle fonts locally, rewrite design tokens for skeuomorphic dark mode"
```

---

### Task 2: Background Component — Brushed Aluminum & Screw Heads

**Files:**
- Modify: `src/components/Background.tsx` (full rewrite)
- Modify: `src/App.css` — `.bg-field`, `.bg-*` classes (replace orb styles)

**Interfaces:**
- Consumes: CSS tokens from Task 1 (`--color-bg`, `--color-border`, `--color-surface-inset`)
- Produces: visual background layer. No API — purely visual, `aria-hidden="true"`.

- [ ] **Step 1: Rewrite Background.tsx**

Replace the orb-based background with brushed aluminum texture + four corner screw SVGs. The pointer-tracking effect stays but drives the noise opacity subtly instead of orb positions.

```tsx
// src/components/Background.tsx
import { useEffect } from "react";

function ScrewHead() {
  return (
    <svg width="16" height="16" viewBox="0 0 16 16" aria-hidden="true">
      <defs>
        <radialGradient id="screwDome" cx="40%" cy="35%" r="50%">
          <stop offset="0%" stopColor="#6B5E50" />
          <stop offset="60%" stopColor="#4A4035" />
          <stop offset="100%" stopColor="#2A2420" />
        </radialGradient>
      </defs>
      <circle cx="8" cy="8" r="7" fill="url(#screwDome)" stroke="#1A1612" strokeWidth="0.5" />
      <line x1="4" y1="8" x2="12" y2="8" stroke="#1A1612" strokeWidth="1" strokeLinecap="round" opacity="0.6" />
      <line x1="8" y1="4" x2="8" y2="12" stroke="#1A1612" strokeWidth="1" strokeLinecap="round" opacity="0.6" />
      <circle cx="6" cy="6" r="2" fill="rgba(237,232,223,0.06)" />
    </svg>
  );
}

export function Background() {
  useEffect(() => {
    function onMove(e: PointerEvent) {
      const x = (e.clientX / window.innerWidth - 0.5) * 40;
      const y = (e.clientY / window.innerHeight - 0.5) * 40;
      document.documentElement.style.setProperty("--pointer-x", x.toFixed(2));
      document.documentElement.style.setProperty("--pointer-y", y.toFixed(2));
    }
    window.addEventListener("pointermove", onMove, { passive: true });
    return () => window.removeEventListener("pointermove", onMove);
  }, []);

  return (
    <div className="bg-field" aria-hidden="true">
      <div className="bg-streaks" />
      <div className="bg-noise" />
      <div className="bg-screw bg-screw-tl"><ScrewHead /></div>
      <div className="bg-screw bg-screw-tr"><ScrewHead /></div>
      <div className="bg-screw bg-screw-bl"><ScrewHead /></div>
      <div className="bg-screw bg-screw-br"><ScrewHead /></div>
    </div>
  );
}
```

- [ ] **Step 2: Replace background CSS in App.css**

Remove all `.bg-orb`, `.orb-1/2/3`, `.bg-grid` rules and the `@keyframes drift`. Replace with:

```css
/* ── Background: brushed aluminum ───────────────── */
.bg-field {
  position: fixed;
  inset: 0;
  z-index: 0;
  pointer-events: none;
  overflow: hidden;
  background: var(--color-bg);
}

.bg-streaks {
  position: absolute;
  inset: 0;
  background-image: repeating-linear-gradient(
    0deg,
    transparent,
    transparent 1px,
    rgba(74, 64, 53, 0.03) 1px,
    rgba(74, 64, 53, 0.03) 2px
  );
  background-size: 100% 3px;
  opacity: 0.7;
}

.bg-noise {
  position: absolute;
  inset: 0;
  background-image: url("data:image/svg+xml,%3Csvg width='200' height='200' viewBox='0 0 200 200' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='200' height='200' filter='url(%23n)' opacity='0.03'/%3E%3C/svg%3E");
  opacity: 0.8;
  mix-blend-mode: soft-light;
}

.bg-screw {
  position: fixed;
  z-index: 2;
}
.bg-screw-tl { top: 10px; left: 10px; }
.bg-screw-tr { top: 10px; right: 10px; }
.bg-screw-bl { bottom: 10px; left: 10px; }
.bg-screw-br { bottom: 10px; right: 10px; }
```

- [ ] **Step 3: Update app-shell and app frame CSS**

Replace the current `.app-shell` and `.app` rules:

```css
.app-shell {
  position: relative;
  min-height: 100vh;
  border: 1px solid var(--color-border-subtle);
}

.app {
  position: relative;
  z-index: 1;
  max-width: 1200px;
  margin: 0 auto;
  padding: var(--space-6) var(--space-4) var(--space-8);
  box-shadow: inset 0 1px 3px rgba(20, 14, 10, 0.3);
}
```

- [ ] **Step 4: Verify visually**

Run: `pnpm dev`

Check: background is dark warm tone with subtle horizontal streaks, four screw heads visible in corners, noise grain barely perceptible, no colored orbs.

- [ ] **Step 5: Commit**

```bash
git add src/components/Background.tsx src/App.css
git commit -m "feat(ui): brushed aluminum background with screw-head details"
```

---

### Task 3: App Header — Engraved Title Plate & Pipeline Toggle Housing

**Files:**
- Modify: `src/App.css` — `.app-header`, `.pipeline-toggle`, `.chip` rules

**Interfaces:**
- Consumes: tokens from Task 1, background from Task 2
- Produces: styled header toolbar, title plate, toggle housing. Used by `App.tsx` and `PipelineToggle.tsx` (no component changes needed).

- [ ] **Step 1: Restyle the app header and title plate**

Replace the `.app-header` and `h1` rules in App.css:

```css
/* ── App header & title plate ──────────────────── */
.app-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-4);
  margin-bottom: var(--space-5);
  padding-bottom: var(--space-4);
  border-bottom: 1px solid var(--color-border-subtle);
}
.app-header > h1 {
  font-family: var(--font-display);
  font-size: var(--text-2xl);
  font-weight: var(--font-bold);
  margin: 0;
  white-space: nowrap;
  color: var(--color-text);
  background: var(--color-surface-inset);
  padding: var(--space-2) var(--space-4);
  border-radius: var(--radius-md);
  box-shadow:
    inset 1px 1px 3px rgba(0, 0, 0, 0.4),
    inset -1px -1px 1px rgba(237, 232, 223, 0.03);
  text-shadow:
    -1px -1px 1px rgba(0, 0, 0, 0.5),
    1px 1px 1px rgba(237, 232, 223, 0.08);
  letter-spacing: var(--tracking-tight);
}
```

- [ ] **Step 2: Restyle the pipeline toggle as tactile radio switches**

Replace `.pipeline-toggle` and `.chip` rules:

```css
/* ── Pipeline toggle: tactile radio switches ────── */
.pipeline-toggle {
  display: inline-flex;
  gap: 2px;
  align-items: center;
  background: var(--color-surface-inset);
  border-radius: var(--radius-md);
  padding: 3px;
  box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.4);
  margin-bottom: 0;
}

.chip {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  border-radius: var(--radius-sm);
  padding: 0.4rem 0.85rem;
  font-family: var(--font-sans);
  font-size: var(--text-sm);
  font-weight: var(--font-semibold);
  border: 1px solid transparent;
  background: transparent;
  color: var(--color-text-muted);
  cursor: pointer;
  transition:
    background   var(--transition-fast),
    color        var(--transition-fast),
    border-color var(--transition-fast),
    box-shadow   var(--transition-fast);
}
.chip:hover {
  color: var(--color-text-secondary);
  background: rgba(74, 64, 53, 0.2);
}
.chip-active {
  background: var(--color-primary-light);
  color: var(--color-primary);
  border-color: var(--color-primary-border);
  box-shadow: 0 0 8px rgba(201, 168, 76, 0.15);
}
.chip:active {
  transform: translateY(1px);
}
```

- [ ] **Step 3: Restyle the settings button**

The "Einstellungen" button uses `.btn.btn-outline.btn-sm` — it gets restyled as part of the button system. Make sure `.app-header-actions` still lays out correctly:

```css
.app-header-actions {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}
.app-header-actions .pipeline-toggle {
  margin-bottom: 0;
}
```

- [ ] **Step 4: Verify visually**

Run dev server. Check: title "NotaPerfecta" appears in Playfair Display, recessed metal plate look. Pipeline toggle has dark housing with the active chip backlit amber. The three chips sit inside a recessed channel.

- [ ] **Step 5: Commit**

```bash
git add src/App.css
git commit -m "feat(ui): engraved title plate and tactile pipeline toggle"
```

---

### Task 4: Buttons — Resin, Amber Lacquer & Accept/Reject

**Files:**
- Modify: `src/App.css` — `.btn`, `.btn-primary`, `.btn-outline`, `.btn-ghost`, `.btn-sm` rules
- Modify: `src/components/SuggestionCard.tsx` — add `btn-accept` and `btn-reject` classes to the accept/reject buttons

**Interfaces:**
- Consumes: tokens from Task 1
- Produces: button styling used across all components (DropZone, ExportButton, SuggestionCard, SettingsDialog, FolderSidebar)

- [ ] **Step 1: Rewrite all button styles in App.css**

```css
/* ── Buttons: embossed resin ───────────────────── */
.btn {
  --btn-depth: 4px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  height: 48px;
  padding: 0 1.25rem;
  font-family: var(--font-sans);
  font-size: var(--text-base);
  font-weight: var(--font-semibold);
  border-radius: var(--radius-md);
  border: 1px solid var(--btn-3d-border);
  cursor: pointer;
  background: linear-gradient(180deg, var(--btn-3d-top), var(--btn-3d-bottom));
  color: var(--color-text);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.06),
    0 var(--btn-depth) 0 var(--btn-3d-shadow),
    0 8px 16px -8px rgba(20, 14, 10, 0.4);
  transition:
    transform    100ms ease,
    box-shadow   100ms ease,
    background   var(--transition-fast),
    color        var(--transition-fast),
    border-color var(--transition-fast);
}
.btn:active {
  transform: translateY(3px);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.04),
    0 1px 0 var(--btn-3d-shadow);
}
.btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
  transform: none;
}

.btn-sm {
  --btn-depth: 3px;
  height: 36px;
  padding: 0 0.75rem;
  font-size: var(--text-sm);
}

/* Primary CTA: amber lacquered */
.btn-primary {
  background: linear-gradient(180deg, #D4B85C, #B8952E);
  color: var(--color-bg);
  border-color: #8A7430;
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.15),
    0 var(--btn-depth) 0 #7A6520,
    0 10px 20px -10px rgba(201, 168, 76, 0.3);
  text-shadow:
    0 -1px 0 rgba(0, 0, 0, 0.2),
    0 1px 0 rgba(255, 255, 255, 0.1);
}
.btn-primary:hover:not(:disabled) {
  background: linear-gradient(180deg, #DCC46A, #C9A84C);
}
.btn-primary:active {
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.1),
    0 1px 0 #7A6520;
}

/* Outline/ghost: standard dark resin */
.btn-outline,
.btn-ghost {
  color: var(--color-text-secondary);
}
.btn-outline:hover:not(:disabled),
.btn-ghost:hover:not(:disabled) {
  background: linear-gradient(180deg, #453E37, #2A2420);
  color: var(--color-text);
}

/* Accept button: green-tinted resin */
.btn-accept {
  background: linear-gradient(180deg, #2D4A2D, #1E331E);
  border-color: #3A5A3A;
  color: var(--color-success);
}
.btn-accept:hover:not(:disabled) {
  background: linear-gradient(180deg, #355A35, #254025);
}

/* Reject button: neutral resin (inherits .btn-outline) */
.btn-reject {
  color: var(--color-text-muted);
}
.btn-reject:hover:not(:disabled) {
  color: var(--color-danger);
  border-color: var(--color-danger-border);
}
```

- [ ] **Step 2: Update SuggestionCard.tsx to use btn-accept and btn-reject**

```tsx
// src/components/SuggestionCard.tsx
import type { KorrekturVorschlag } from "../types/corrections";

interface Props {
  v: KorrekturVorschlag;
  onAccept: () => void;
  onReject: () => void;
}

export function SuggestionCard({ v, onAccept, onReject }: Props) {
  return (
    <article className={`karte ${v.status}`}>
      <div className="karte-kopf">
        <span className="badge">{v.type}</span>
        <span className="badge badge-muted">{v.source}</span>
      </div>
      <p>
        <span className="alt">{v.original}</span> → <span className="neu">{v.correction}</span>
      </p>
      <small>{v.explanation}</small>
      <div className="aktionen">
        <button className="btn btn-accept btn-sm" onClick={onAccept}>
          Annehmen
        </button>
        <button className="btn btn-reject btn-sm" onClick={onReject}>
          Ablehnen
        </button>
      </div>
    </article>
  );
}
```

Note: removed the unicode checkmark/cross characters — the button color and label are sufficient. Keeps it cleaner.

- [ ] **Step 3: Verify visually**

Run dev server. Load a PDF and check:
- Resin buttons have visible depth (4px cast shadow)
- Press animation feels snappy (3px down, shadow compresses)
- Accept button has green tint, reject is neutral
- Primary amber CTA ("PDF exportieren") is bright amber with dark text
- Pipeline toggle chips from Task 3 still work correctly

- [ ] **Step 4: Commit**

```bash
git add src/App.css src/components/SuggestionCard.tsx
git commit -m "feat(ui): embossed resin buttons, amber CTA, accept/reject tinting"
```

---

### Task 5: Drop Zone — Leather Inset with Stitched Border

**Files:**
- Modify: `src/App.css` — `.dropzone`, `.dropzone-*`, `@keyframes` rules

**Interfaces:**
- Consumes: tokens from Task 1, button styles from Task 4
- Produces: styled drop zone. Used by `DropZone.tsx` (no component changes needed).

- [ ] **Step 1: Rewrite drop zone CSS**

```css
/* ── Drop zone: leather inset with stitching ───── */
.dropzone {
  position: relative;
  width: 100%;
  min-height: 120px;
  display: flex;
  align-items: center;
  justify-content: center;
  text-align: center;
  font-family: var(--font-display);
  font-size: var(--text-lg);
  font-weight: var(--font-medium);
  color: var(--color-text-muted);
  background: var(--color-surface);
  background-image: url("data:image/svg+xml,%3Csvg width='200' height='200' viewBox='0 0 200 200' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='lg'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.65' numOctaves='3' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='200' height='200' filter='url(%23lg)' opacity='0.04'/%3E%3C/svg%3E");
  border: 2px dashed #6B5E50;
  border-radius: var(--radius-lg);
  margin-bottom: var(--space-4);
  box-shadow:
    inset 0 2px 6px rgba(20, 14, 10, 0.5),
    inset 0 -1px 2px rgba(237, 232, 223, 0.02);
  transition:
    background    var(--transition-fast),
    border-color  var(--transition-fast),
    box-shadow    var(--transition-base),
    color         var(--transition-fast);
}
.dropzone:hover {
  border-color: #8C7A65;
  box-shadow:
    inset 0 3px 8px rgba(20, 14, 10, 0.6),
    inset 0 -1px 2px rgba(237, 232, 223, 0.02);
  background-color: #262018;
}
.dropzone.drag-aktiv {
  border-color: var(--color-primary);
  color: var(--color-primary);
  box-shadow:
    inset 0 2px 6px rgba(20, 14, 10, 0.5),
    0 0 20px rgba(201, 168, 76, 0.15);
  animation: amber-pulse 1.5s ease-in-out infinite;
}

@keyframes amber-pulse {
  0%, 100% { box-shadow: inset 0 2px 6px rgba(20,14,10,0.5), 0 0 15px rgba(201,168,76,0.1); }
  50%      { box-shadow: inset 0 2px 6px rgba(20,14,10,0.5), 0 0 25px rgba(201,168,76,0.25); }
}

.dropzone-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-5);
}
.dropzone-text {
  font-family: var(--font-display);
  font-size: var(--text-base);
  color: var(--color-text-muted);
  text-shadow:
    -1px -1px 1px rgba(0, 0, 0, 0.3),
    1px 1px 1px rgba(237, 232, 223, 0.05);
}
.dropzone-buttons {
  display: flex;
  gap: var(--space-2);
}
```

- [ ] **Step 2: Verify visually**

Run dev server. Check:
- Drop zone has leather grain texture, stitched dashed border
- Hover deepens the deboss
- Drag a file over: amber glow pulses, border turns gold

- [ ] **Step 3: Commit**

```bash
git add src/App.css
git commit -m "feat(ui): leather-inset drop zone with stitched border and amber drag glow"
```

---

### Task 6: Content Panels — Mahogany Frame, Paper Inset & Metal Clipboard

**Files:**
- Modify: `src/App.css` — `.panel`, `.split`, `pre`, `.liste`, `.toolbar`, `.status`, badges, loading, `.meldung`

**Interfaces:**
- Consumes: tokens from Task 1
- Produces: panel and pre-tag styling used by both single-file and folder views in `App.tsx`

- [ ] **Step 1: Rewrite panel and pre styles**

```css
/* ── Panels: mahogany frame ────────────────────── */
.panel {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-xl);
  box-shadow:
    var(--shadow-md),
    inset 0 1px 0 rgba(237, 232, 223, 0.03);
  padding: var(--space-5);
}
.panel > h2 {
  margin: 0 0 var(--space-4);
  font-family: var(--font-display);
  font-size: var(--text-xl);
  color: var(--color-text);
  text-shadow:
    -1px -1px 1px rgba(0, 0, 0, 0.3),
    1px 1px 1px rgba(237, 232, 223, 0.05);
}

.panel-muted {
  background: var(--color-surface-raised);
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--radius-lg);
}

/* Corrections pane clipboard: riveted top edge */
.split > .panel:last-child {
  position: relative;
}
.split > .panel:last-child::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 6px;
  border-radius: var(--radius-xl) var(--radius-xl) 0 0;
  background: linear-gradient(180deg, #4A4035, #342E29);
  background-image:
    radial-gradient(circle at 20%, #6B5E50 3px, transparent 3px),
    radial-gradient(circle at 80%, #6B5E50 3px, transparent 3px);
  background-size: 100% 6px;
  background-repeat: no-repeat;
}

/* ── Original text: aged paper inset ────────────── */
pre {
  white-space: pre-wrap;
  max-height: 55vh;
  overflow: auto;
  margin: 0;
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  line-height: var(--leading-relaxed);
  color: var(--color-text-secondary);
  background: var(--color-surface-inset);
  background-image:
    repeating-linear-gradient(
      180deg,
      transparent,
      transparent 23px,
      rgba(201, 168, 76, 0.04) 23px,
      rgba(201, 168, 76, 0.04) 24px
    );
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--radius-md);
  padding: var(--space-4);
  box-shadow: inset 0 2px 6px rgba(20, 14, 10, 0.4);
  text-shadow: 0 0 1px rgba(0, 0, 0, 0.3);
}

/* ── Split layout ──────────────────────────────── */
.split {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--space-5);
}
@media (max-width: 900px) {
  .split { grid-template-columns: 1fr; }
}
```

- [ ] **Step 2: Restyle badges as embossed metal counters**

```css
/* ── Badges: embossed metal counters ───────────── */
.badge {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.25rem 0.6rem;
  border-radius: var(--radius-full);
  font-size: var(--text-xs);
  font-weight: var(--font-semibold);
  background: var(--color-surface);
  color: var(--color-primary);
  border: 1px solid var(--color-border);
  white-space: nowrap;
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.05),
    0 2px 0 var(--color-bg);
  text-shadow:
    0 -1px 0 rgba(0, 0, 0, 0.3),
    0 1px 0 rgba(237, 232, 223, 0.06);
}
.badge-muted {
  background: var(--color-surface-raised);
  color: var(--color-text-muted);
  border-color: var(--color-border-subtle);
}
.badge-success {
  background: var(--color-success-bg);
  color: var(--color-success);
  border-color: rgba(39, 174, 96, 0.25);
}
.badge-danger {
  background: var(--color-danger-bg);
  color: var(--color-danger);
  border-color: var(--color-danger-border);
}
```

- [ ] **Step 3: Restyle loading spinner as mechanical clock tick**

```css
/* ── Loading: mechanical clock tick ────────────── */
.loading {
  display: flex;
  gap: var(--space-4);
  align-items: center;
  margin-bottom: var(--space-4);
  color: var(--color-text-muted);
  font-size: var(--text-sm);
}
.loading span {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
}
.loading span::before {
  content: '';
  width: 18px;
  height: 18px;
  border-radius: var(--radius-full);
  background: var(--color-surface-inset);
  box-shadow: inset 0 1px 3px rgba(0, 0, 0, 0.5);
  position: relative;
  background-image:
    linear-gradient(var(--color-primary), var(--color-primary));
  background-size: 2px 7px;
  background-repeat: no-repeat;
  background-position: center 2px;
  animation: tick 1.2s steps(12) infinite;
}

@keyframes tick {
  to { transform: rotate(360deg); }
}
```

- [ ] **Step 4: Restyle status row, toolbar, and error messages**

```css
/* ── Status / toolbar rows ─────────────────────── */
.pipeline-toggle,
.status,
.toolbar {
  display: flex;
  gap: var(--space-2);
  flex-wrap: wrap;
  align-items: center;
  margin-bottom: var(--space-4);
}

/* ── Error messages ────────────────────────────── */
.meldung {
  color: var(--color-danger);
  background: var(--color-danger-bg);
  border: 1px solid var(--color-danger-border);
  border-radius: var(--radius-md);
  padding: var(--space-3) var(--space-4);
  margin: 0 0 var(--space-4);
  font-size: var(--text-sm);
  box-shadow: inset 0 1px 3px rgba(192, 57, 43, 0.1);
}
```

- [ ] **Step 5: Verify visually**

Run dev server. Load a PDF. Check:
- Left panel has mahogany-framed look with inset shadow
- Pre/text area shows typewriter font on dark paper with faint ruled lines
- Right panel has a riveted metal strip across the top with two rivet dots
- Badges look embossed (highlight on top, shadow on bottom)
- Loading spinner ticks in 12 steps, not smooth
- Error messages have crimson tint

- [ ] **Step 6: Commit**

```bash
git add src/App.css
git commit -m "feat(ui): mahogany panels, paper inset, metal clipboard, mechanical spinner, embossed badges"
```

---

### Task 7: Suggestion Cards — Index Cards with Stamp & Slash Overlays

**Files:**
- Modify: `src/App.css` — `.karte`, `.karte-kopf`, `.aktionen`, `.alt`, `.neu` rules

**Interfaces:**
- Consumes: tokens from Task 1, `SuggestionCard.tsx` classes (`.karte`, `.angenommen`, `.abgelehnt`)
- Produces: styled suggestion cards with stamp/slash overlays

- [ ] **Step 1: Rewrite suggestion card CSS with stamp and slash overlays**

```css
/* ── Suggestion cards: physical index cards ─────── */
.liste h3 {
  font-family: var(--font-display);
  font-size: var(--text-lg);
  margin: var(--space-4) 0 var(--space-2);
  color: var(--color-text);
  text-shadow:
    -1px -1px 1px rgba(0, 0, 0, 0.3),
    1px 1px 1px rgba(237, 232, 223, 0.05);
}
.liste h3:first-child { margin-top: 0; }

.karte {
  position: relative;
  background: var(--color-surface-raised);
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--radius-lg);
  padding: var(--space-3) var(--space-4);
  margin-bottom: var(--space-3);
  box-shadow:
    2px 3px 0 var(--color-surface),
    4px 6px 0 var(--color-surface-inset),
    0 8px 16px -8px rgba(20, 14, 10, 0.3);
  transition:
    border-color  var(--transition-fast),
    background    var(--transition-fast),
    opacity       var(--transition-fast),
    box-shadow    var(--transition-fast);
  overflow: hidden;
}

/* Paper curl at bottom */
.karte::after {
  content: '';
  position: absolute;
  bottom: 0;
  left: 10%;
  right: 10%;
  height: 4px;
  background: transparent;
  border-radius: 0 0 50% 50%;
  box-shadow: 0 2px 4px rgba(20, 14, 10, 0.3);
  pointer-events: none;
}

/* Accepted: green stamp overlay */
.karte.angenommen {
  border-color: rgba(39, 174, 96, 0.3);
}
.karte.angenommen::before {
  content: 'ANGENOMMEN';
  position: absolute;
  top: 50%;
  right: var(--space-4);
  transform: translateY(-50%) rotate(-12deg);
  font-family: var(--font-display);
  font-size: var(--text-sm);
  font-weight: var(--font-bold);
  color: var(--color-success);
  border: 2px solid var(--color-success);
  border-radius: var(--radius-full);
  padding: 0.15rem 0.6rem;
  opacity: 0.6;
  letter-spacing: var(--tracking-wide);
  text-transform: uppercase;
  pointer-events: none;
  white-space: nowrap;
}

/* Rejected: red diagonal slash */
.karte.abgelehnt {
  opacity: 0.5;
}
.karte.abgelehnt::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(
    to bottom right,
    transparent calc(50% - 1.5px),
    var(--color-danger) calc(50% - 1.5px),
    var(--color-danger) calc(50% + 1.5px),
    transparent calc(50% + 1.5px)
  );
  opacity: 0.4;
  pointer-events: none;
  border-radius: var(--radius-lg);
}

.karte-kopf {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: var(--space-2);
  margin-bottom: var(--space-2);
}
.karte p {
  margin: 0 0 var(--space-1);
  font-size: var(--text-base);
}
.karte small {
  color: var(--color-text-muted);
  font-size: var(--text-sm);
}

.alt {
  color: var(--color-text-muted);
  text-decoration: line-through;
}
.neu {
  color: var(--color-primary);
  font-weight: var(--font-semibold);
}

.aktionen {
  display: flex;
  gap: var(--space-2);
  margin-top: var(--space-3);
}
```

- [ ] **Step 2: Verify visually**

Run dev server. Load a PDF with corrections. Check:
- Cards have stacked shadow (two offset layers beneath)
- Paper curl shadow at bottom edge
- Accept a card: green stamp "ANGENOMMEN" appears rotated, faded ink
- Reject a card: red diagonal slash, card fades to 50% opacity

- [ ] **Step 3: Commit**

```bash
git add src/App.css
git commit -m "feat(ui): index-card suggestion cards with stamp and slash overlays"
```

---

### Task 8: Settings Dialog — Leather Panel with Metal Inputs

**Files:**
- Modify: `src/App.css` — `.dialog-*`, `.settings-*` rules

**Interfaces:**
- Consumes: tokens from Task 1, button styles from Task 4
- Produces: styled dialog used by `SettingsDialog.tsx` (no component changes)

- [ ] **Step 1: Rewrite dialog and settings form styles**

```css
/* ── Settings dialog: leather panel ────────────── */
.dialog-backdrop {
  position: fixed;
  inset: 0;
  z-index: 100;
  background: rgba(26, 22, 18, 0.8);
  display: flex;
  align-items: center;
  justify-content: center;
  animation: fade-in 150ms ease;
}
@keyframes fade-in {
  from { opacity: 0; }
  to   { opacity: 1; }
}

.dialog {
  width: min(480px, 92vw);
  max-height: 85vh;
  overflow-y: auto;
  background: var(--color-surface);
  background-image: url("data:image/svg+xml,%3Csvg width='200' height='200' viewBox='0 0 200 200' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='lg2'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.65' numOctaves='3' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='200' height='200' filter='url(%23lg2)' opacity='0.03'/%3E%3C/svg%3E");
  border: 1px solid var(--color-border);
  border-radius: var(--radius-xl);
  box-shadow: var(--shadow-lg);
  animation: dialog-in 200ms ease;
}
@keyframes dialog-in {
  from { opacity: 0; transform: scale(0.95) translateY(8px); }
  to   { opacity: 1; transform: scale(1) translateY(0); }
}

.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-5) var(--space-5) 0;
}
.dialog-header h2 {
  margin: 0;
  font-family: var(--font-display);
  font-size: var(--text-xl);
  text-shadow:
    -1px -1px 1px rgba(0, 0, 0, 0.3),
    1px 1px 1px rgba(237, 232, 223, 0.05);
}
.dialog-close {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background: linear-gradient(180deg, var(--btn-3d-top), var(--btn-3d-bottom));
  border: 1px solid var(--btn-3d-border);
  border-radius: var(--radius-full);
  font-size: 1.1rem;
  color: var(--color-text-muted);
  cursor: pointer;
  padding: 0;
  line-height: 1;
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.06),
    0 2px 0 var(--btn-3d-shadow);
}
.dialog-close:hover { color: var(--color-text); }
.dialog-close:active {
  transform: translateY(1px);
  box-shadow: 0 1px 0 var(--btn-3d-shadow);
}

.dialog-body {
  padding: var(--space-5);
  display: flex;
  flex-direction: column;
  gap: var(--space-5);
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-2);
  padding: 0 var(--space-5) var(--space-5);
}

/* ── Settings form elements ──────────────────────── */
.settings-group {
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--radius-lg);
  padding: var(--space-4);
  margin: 0;
  box-shadow: inset 0 2px 4px rgba(20, 14, 10, 0.3);
}
.settings-group legend {
  font-size: var(--text-sm);
  font-weight: var(--font-semibold);
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: var(--tracking-wide);
  padding: 0 var(--space-2);
}

.settings-label {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
  font-size: var(--text-sm);
  font-weight: var(--font-medium);
  color: var(--color-text-secondary);
  margin-top: var(--space-3);
}
.settings-label:first-of-type { margin-top: 0; }

.settings-input,
.settings-select {
  width: 100%;
  height: 40px;
  padding: 0 var(--space-3);
  font-family: var(--font-sans);
  font-size: var(--text-sm);
  color: var(--color-text);
  background: var(--color-surface-inset);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  box-shadow: inset 0 1px 3px rgba(20, 14, 10, 0.3);
  transition: border-color var(--transition-fast), box-shadow var(--transition-fast);
}
.settings-input:focus,
.settings-select:focus {
  border-color: var(--color-primary);
  box-shadow:
    inset 0 1px 3px rgba(20, 14, 10, 0.3),
    0 0 0 2px rgba(201, 168, 76, 0.2);
  outline: none;
}

.settings-row {
  display: flex;
  gap: var(--space-2);
  align-items: center;
}
.settings-row .settings-select { flex: 1; }

.settings-hint {
  font-size: var(--text-xs);
  color: var(--color-text-muted);
}
.settings-hint-error {
  color: var(--color-danger);
}
```

- [ ] **Step 2: Verify visually**

Click "Einstellungen" to open the dialog. Check:
- Backdrop is warm dark (no blur)
- Dialog has leather texture, warm shadow
- Fieldsets have recessed inset look
- Inputs are dark metal slots with amber focus ring
- Close button is a small round resin button with depth

- [ ] **Step 3: Commit**

```bash
git add src/App.css
git commit -m "feat(ui): leather settings dialog with metal inputs and resin close button"
```

---

### Task 9: Folder Sidebar — Filing Cabinet with Index-Card Tabs

**Files:**
- Modify: `src/App.css` — `.sidebar`, `.sidebar-*`, `.folder-*` rules

**Interfaces:**
- Consumes: tokens from Task 1
- Produces: styled sidebar used by `FolderSidebar.tsx` (no component changes)

- [ ] **Step 1: Rewrite sidebar and folder layout styles**

```css
/* ── Folder layout & sidebar ───────────────────── */
.folder-layout {
  display: grid;
  grid-template-columns: 280px 1fr;
  gap: var(--space-4);
  align-items: start;
}
@media (max-width: 900px) {
  .folder-layout { grid-template-columns: 1fr; }
}

.folder-main {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}
.folder-panel > pre {
  max-height: 35vh;
}
.folder-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 200px;
  color: var(--color-text-muted);
}

/* ── Sidebar: filing cabinet ───────────────────── */
.sidebar {
  position: sticky;
  top: var(--space-4);
  display: flex;
  flex-direction: column;
  max-height: calc(100vh - 120px);
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-xl);
  box-shadow: var(--shadow-md);
  overflow: hidden;
}

.sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-3) var(--space-4);
  border-bottom: 1px solid var(--color-border-subtle);
  position: relative;
  background: linear-gradient(180deg, #4A4035, #342E29);
}
/* Rivets on sidebar header */
.sidebar-header::after {
  content: '';
  position: absolute;
  top: 50%;
  left: 14px;
  width: 6px;
  height: 6px;
  border-radius: 50%;
  transform: translateY(-50%);
  background: radial-gradient(circle at 35% 35%, #6B5E50, #3A3228);
  box-shadow:
    calc(100% - 28px) 0 0 0 #4A4035,
    0 0 1px rgba(0,0,0,0.5);
}

.sidebar-title {
  font-family: var(--font-display);
  font-size: var(--text-sm);
  font-weight: var(--font-semibold);
  color: var(--color-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  padding-left: var(--space-3);
}

.sidebar-list {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-2);
}
.sidebar-section { margin-bottom: var(--space-2); }
.sidebar-section-title {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  font-size: var(--text-xs);
  font-weight: var(--font-semibold);
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: var(--tracking-wide);
  padding: var(--space-1) var(--space-2);
  margin: 0;
  background: var(--color-surface);
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--radius-sm);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.03),
    0 1px 0 var(--color-bg);
}
.sidebar-count {
  font-size: var(--text-xs);
  background: var(--color-surface-raised);
  border-radius: var(--radius-full);
  padding: 0 0.4rem;
}

.sidebar-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-2);
  width: 100%;
  padding: var(--space-2);
  border: none;
  border-radius: var(--radius-sm);
  border-left: 2px solid transparent;
  background: transparent;
  color: var(--color-text);
  font-family: var(--font-sans);
  font-size: var(--text-sm);
  cursor: pointer;
  text-align: left;
  transition:
    background     var(--transition-fast),
    border-color   var(--transition-fast),
    box-shadow     var(--transition-fast);
}
.sidebar-item:hover {
  background: var(--color-surface-raised);
  box-shadow: 0 2px 4px rgba(20, 14, 10, 0.2);
}
.sidebar-item-active {
  background: var(--color-primary-light);
  color: var(--color-primary);
  font-weight: var(--font-medium);
  border-left-color: var(--color-primary);
}
.sidebar-item-done { opacity: 0.6; }
.sidebar-item-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
  min-width: 0;
}

.sidebar-badge {
  flex-shrink: 0;
  font-size: 0.65rem;
  font-weight: var(--font-semibold);
  padding: 0.1rem 0.4rem;
  border-radius: var(--radius-full);
  white-space: nowrap;
}
.sidebar-badge-muted  { background: var(--color-surface-raised); color: var(--color-text-muted); }
.sidebar-badge-loading { color: var(--color-primary); }
.sidebar-badge-open   { background: var(--color-primary-light); color: var(--color-primary); }
.sidebar-badge-done   { background: var(--color-success-bg); color: var(--color-success); }
.sidebar-badge-clean  { background: var(--color-success-bg); color: var(--color-success); }
.sidebar-badge-error  { background: var(--color-danger-bg); color: var(--color-danger); }

.sidebar-footer {
  padding: var(--space-3);
  border-top: 1px solid var(--color-border-subtle);
}
.sidebar-export-btn { width: 100%; }
.sidebar-export-status {
  font-size: var(--text-xs);
  color: var(--color-success);
  margin: 0 0 var(--space-2);
  text-align: center;
}
```

- [ ] **Step 2: Verify visually**

Run dev server. Open a folder. Check:
- Sidebar has metal header strip with rivets
- Section titles ("Offen"/"Fertig") look like embossed metal labels
- Active file has amber left border
- Hover lifts the item slightly (shadow appears)
- Export button at bottom is amber primary CTA style

- [ ] **Step 3: Commit**

```bash
git add src/App.css
git commit -m "feat(ui): filing-cabinet sidebar with riveted header and index-card tabs"
```

---

### Task 10: Global Reset, Scrollbar, Selection & Focus Styles

**Files:**
- Modify: `src/App.css` — top-of-file reset, scrollbar, selection, focus-visible rules

**Interfaces:**
- Consumes: tokens from Task 1
- Produces: global styles that affect all elements

- [ ] **Step 1: Rewrite global reset and utility styles at top of App.css**

These should be the very first rules in App.css (after the `@import`):

```css
@import "./styles/tokens.css";

/* ── Reset & base ────────────────────────────────── */
*, *::before, *::after { box-sizing: border-box; }

body {
  margin: 0;
  background: var(--color-bg);
  color: var(--color-text);
  font-family: var(--font-sans);
  font-size: var(--body-size);
  line-height: var(--body-line-height);
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

h1, h2, h3, .display {
  font-family: var(--font-display);
  letter-spacing: var(--tracking-tight);
}

::selection {
  background: var(--color-primary);
  color: var(--color-bg);
}

:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
  border-radius: var(--radius-sm);
}

::-webkit-scrollbar       { width: 6px; height: 6px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb {
  background: var(--color-border);
  border-radius: 99px;
}
::-webkit-scrollbar-thumb:hover { background: var(--color-text-muted); }
```

- [ ] **Step 2: Verify the full app end-to-end**

Run dev server. Walk through the complete flow:
1. App loads — brushed aluminum background, screw heads in corners, engraved title plate
2. Drop zone shows — leather pad with stitched border
3. Drag a file over — amber glow pulses
4. Load a PDF — loading spinner ticks mechanically
5. Two-column layout appears — left panel has paper inset with typewriter font, right has clipboard with rivets
6. Suggestion cards are stacked index cards
7. Accept a card — green stamp appears
8. Reject a card — red slash, card fades
9. Open settings — leather dialog, metal inputs
10. Open a folder — filing cabinet sidebar with amber active highlight

Check that no `#000` pure black appears anywhere, no cold grey shadows, no neon glows, no flat fills.

- [ ] **Step 3: Commit**

```bash
git add src/App.css
git commit -m "feat(ui): global dark-mode reset, warm scrollbar, amber selection and focus"
```

---

### Task 11: Final Assembly & Cleanup

**Files:**
- Modify: `src/App.css` — ensure all rules are in correct order, no duplicates
- Modify: `src/App.tsx` — update the h1 text to "NotaPerfecta — Zeugnisprüfung"

**Interfaces:**
- Consumes: all prior tasks
- Produces: complete, clean CSS file with all rules in logical order

- [ ] **Step 1: Update App.tsx title**

Change the h1 text:

```tsx
<h1>NotaPerfecta — Zeugnisprüfung</h1>
```

This replaces the current `<h1>NotaPerfecta</h1>`.

- [ ] **Step 2: Verify App.css rule order**

Ensure App.css follows this section order (top to bottom):
1. `@import "./styles/tokens.css";`
2. Reset & base (body, headings, selection, focus, scrollbar)
3. Background (bg-field, bg-streaks, bg-noise, bg-screw)
4. App shell & layout (app-shell, app, app-header, title plate)
5. Pipeline toggle
6. Buttons (btn, btn-sm, btn-primary, btn-outline, btn-accept, btn-reject)
7. Chips (already folded into pipeline toggle section)
8. Badges
9. Status / toolbar rows
10. Loading indicator
11. Error messages
12. Drop zone
13. Split layout
14. Panels and pre
15. Suggestion cards
16. Folder layout, sidebar
17. Settings dialog and form elements
18. Keyframes (amber-pulse, tick, fade-in, dialog-in)

Remove any duplicate rules or leftover orb/light-mode styles. Remove the `@keyframes drift` and `@keyframes spin` (replaced by `tick`).

- [ ] **Step 3: Full visual smoke test**

Run dev server. Screenshot or walk through every screen state. Verify against the design spec:
- No flat color fills
- No `#000` anywhere
- No cold grey shadows
- All shadows warm-tinted
- Upper-left lighting consistent
- All fonts load (Playfair, Atkinson, Special Elite)
- Reduced-motion still works

- [ ] **Step 4: Commit**

```bash
git add src/App.css src/App.tsx
git commit -m "feat(ui): final assembly — title plate text, rule ordering, cleanup"
```
