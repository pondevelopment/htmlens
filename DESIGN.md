## Overview

Pon Datalab's design system is built on the tension between intelligence and momentum — a system that feels engineered and precise while radiating confidence and disruption. The mission ("We transform each company into a digital leader within their industry") demands a visual language that is simultaneously approachable and forceful. Every layout earns attention through bold typographic hierarchy — PT Serif display lockups anchoring campaign hero moments and data-impact callouts — while everything else (navigation, cards, forms, footer) is carried by PT Sans and a restrained deep-navy/white core palette (`{colors.deep-navy}`, `{colors.canvas}`). There are no decorative gradients, no arbitrary accent splashes — the system reserves its chromatic energy (`{colors.accent-purple}`, `{colors.accent-mint}`) for the moments that genuinely signal: innovation, acceleration, a key metric, a primary conversion action.

The result is a layout that feels confident and structured — campaign hero, metric impact row, service tiles, footer — stacked like a precision-engineered report rather than decorated like a typical SaaS marketing page. Density is calibrated but never crowded, because the system relies on three relentless devices: the rounded-corner card (`{rounded.lg}` 20px) as the universal content container, a deep-navy CTA (`{component.button-primary}`) as the single action anchor per section, and an 8px-base spacing scale that keeps cards and content blocks mathematically aligned across marketing pages, presentation slides, and print materials.

Across web, slides, and posters the same chrome appears in identical proportions — only the data story and imagery change. That is the system's strength: maximum impact in the numbers and headlines, maximum restraint everywhere else.

**Key Characteristics:**
- Impact display tier: PT Serif Bold at `{typography.display-hero}` (80px, line-height 1.0, tracking -1px) for campaign hero headlines, slide title pages, and poster lockups
- Deep-navy/white/brand-blue core palette: `{colors.deep-navy}`, `{colors.canvas}`, and `{colors.brand-blue}` carry ~90% of chrome surface area
- Rounded-corner geometry: `{rounded.lg}` (20px) on every button, card, chip, and input — the visual fingerprint that connects web, slide, and print
- Accent energy reserved: `{colors.accent-purple}` and `{colors.accent-mint}` fire only at inflection points — the primary conversion CTA, a "New" badge, an active nav indicator, a highlighted metric
- Metric callout as hero unit: `{typography.metric}` (PT Serif 48px bold) paired with a 3–5-word descriptor label is the data counterpart to a campaign headline — three in a row is the system's signature impact section
- Dual-register legibility: heading sizes are calibrated to read at both 2 metres (poster/slide) and 30 cm (screen), using the same PT Serif/PT Sans pairing throughout

## Colors

> **Source surfaces:** Marketing website, presentation slide templates, A1/A0 poster layouts. The chrome palette is identical across all three — only content and imagery vary.

### Brand Core
- **Pon Deep Blue** (`{colors.deep-navy}` — `#002c47`): The brand's gravitational center. Primary CTAs, headings, nav, labels, footer background, and primary text on light surfaces. When Datalab wants to assert, it goes deep navy.
- **Pon Blue Tint** (`{colors.brand-blue}` — `#dae7fe`): The signature atmospheric surface. Section backgrounds, card image area fills, hero wash layers, search field background. Warm but professional — the "sky" behind every data story.
- **Canvas** (`{colors.canvas}` — `#ffffff`): Content card interiors, page backgrounds, modal surfaces. The stage for data and copy.

### Accent / Energy
- **Popping Purple** (`{colors.accent-purple}` — `#b399ff`): Innovation and disruption signal. Accent chips, highlighted metric borders, "New" badge backgrounds, select hero accent marks, active nav underline alternative. Never used for body text; never used as a large section background.
- **Popping Mint** (`{colors.accent-mint}` — `#52e9c0`): Acceleration and success signal. The single highest-priority CTA background per page, positive metric borders, data success indicators, forward-momentum decorative elements. The system's "go" color.

