# NotaPerfecta — Skeuomorphic Dark-Mode UI Design

**Date:** 2026-06-17
**Approach:** CSS reskin + lightweight SVG assets (Approach B)
**Scope:** Full visual overhaul of all existing components. No structural changes to React component logic.

## Aesthetic

Every surface feels like a real physical material: dark brushed aluminum bezels, aged dark leather panels, recessed ink-paper insets, cast-metal buttons with physical depth. Lighting comes from the upper-left. No flat fills — gradients, inner shadows, and specular highlights fake material depth. Textures are subtle (noise, grain, weave), never garish.

## Palette

| Token | Value | Role |
|---|---|---|
| `--color-bg` | `#1A1612` | Near-black warm base |
| `--color-surface` | `#2A2420` | Dark leather panels |
| `--color-surface-raised` | `#342E29` | Raised warm dark panels |
| `--color-border` | `#4A4035` | Warm dark border |
| `--color-border-subtle` | `#3A3228` | Subtle inner borders |
| `--color-text` | `#EDE8DF` | Warm off-white |
| `--color-text-secondary` | `#B8B0A4` | Warm mid-light |
| `--color-text-muted` | `#8C8278` | Warm mid-grey |
| `--color-primary` | `#C9A84C` | Aged amber-gold |
| `--color-primary-hover` | `#D4B85C` | Lighter amber hover |
| `--color-primary-light` | `rgba(201,168,76,0.12)` | Amber tint bg |
| `--color-primary-border` | `#8A7430` | Amber border |
| `--color-accent` | `#C9A84C` | Same amber (unified) |
| `--color-danger` | `#C0392B` | Deep crimson |
| `--color-danger-bg` | `rgba(192,57,43,0.12)` | Dark red tint |
| `--color-danger-border` | `rgba(192,57,43,0.25)` | Crimson border |
| `--color-success` | `#27AE60` | Forest green |
| `--color-success-bg` | `rgba(39,174,96,0.10)` | Dark green tint |
| `--color-warning` | `#D4A84C` | Warm amber warning |
| `--color-warning-bg` | `rgba(212,168,76,0.12)` | Amber warning tint |

## Typography

All fonts bundled locally in `src/assets/fonts/`.

| Role | Font | Token |
|---|---|---|
| Headings / display | Playfair Display | `--font-display` |
| Body / labels | Atkinson Hyperlegible | `--font-sans` |
| Text pane (typewriter) | Special Elite, Courier Prime fallback | `--font-mono` |

## Radii

Tightened to machined edges (8-12px), not pill shapes:

- `--radius-sm: 6px`
- `--radius-md: 8px`
- `--radius-lg: 10px`
- `--radius-xl: 12px`
- `--radius-2xl: 12px`

## Shadows

All warm-tinted (brown/amber bias, never cold grey):

- `--shadow-sm: 0 2px 4px rgba(20,14,10,0.5)`
- `--shadow-md: 0 6px 16px rgba(20,14,10,0.6)`
- `--shadow-lg: 0 12px 32px rgba(20,14,10,0.7)`

## 3D Button System

Dark resin buttons with convex top face:

- `--btn-3d-top: #3E3630` (convex highlight)
- `--btn-3d-bottom: #2A2420` (darker base)
- `--btn-3d-border: #4A4035`
- `--btn-3d-shadow: #1A1612` (cast shadow)

---

## Component Designs

### Background & App Frame

- **Background:** Remove colored orbs. Keep noise texture layer (retune for dark aluminum grain). Replace dot grid with horizontal brushed-aluminum streak pattern (subtle, low-opacity repeating SVG). Four corner screw-head SVGs (~16px, radial gradient dome, crosshead slot, upper-left specular highlight).
- **App shell:** The shell is the brushed aluminum bezel. Thin 1px inset border (warm dark). Main content area sits inside with `inset box-shadow` on all sides for depth.
- **Title plate:** "NotaPerfecta — Zeugnisprüfung" in Playfair Display. Engraved metal plate effect: recessed background `#1E1A15`, inset shadow on top/left edges, text-shadow with 1px light highlight below/right simulating engraved letterforms.

### Drop Zone

- **Material:** Dark leather inset with stitched border.
- Background `#2A2420` with leather grain SVG noise texture (coarser than aluminum).
- Border: `2px dashed #6B5E50` mimicking stitching thread. `box-shadow: inset 0 2px 4px rgba(20,14,10,0.5)` for debossed feel.
- Label "PDF hier ablegen" in Playfair Display with embossed text-shadow (dark upper-left, light lower-right).
- **Hover:** Deeper deboss (increased inset shadow, darker bg).
- **Drag-over:** Amber glow pulse. Border transitions to `#C9A84C`, soft pulsing `box-shadow: 0 0 20px rgba(201,168,76,0.2)` over ~1.5s, label text shifts to amber.
- Buttons ("Einzelnes PDF" / "Ganzer Ordner") styled as small resin buttons.

