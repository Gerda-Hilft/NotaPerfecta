Design a skeuomorphic dark-mode UI for "NotaPerfecta", a desktop app that proofreads
German school reports (Zeugnisse). The entire interface lives in dark mode only — no
light-mode fallback.

─── AESTHETIC ────────────────────────────────────────────────────────────────────
Skeuomorphism: every surface should feel like a real physical material.
Think dark brushed aluminum bezels, aged dark leather panels, recessed ink-paper
insets, cast-metal buttons with physical depth. Lighting comes from the upper-left.
Avoid flat fills — use gradients, inner shadows, and specular highlights to fake
material depth. Textures should be subtle (noise, grain, weave) not garish.

─── PALETTE ──────────────────────────────────────────────────────────────────────
Base background: near-black with a very slight warm undertone (#1A1612)
Primary surface: dark charcoal leather with a fine grain texture (#2A2420)
Raised panels: slightly lighter warm dark (#342E29)
Accent / ink: aged amber-gold (#C9A84C) for primary actions
Text: warm off-white (#EDE8DF) on dark, warm mid-grey (#8C8278) for secondary
Danger: deep crimson (#C0392B) with a dark red tint background
Success: forest green (#27AE60) with a dark green tint background

─── MATERIALS BY ELEMENT ─────────────────────────────────────────────────────────
App frame          → dark brushed aluminum rail, subtle horizontal streaks, inset
                     screw-head details in the four corners

Drop zone          → dark leather inset with a stitched border (dashed CSS border
                     mimicking thread), embossed label "PDF hier ablegen", subtle
                     deboss effect when hovered, glow pulse on drag-over

Original text pane → aged paper inset with a very slight cream tint (#1E1A15),
                     faint ruled-line texture, feels like reading a physical document
                     inside a dark mahogany frame

Corrections pane   → dark metal clipboard feel, riveted top edge, corrections listed
                     as index-card slips with a slight paper curl shadow at the bottom

Suggestion cards   → physical index cards stacked with offset shadow. Accepted cards
                     get a green ink stamp overlay (circular, "ANGENOMMEN"). Rejected
                     cards get a red diagonal slash, reduced opacity.

Buttons (accept / reject) → chunky embossed resin buttons with a convex top face,
                     thick cast shadow below (4–6px), inner highlight on top edge,
                     press animation pushes the shadow to 1px and shifts content down

Primary CTA "PDF exportieren" → large amber-gold lacquered button, engraved label,
                     real press depth when clicked

Pipeline toggle chips → tactile radio switches in a dark metal housing; active chip
                     is backlit amber, inactive chips are recessed dark metal

Loading spinners   → analog clock / film-reel feel — a mechanical tick animation,
                     not a smooth CSS spin

Status badges      → embossed metal counters like an old odometer or typewriter key

─── TYPOGRAPHY ───────────────────────────────────────────────────────────────────
Headings: a serif with old-press character (Playfair Display or similar), warm white
Body / labels: a humanist sans (Atkinson Hyperlegible), warm off-white
Monospace text panel: a typewriter-style mono (Special Elite or Courier Prime),
  slightly aged cream on the paper inset

─── LAYOUT ───────────────────────────────────────────────────────────────────────
Two-column split below the drop zone (original text left, corrections right).
Add a top toolbar rail with the pipeline toggle and a subtle engraved title plate
reading "NotaPerfecta — Zeugnisprüfung".
Rounded corners but with a slightly tighter radius than modern flat UI (8–12px),
because physical objects have machined edges, not pill shapes.

─── WHAT TO AVOID ────────────────────────────────────────────────────────────────
No flat color fills. No pure black (#000). No neon glows. No glassmorphism blur.
No gradients that read as "app gradient" rather than "material depth".
Every shadow should be warm-tinted (#000 with a brown/amber bias), never cold grey.