### Suggested Neutrals
- **Off-White** (`{colors.off-white}` — `#f0f4f8`): Alternate surface for alternating section rows, table row backgrounds, inactive chip fill.
- **Text Secondary** (`{colors.text-secondary}` — `#5c7185`): Secondary body text, captions, form helper text, card metadata. Full `{colors.deep-navy}` body copy is too heavy for long reads — use this for body paragraphs.
- **Hairline** (`{colors.hairline}` — `#d1dde8`): 1px dividers between content rows, card borders on `{colors.canvas}`, table separators.

### Semantic
- **Success** (`{colors.success}` — `#52e9c0`): Mirrors `{colors.accent-mint}` — positive data trends, form completion states, confirmation messages.
- **Error** (`{colors.error}` — `#ef4444`): Data anomalies, validation errors, destructive action warnings.
- **Warning** (`{colors.warning}` — `#f59e0b`): Data health alerts, threshold warnings, degraded-state indicators.
- **Info** (`{colors.info}` — `#b399ff`): Mirrors `{colors.accent-purple}` — informational banners, tooltip accents, non-critical callouts.

### Dark Mode (Web Implementation)
- **Theme trigger:** Web UI uses `html[data-theme="dark"]` token remapping for surfaces, borders, text, links, tables, status cards, and code blocks.
- **Default behavior:** If no user preference exists, theme follows `prefers-color-scheme`.
- **User override:** Theme toggle persists preference in `localStorage` (`htmlens-theme`).
- **Flash prevention:** Theme is applied in `<head>` before first paint to avoid light/dark flicker.
- **Dark surface model:** Primary backgrounds shift to deep navy surfaces (`--color-surface-dark-code-strong`, `--color-surface-dark-code`, `--color-surface-dark-input`) while semantic states (success/warning/error) keep meaning via border + text contrast instead of full-saturation fills.

## Typography

### Font Family
- **PT Serif Bold** (display, headings, and metric numbers) — The editorial anchor. Serif confidence signals expertise and earned authority. Used for campaign-scale display headlines, section hero titles, major page headings, and all quantitative impact callouts. Loaded via Google Fonts (`font-family: 'PT Serif', serif`).
- **PT Sans Medium / Regular** (body, UI, small titles) — The system's workhorse. A designed pairing with PT Serif. Used for all body copy, button labels, captions, nav links, form inputs, and utility text. Loaded via Google Fonts (`font-family: 'PT Sans', sans-serif`).

When substituting: **Playfair Display** for PT Serif (comparable serif authority); **Lato** or **Source Sans 3** for PT Sans.

### Hierarchy

| Token | Font | Size | Weight | Line Height | Letter Spacing | Use |
|---|---|---|---|---|---|---|
| `{typography.display-hero}` | PT Serif | 80px | 700 | 1.0 | -1px | Campaign hero headline — web hero section, poster lockup, slide title page |
| `{typography.display-impact}` | PT Serif | 56px | 700 | 1.1 | -0.5px | Section hero headline, mission statement pull-quote, slide section divider |
| `{typography.heading-xl}` | PT Serif | 36px | 700 | 1.2 | 0 | Major section titles, slide content headers |
| `{typography.heading-lg}` | PT Serif | 28px | 700 | 1.3 | 0 | Subsection headers, card feature titles, slide sub-headers |
| `{typography.heading-md}` | PT Sans | 20px | 700 | 1.4 | 0 | Card titles, form section labels, small modal titles |
| `{typography.heading-sm}` | PT Sans | 16px | 700 | 1.5 | 0 | Label headings, accordion row labels, table headers |
| `{typography.body-lg}` | PT Sans | 18px | 400 | 1.6 | 0 | Lead / introductory body copy, slide body |
| `{typography.body-md}` | PT Sans | 16px | 400 | 1.6 | 0 | Standard body paragraphs, list items, card descriptions |
| `{typography.body-strong}` | PT Sans | 16px | 700 | 1.6 | 0 | Emphasized inline text, key stat labels, nav links |
| `{typography.button-lg}` | PT Sans | 18px | 700 | 1.4 | 0.5px | Large CTAs in hero sections |
| `{typography.button-md}` | PT Sans | 15px | 700 | 1.4 | 0.5px | Standard button labels across the system |
| `{typography.button-sm}` | PT Sans | 13px | 700 | 1.4 | 0.5px | Compact chip labels, badge text, tag labels |
| `{typography.caption-md}` | PT Sans | 14px | 400 | 1.5 | 0 | Card metadata, form helper text, footer links, slide captions |
| `{typography.caption-sm}` | PT Sans | 12px | 400 | 1.5 | 0 | Fine print, legal copy, utility row text |
| `{typography.metric}` | PT Serif | 48px | 700 | 1.0 | -1px | Data hero numbers — "87% faster", "3× ROI", "40+ companies" |
| `{typography.metric-sm}` | PT Serif | 32px | 700 | 1.1 | -0.5px | Smaller data callouts in card or sidebar contexts |