### Buttons

**Standard (accept/reject) — chunky embossed resin:**
- Convex top: `linear-gradient(180deg, #3E3630, #2A2420)`.
- 1px inner highlight on top: `box-shadow: inset 0 1px 0 rgba(255,255,255,0.06)`.
- Cast shadow: `0 4px 0 #1A1612`.
- Press: shadow to `0 1px 0`, `translateY(3px)`, ~100ms transition.
- Accept: green-tinted gradient `#2D4A2D` to `#1E331E`.
- Reject: neutral dark resin.

**Primary CTA ("PDF exportieren") — amber lacquered:**
- `linear-gradient(180deg, #D4B85C, #B8952E)`.
- Engraved label: dark text `#1A1612`, text-shadow simulating engraved surface.
- Cast shadow: `0 6px 0 #7A6520`. Press compresses 6px to 2px.

**Pipeline toggle chips — tactile radio switches:**
- Group housing: `#1E1A15`, `border-radius: 8px`, `inset box-shadow`.
- Inactive: recessed dark metal, transparent bg, `#8C8278` text.
- Active: backlit amber — `rgba(201,168,76,0.15)` bg, `#C9A84C` text, amber glow, thin amber border.

### Content Panels

**Original text pane (left):**
- Outer frame: `#2A2420` with `inset box-shadow` (mahogany frame).
- Inner paper: `#1E1A15` with faint ruled lines via `repeating-linear-gradient` (horizontal, ~24px spacing, `rgba(201,168,76,0.04)`).
- Text in Special Elite (typewriter mono), `#B8B0A4`, subtle ink-bleed `text-shadow`.

**Corrections pane (right) — dark metal clipboard:**
- Background: `#2A2420`.
- Riveted top edge: `::before` pseudo-element, 6px height, metal gradient, two rivet SVGs at ~20% and ~80% positions.

### Suggestion Cards — physical index cards

- Background: `#342E29`.
- Stacked shadow: `2px 3px 0 #2A2420, 4px 6px 0 #1E1A15`.
- Paper-curl shadow at bottom via `::after`.
- **Accepted:** Circular stamp SVG "ANGENOMMEN" in `#27AE60`, rotated ~-12deg, `opacity: 0.7`, upper-right.
- **Rejected:** Red diagonal slash SVG in `#C0392B`, card at `opacity: 0.5`.

### Loading Spinner

Mechanical clock tick: `animation-timing-function: steps(12)` — 12 discrete ticks per revolution. Thin amber line rotating inside a dark circular housing with inset shadow.

### Status Badges — embossed metal counters

- `background: #2A2420`, `border: 1px solid #4A4035`.
- `box-shadow: inset 0 1px 0 rgba(255,255,255,0.05), 0 2px 0 #1A1612`.
- Letterpress text-shadow on count numbers.

### Settings Dialog

- Backdrop: `rgba(26,22,18,0.8)` — no blur (no glassmorphism).
- Dialog: `#2A2420` leather surface, `border-radius: 12px`, warm shadow.
- Fieldsets: recessed inset with `inset box-shadow`.
- Inputs: dark metal slot `#1E1A15`, amber focus ring.
- Close button: small circular resin button.

### Folder Sidebar — dark metal filing cabinet

- Panel: `#2A2420` leather surface.
- Header: metal strip with rivet treatment (same as clipboard).
- List items: index-card tabs, hover lifts slightly, active gets amber left-border.
- Section titles: small embossed metal labels, uppercase, letterspaced.

---

## SVG Assets Required

Small inline SVGs or `url()` references (not full image files):

1. **Screw heads** (~16px) — radial gradient dome, crosshead slot, specular highlight
2. **Rivets** (~8px) — dome head, radial gradient
3. **"ANGENOMMEN" stamp** — circular ring with text, faded ink aesthetic
4. **Rejected slash** — thick diagonal line
5. **Brushed aluminum streaks** — repeating horizontal pattern
6. **Leather grain** — SVG noise filter (coarser than aluminum grain)

## What to Avoid

- No flat color fills
- No pure black (#000)
- No neon glows
- No glassmorphism blur
- No gradients that read as "app gradient" rather than "material depth"
- Every shadow warm-tinted, never cold grey

## Files Changed

- `src/styles/tokens.css` — full rewrite (palette, fonts, radii, shadows, button system)
- `src/App.css` — full rewrite (all component styles)
- `src/components/Background.tsx` — replace orbs with brushed aluminum + screw SVGs
- `src/assets/fonts/` — new directory with bundled font files
- No changes to component logic, hooks, types, or Tauri backend