### Principles
The system runs on a deliberate serif/sans bifurcation: PT Serif owns every headline and impact number; PT Sans carries all functional text. This creates visual rhythm without requiring decorative elements — a reader immediately knows that serif means "headline or key fact," and sans means "functional." The `{typography.metric}` tier is the system's data counterpart to a campaign hero headline — a 48px PT Serif Bold number with a short PT Sans label below it communicates Datalab's value faster than any paragraph can.

Letter spacing is tightened at the display and metric tiers (-1px at 80px, -0.5px at 56px/32px) to add editorial density. Button labels use a slight positive tracking (+0.5px) for legibility at small sizes and to signal "action" versus "content."

### Note on Slide & Print Sizes
- **Presentation slides (16:9, 1920×1080pt equivalent):** `{typography.display-hero}` → 60pt; `{typography.display-impact}` → 42pt; `{typography.heading-xl}` → 28pt; `{typography.body-md}` → 16pt.
- **Posters (A1 at 150dpi):** `{typography.display-hero}` scales to 120–160pt. All font families remain PT Serif / PT Sans — embed both at export.

## Layout

### Spacing System
- **Base unit:** 8px
- **Tokens:** `{spacing.xxs}` (2px) · `{spacing.xs}` (4px) · `{spacing.sm}` (8px) · `{spacing.md}` (16px) · `{spacing.lg}` (24px) · `{spacing.xl}` (32px) · `{spacing.xxl}` (48px) · `{spacing.section}` (64px+)
- **Universal rhythm:** Section-to-section vertical gaps use `{spacing.section}` (64px) on web, 40pt on slides. Internal card padding uses `{spacing.lg}` (24px) on all sides — Datalab cards breathe because data and copy need white space to resolve. Metric callout groups use `{spacing.xl}` (32px) above and below the number.
- **Inline stacking:** Within a card: heading → `{spacing.sm}` (8px) → body copy → `{spacing.lg}` (24px) → CTA.
- **Print bleed:** Poster layouts allow a 3mm bleed on all sides. Section background colors extend to bleed; logo and critical text stay within a 20mm safe margin.

### Grid & Container
- **Max width:** 1280px content area with 32px edge gutters at desktop, growing to 48px at 1440px+.
- **Column system:** 12-column grid. Marketing sections use 2-up (6+6), 3-up (4+4+4), or asymmetric hero layouts (5+7 or 7+5). Slide templates use the same 12-column proportional logic at full slide width.
- **Card grids:** 3-up at desktop, 2-up at tablet (768–1023px), 1-up at mobile with `{spacing.md}` (16px) gutters throughout.
- **Metric callout row:** 3 `{component.metric-callout}` units in a 4+4+4 column row is the standard "impact" layout. On slides, use 3 across at equal widths.

### Whitespace Philosophy
Whitespace is purposeful and generous inside content containers, tight between section blocks. Cards carry `{spacing.lg}` (24px) internal padding so data and copy have room to breathe. Section backgrounds alternate between `{colors.canvas}` and `{colors.brand-blue}` — this background-color shift is the visual section divider; no decorative horizontal rules are needed between sections.

## Elevation & Depth

| Level | Treatment | Use |
|---|---|---|
| 0 — Flat | No shadow, no border | Full-bleed hero sections, section backgrounds |
| 1 — Subtle card | `box-shadow: 0 2px 8px rgba(0,44,71,0.08)` | Default card elevation — soft deep-navy-tinted shadow on both `{colors.canvas}` and `{colors.brand-blue}` |
| 2 — Raised card | `box-shadow: 0 4px 16px rgba(0,44,71,0.14)` | Hovered/focused card, featured card, modal dialog |
| 3 — Overlay | `box-shadow: 0 8px 32px rgba(0,44,71,0.22)` | Dropdown menus, tooltip panels |

Unlike a fully flat system, Datalab allows a subtle deep-navy-tinted shadow at the card level. This grounds cards without competing with content, and the navy tint keeps shadows on-brand rather than neutral gray. Shadow elevation should never exceed Level 2 in marketing layouts; Level 3 is reserved for transient overlay UI.

### Depth on Print & Slides
Drop shadows do not translate reliably to print or low-resolution projectors. For poster and slide layouts, replace card shadow with a `1px solid {colors.hairline}` border on `{colors.canvas}` cards, or rely on the `{colors.brand-blue}` background contrast to visually separate cards.

## Shapes

### Border Radius Scale

| Token | Value | Use |
|---|---|---|
| `{rounded.none}` | 0px | Full-bleed section backgrounds and horizontal rule dividers only — never on interactive elements |
| `{rounded.sm}` | 8px | Inline badge containers, code snippet tags |
| `{rounded.md}` | 12px | Form inputs, search fields, image containers within cards |
| `{rounded.lg}` | 20px | **The signature radius.** All cards, all buttons, all chips, metric callout panels, service tiles, feature containers — every interactive and content surface in the system |
| `{rounded.xl}` | 32px | Large illustration panels, hero content inset containers, oversized accent blocks |
| `{rounded.full}` | 9999px | Avatar / icon circles, circular icon buttons (close, back, carousel), brand logo container |

**`{rounded.lg}` (20px) is Datalab's visual fingerprint** — the equivalent of Nike's pill. It must appear on every button, card, and chip across web, slides, and posters. Consistency of this radius across all surfaces is what makes the system feel unified.

## Components

> Each spec covers Default and Active/Pressed states. Variants live as separate entries. Hover states are documented where they affect layout (e.g., card shadow elevation) but micro-animation timing is out of scope.

### Buttons

**`button-primary`** — deep-navy CTA
- Background `{colors.deep-navy}`, text `{colors.canvas}`, type `{typography.button-md}`, padding `14px 28px`, height 48px, rounded `{rounded.lg}` (20px).
- Used for all primary actions: "Get Started", "Book a Demo", "Learn More", "Download Report".
- Pressed: background lightens slightly to `#004070`, `scale(0.98)`.

**`button-accent`** — mint green conversion CTA
- Background `{colors.accent-mint}`, text `{colors.deep-navy}`, type `{typography.button-md}`, padding `14px 28px`, height 48px, rounded `{rounded.lg}`.
- Reserved for the single highest-priority conversion action per page or slide — "Start Your Digital Journey", "See It in Action". **One per layout maximum.**
- Pressed: background `#3dd4ab`, `scale(0.98)`.

**`button-secondary`** — outlined alternative
- Background transparent, border `2px solid {colors.deep-navy}`, text `{colors.deep-navy}`, type `{typography.button-md}`, padding `14px 28px`, height 48px, rounded `{rounded.lg}`.
- Paired alongside `button-primary` as the soft alternative ("Watch Demo" next to "Book a Demo").

**`button-ghost-on-dark`** — on deep-navy or dark image backgrounds
- Background transparent, border `2px solid {colors.canvas}`, text `{colors.canvas}`, type `{typography.button-md}`, height 48px, rounded `{rounded.lg}`.
- Used when the background is `{colors.deep-navy}` — never on light surfaces.

**`button-icon-circular`** — icon controls
- Background `{colors.brand-blue}` or `{colors.off-white}`, icon `{colors.deep-navy}`, rounded `{rounded.full}`, size 40px.
- Back arrow, carousel paddle, modal close, social share.

**`chip`** + **`chip-active`**
- Default: background `{colors.off-white}`, text `{colors.deep-navy}`, border `1px solid {colors.hairline}`, type `{typography.button-sm}`, rounded `{rounded.lg}`, padding `6px 14px`.
- Active: background `{colors.deep-navy}`, text `{colors.canvas}` — full inversion. No middle state.
- Used for filter tags, category selectors, service area filters.

**`chip-accent`**
- Background `{colors.accent-purple}`, text `{colors.deep-navy}`, type `{typography.button-sm}`, rounded `{rounded.lg}`, padding `6px 14px`.
- "New", "AI-powered", "Featured" highlight tags on service cards. Maximum 2 visible per section.

### Inputs & Forms

**`input-field`**
- Background `{colors.canvas}`, border `1.5px solid {colors.hairline}`, type `{typography.body-md}`, rounded `{rounded.md}` (12px), padding `12px 16px`, height 48px.
- Focused: border `2px solid {colors.deep-navy}`, `box-shadow: 0 0 0 4px rgba(0,44,71,0.12)`.
- Error: border `2px solid {colors.error}`, helper text `{typography.caption-md}` in `{colors.error}` below the field.

**`search-field`**
- Background `{colors.brand-blue}`, border none, type `{typography.body-md}`, rounded `{rounded.lg}` (20px), padding `12px 20px`, height 48px. Magnifier icon left-aligned inside at `{colors.deep-navy}`.
- Focused: border `2px solid {colors.deep-navy}`.

### Cards & Containers

**`service-card`** — the primary content unit
- Container: background `{colors.canvas}`, rounded `{rounded.lg}` (20px), padding `{spacing.lg}` (24px), shadow `{elevation.1}`.
- Structure (top to bottom): 48px icon or illustration on a `{colors.brand-blue}` or `{colors.accent-purple}` circular (`{rounded.full}`) background → heading `{typography.heading-lg}` `{colors.deep-navy}` → body copy `{typography.body-md}` `{colors.text-secondary}` → optional CTA link `{typography.body-strong}` `{colors.deep-navy}` underlined.
- Hover: shadow transitions to `{elevation.2}`.

**`metric-callout`** — the system's data hero unit
- Container: background `{colors.canvas}` or `{colors.brand-blue}`, rounded `{rounded.lg}` (20px), padding `{spacing.xl}` (32px).
- Structure: large number `{typography.metric}` `{colors.deep-navy}` → descriptor label `{typography.caption-md}` `{colors.text-secondary}` (3–5 words maximum).
- Accent variant: left border `4px solid {colors.accent-mint}` for positive/forward metrics; `4px solid {colors.accent-purple}` for neutral/informational metrics.
- **Three metric callouts in a row is the standard impact section.** This layout is the system's signature momentum moment.

**`hero-section`** — campaign-level impact block
- Full-width section background in `{colors.deep-navy}` or `{colors.brand-blue}`.
- Content: `{typography.display-hero}` headline in `{colors.canvas}` (on deep-navy) or `{colors.deep-navy}` (on brand-blue) → sub-headline `{typography.body-lg}` → CTA pair (`{component.button-accent}` + `{component.button-ghost-on-dark}` or `{component.button-secondary}`).
- The display headline may span 2–3 lines at the designated size — this editorial depth is intentional.

**`feature-tile`** — mixed image + text editorial unit
- Full-bleed section image (or solid `{colors.brand-blue}` fill) on one half; content slot on the other.
- Content slot: `{typography.heading-xl}` `{colors.deep-navy}` → `{typography.body-md}` body copy `{colors.text-secondary}` → `{component.button-primary}`.
- Alternates left/right image positioning across consecutive tiles to create visual rhythm.

**`testimonial-card`**
- Background `{colors.brand-blue}`, rounded `{rounded.lg}` (20px), padding `{spacing.xl}` (32px).
- Large decorative quote mark in `{colors.accent-purple}` → quote text `{typography.body-lg}` `{colors.deep-navy}` italic → attribution `{typography.caption-md}` `{colors.text-secondary}`.

**`badge-new`** + **`badge-featured`**
- Background `{colors.accent-purple}` (New) or `{colors.accent-mint}` (Featured), text `{colors.deep-navy}`, type `{typography.button-sm}`, rounded `{rounded.sm}` (8px), padding `4px 10px`.
- Sits top-left on `{component.service-card}` for newly launched capabilities. Maximum 2 badges visible per section.

### Navigation

**`primary-nav`**
- Background `{colors.canvas}` (default light) or `{colors.deep-navy}` (dark variant for hero overlay).
- Logo left-aligned; nav links center or right-aligned in `{typography.body-strong}` `{colors.deep-navy}`; primary CTA `{component.button-primary}` far right.
- Active nav link: 2px bottom border in `{colors.accent-mint}` — the mint underline is the active signal (not a background fill, not deep-navy).
- Height: 64px, rounded `{rounded.none}` (nav bar is edge-to-edge).

**`footer`**
- Background `{colors.deep-navy}`, text `{colors.canvas}`.
- Top: Datalab logo + tagline ("We transform each company into a digital leader within their industry") left-aligned.
- Link columns: column header `{typography.heading-sm}` `{colors.canvas}` → link list `{typography.caption-md}` `rgba(255,255,255,0.65)` (brightens to `{colors.canvas}` on hover).
- Bottom row: `{typography.caption-sm}` `rgba(255,255,255,0.45)` for copyright, legal links, and Pon holding attribution.

**`nav-mobile`** — mobile navigation
- Hamburger icon left (40px touch target, `{rounded.full}`), Datalab logo center, CTA icon or "Book Demo" text right.
- Tapping hamburger opens a full-height drawer from the left: nav links stacked at `{spacing.xl}` (32px) vertical padding, background `{colors.deep-navy}`, text `{colors.canvas}`.

### Signature Components

**`slide-title-page`** — presentation title slide
- Full background `{colors.deep-navy}`.
- `{typography.display-hero}` headline `{colors.canvas}` → `{typography.body-lg}` subtitle `rgba(255,255,255,0.75)` → presenter name and date `{typography.caption-md}` `rgba(255,255,255,0.5)`.
- Datalab logo white reverse version, bottom-left.
- Optional: a thin `4px` accent bar in `{colors.accent-mint}` at the very top edge of the slide.

**`slide-section-divider`** — presentation section break slide
- Full background `{colors.deep-navy}`.
- Single `{typography.display-impact}` section title `{colors.canvas}` centered vertically.
- `{chip-accent}` label (section number or category) above the heading in `{colors.accent-purple}`.

**`slide-content`** — standard content slide
- Background `{colors.canvas}` or alternating `{colors.brand-blue}`.
- `{typography.heading-xl}` slide title `{colors.deep-navy}` → body content area in `{typography.body-md}` `{colors.text-secondary}`.
- Slide number `{typography.caption-sm}` `{colors.text-secondary}` bottom-right; Datalab logo `{colors.deep-navy}` bottom-left at minimum size (25mm print equivalent).

**`poster-lockup`** — A1/A0 print poster
- Background `{colors.brand-blue}` or `{colors.deep-navy}` (full bleed to 3mm).
- `{typography.display-hero}` headline (scaled to 120–160pt at print DPI) → `{typography.display-impact}` sub-headline → `{component.metric-callout}` row of 3.
- `{component.button-accent}` shape rendered as a static decorative element (no interactivity) with URL or QR code beside it.
- Datalab logo at minimum 25mm width, positioned in a `20mm` safe margin from edges.

## Do's and Don'ts

### Do
- Use `{typography.display-hero}` PT Serif exclusively for campaign hero moments: the web hero section headline, the slide title page, the poster main lockup — never for section sub-headers or card titles.
- Apply `{rounded.lg}` (20px) to every button, card, chip, and input. It's the system's visual fingerprint. Deviate only to `{rounded.xl}` for large panel containers or `{rounded.full}` for circular elements.
- Use `{component.metric-callout}` in groups of three to communicate scale: "87% faster · 3× ROI · 40+ companies". This three-up layout is the system's signature impact moment.
- Reserve `{component.button-accent}` (mint green) for the single highest-priority conversion CTA per page or slide. One per layout — its scarcity is its power.
- Keep body copy in `{colors.text-secondary}` (`#5c7185`) on `{colors.canvas}` surfaces — full `{colors.deep-navy}` at body sizes is too heavy for long reads.
- Use `{colors.brand-blue}` sections as visual breathing rooms between dense content sections — alternating canvas/brand-blue background rhythm replaces decorative dividers.
- Embed PT Serif and PT Sans from Google Fonts in all digital outputs; embed both font files in print-ready PDFs and PowerPoint exports.

### Don't
- Don't use `{rounded.full}` (pill) on rectangular buttons. Datalab's signature is `{rounded.lg}` (20px), not full pill. Reserve `{rounded.full}` strictly for circular icon buttons and avatar containers.
- Don't place `{colors.accent-purple}` or `{colors.accent-mint}` as large section backgrounds — they belong as accent marks, borders, chip fills, and badge backgrounds only.
- Don't set body copy in PT Serif — PT Serif is exclusively for headings and metric numbers. All functional text is PT Sans.
- Don't use more than two accent-colored elements per visible viewport — accent scarcity is what gives them energy. If everything is highlighted, nothing is.
- Don't introduce sharp corners (`{rounded.none}`) on interactive elements. All buttons, chips, cards, and form inputs use `{rounded.lg}` or greater.
- Don't use `{colors.accent-mint}` or `{colors.accent-purple}` for small body text — they fail WCAG AA contrast on `{colors.canvas}` at sizes below 24px. Use the semantic tokens (`{colors.success}`, `{colors.info}`) in UI contexts and reserve the accent colors for graphic/decorative use.
- Don't crowd `{component.metric-callout}` with explanatory copy — the number is the hero. The label must be 3–5 words maximum.
- Don't place the Datalab logo on `{colors.accent-purple}` or `{colors.accent-mint}` backgrounds — insufficient contrast. Use `{colors.deep-navy}` (logo reversed white) or `{colors.canvas}` / `{colors.brand-blue}` (logo in `{colors.deep-navy}`).

## Responsive Behavior

### Breakpoints

| Name | Width | Key Changes |
|---|---|---|
| ultrawide | 1920px+ | Max-width 1280px holds; outer gutters grow to ~64px on each side |
| desktop-large | 1440px | Default layout — 3-up card grid, full metric row, side-by-side feature tiles, full nav |
| desktop | 1280px | Same layout, gutters at 32px |
| desktop-small | 1024px | Feature tiles start compressing; metric callouts can stay 3-up |
| tablet | 768–1023px | 2-up card grid; nav collapses to hamburger; hero headline scales to `{typography.display-impact}` (56px) |
| mobile | 375–767px | 1-up card grid; metric callouts stack to 1-up; hero headline scales to `{typography.heading-xl}` (36px); CTA buttons go full-width |

### Slide & Print Adaptations
- **Presentation slides (16:9, 1920×1080):** Use the slide-specific typography mapping in the Typography section. Maintain the hex color values exactly. Avoid drop shadows — replace with `1px solid {colors.hairline}` borders on cards.
- **Posters (A1 at 150dpi+):** Display headline scales to 120–160pt. Verify `{colors.accent-mint}` (`#52e9c0`) and `{colors.accent-purple}` (`#b399ff`) against the print vendor's gamut before final output — these are vibrant RGB values that may need gamut mapping for offset printing. `{colors.brand-blue}` (`#dae7fe`) converts safely to CMYK (approx. C:10 M:7 Y:0 K:1).

### Collapsing Strategy
- **Nav:** Full horizontal → hamburger drawer at 1023px.
- **Hero headline:** 80px → 56px → 36px across desktop / tablet / mobile; line-height stays at 1.0–1.2 across all sizes.
- **Metric callouts:** 3-up → 2-up → 1-up at tablet and mobile respectively.
- **Feature tiles:** Side-by-side → stacked (image top, content below) at 1023px.
- **Card grids:** 3-up → 2-up → 1-up with `{spacing.md}` (16px) gutters throughout.
- **Section spacing:** `{spacing.section}` 64px desktop → 48px tablet → 32px mobile.
- **CTA buttons:** Fixed width (auto) on desktop → full-width on mobile for maximum tap area.

### Touch Targets
All interactive elements target WCAG AA minimums (44×44px). `{component.button-primary}` and `{component.button-accent}` sit at 48px height with 28px horizontal padding. `{component.button-icon-circular}` at 40px has tap-target padding extending the tappable area to 48px. Chip elements sit at 40px height.

## Logo Usage

The Datalab wordmark (Pon Datalab) is the primary brand asset and must be handled consistently across all surfaces.

- **Clearspace:** Equal to the cap-height of the "D" in "Datalab" on all four sides — approximately 12px digital / 4mm print at standard sizes.
- **On-light surfaces** (`{colors.canvas}`, `{colors.brand-blue}`, `{colors.off-white}`): Full-color version in `{colors.deep-navy}`.
- **On-dark surfaces** (`{colors.deep-navy}` backgrounds): White reverse version in `{colors.canvas}`.
- **Minimum size:** 120px wide on digital; 25mm wide at print.
- **Forbidden placements:** Do not place the logo on `{colors.accent-purple}` or `{colors.accent-mint}` backgrounds (insufficient contrast). Do not stretch, rotate, recolor, or apply drop shadows to the logo.
- **Co-branding with Pon:** When appearing alongside the Pon holding logo, place the Pon logo to the left with a `1px solid {colors.hairline}` vertical divider between them, with equal clearspace from the divider on each side.

## Iteration Guide

1. Establish the background color first (`{colors.canvas}`, `{colors.brand-blue}`, or `{colors.deep-navy}`) — this immediately determines which button variant, text color, and logo version to reach for.
2. Apply `{rounded.lg}` (20px) to every new card or interactive element before considering any other radius. Only deviate to `{rounded.xl}` for large panel containers or `{rounded.full}` for circles/avatars.
3. Place one `{component.metric-callout}` group (3-up) early in any marketing layout — impact numbers are the fastest path to communicating Datalab's value proposition.
4. Reach for `{typography.metric}` (PT Serif 48px) for any quantitative proof point before reaching for a chart or data visualization.
5. Reserve `{component.button-accent}` (mint green) for the single most important CTA on a page or slide. If the layout has no primary conversion moment, leave the accent CTA out.
6. For slide decks: use `{colors.deep-navy}` for title slides and section divider slides; use `{colors.canvas}` with `{colors.brand-blue}` accent sections for content slides.
7. Check contrast before shipping: `{colors.accent-mint}` on `{colors.canvas}` passes AA at 24px+ (large text) but fails for body text. `{colors.accent-purple}` similarly — use only for decorative or graphic contexts below 24px.
8. When introducing a new component, verify it can be expressed using the existing vocabulary (`{rounded.lg}` card, `{component.button-primary}`, `{typography.metric}`, `{colors.brand-blue}` fill) before adding new tokens. The system's strength is disciplined reuse.

## Known Gaps

- **Data visualization palette:** Charts, graphs, and dashboard widgets used within Pon Datalab's products need a dedicated dataviz palette. Recommended starting point: sequential scale from `{colors.brand-blue}` → `{colors.deep-navy}`, with `{colors.accent-mint}` and `{colors.accent-purple}` as callout/highlight series colors. Categorical palette needs 5–6 distinct hues.
- **Dark mode for slide/print outputs:** Web dark mode is implemented, but slide and print deliverables still define only the light export standard. A dedicated dark slide/poster variant (token map + contrast guardrails) is still pending.
- **Icon system:** Not yet defined. Recommended: consistent outline-weight icon set at 2px stroke with rounded caps and joins matching `{rounded.sm}` to align with the system's rounded-corner language.
- **Motion & animation:** Not specified. Recommended baseline: ease-out transitions at 200ms for button state changes; 300ms for card hover elevation; 400ms for section/slide transitions.
- **Dutch / NL language adaptations:** As Pon is a Netherlands-based company, all layouts should accommodate Dutch text, which runs approximately 30% longer than English on average. Avoid fixed-width text containers below `{typography.heading-md}` — allow text containers to flex rather than truncate.
- **Mobile app patterns:** This document covers web, slides, and print. Native mobile (iOS/Android) patterns for any Datalab product apps are not specified here and would require a separate component specification.
