# VirtualPet.in Website Analysis and Implementation Plan

Date: 29 August 2026

## 1. Scope and working assumption

This plan covers rebuilding the public marketing and customer-download website represented by [virtualpet.in](https://virtualpet.in/). It includes the landing page, order handoff, activation-code lookup, product download/instruction pages, legal pages, analytics, SEO, accessibility, and deployment.

It does **not** include building the macOS/Windows Virtual Cat or Virtual Puppy desktop applications. Those binaries, their in-app activation behavior, signing/notarization, and release pipeline are dependencies that must be supplied or scoped separately.

The workspace is currently empty and is not a Git repository, so implementation starts with repository bootstrap.

## 2. Current-site analysis

### 2.1 User proposition

The site sells two distinct desktop companions:

- Virtual Cat: slower, aloof behavior, laser play, grooming, and a cat bed.
- Virtual Puppy: eager cursor following, ball play, and a kennel.
- Custom Pet: a personalized version based on a customer's pet photo.

The primary conversion path is:

1. Visitor lands on the marketing page.
2. Visitor learns the shared features and animal-specific behaviors.
3. Visitor compares three pricing cards.
4. Visitor opens a Google Form and submits contact/order details.
5. The seller manually handles payment and sends an activation code.
6. The customer enters the code on the “My Download” page.
7. The site reveals the download page for the purchased product.
8. The customer downloads a macOS or Windows installer and activates the desktop app.

### 2.2 Information architecture

| Route | Purpose | Current behavior |
| --- | --- | --- |
| `/index.html` | Marketing and conversion | Single-page landing with anchor navigation |
| `/login.html` | Activation-code lookup | Calls Google Apps Script through JSONP and reveals product links |
| `/cat.html` | Cat downloads and installation help | Publicly accessible Mac and Windows links |
| `/puppy.html` | Puppy downloads and installation help | Publicly accessible Mac and Windows links |
| `/privacy.html` | Privacy policy | Static legal content |
| `/terms.html` | Terms and conditions | Static legal content |
| Google Form | Lead/order capture | External form; no payment is collected |

The landing page contains these sections in order:

1. Header/navigation
2. Hero offer, value proposition, calls to action, and autoplay product demo
3. Six shared feature cards
4. Cat-versus-puppy behavior comparison
5. Three pricing cards
6. Google Form order handoff
7. Native `<details>` FAQ
8. Footer and legal links

### 2.3 Visual system

The visual identity is coherent and worth preserving:

- Display font: `Press Start 2P`
- Body font: `VT323`
- Warm cream background with pink/orange radial gradients
- Orange primary brand color, blue conversion buttons, dark brown outlines
- Three-pixel borders and hard four-to-eight-pixel shadows
- Pixelated image rendering for sprites and video
- Desktop content width of approximately `1120px`
- Desktop grids: two-column hero, three-column features, two-column pet comparison, and three-column pricing
- Small stepped reveal animations and tactile hover/active button movement

Core color tokens observed on the live site:

| Token | Value | Use |
| --- | --- | --- |
| Background | `#fff1d6` | Page base |
| Background accent | `#ffe1b0` | Pattern/illustration panels |
| Panel | `#fffdf6` | Cards |
| Ink | `#241611` | Borders, shadows, strong text |
| Text | `#2b1b11` | Body copy |
| Muted | `#5a4230` | Supporting copy |
| Brand | `#ff7a1a` | Icons and primary accent |
| Brand dark | `#dc4c00` | Headings |
| CTA blue | `#2f66f0` | Purchase/download calls to action |
| Green | `#2f9e44` | Savings and success states |
| Sky | `#aadcff` | Demo/step panels |

### 2.4 Technical implementation observed

The live site is a small static implementation:

- Hand-authored HTML with page-local CSS and JavaScript
- No frontend framework or visible build pipeline
- Hostinger hosting behind Hostinger CDN
- Google Fonts, Google Analytics, Google Forms, and Google Apps Script
- `IntersectionObserver` for reveal-on-scroll
- Native `<details>/<summary>` for FAQs
- A 1280×804 H.264 hero video, about 75.7 seconds and 1.68 MB
- Pixel-art PNGs ranging from about 20 KB to 301 KB each
- Seven-day browser caching on inspected PNG assets

This content does not need React, a SPA router, a CMS, or a component framework. A static multi-page site with shared CSS and small JavaScript modules is the shortest maintainable solution.

## 3. Findings to resolve before implementation

### 3.1 Release-blocking contract inconsistencies

These are content/business decisions, not cosmetic edits. One canonical answer must be approved and used everywhere.

| Issue | Current conflict | Required decision |
| --- | --- | --- |
| Platform entitlement | Landing page, FAQ, and download pages say one purchase covers both macOS and Windows; Terms section 1 says the customer chooses one platform | Confirm whether one code licenses both platforms |
| Product selection | Site sells cat and puppy as separate apps/codes; the live Google Form only offers “The App” or “Custom Pet” | Add explicit Cat, Puppy, and Custom Pet choices |
| Computer selection | Google Form requires Mac or Windows even though the site says the same purchase works on both | Remove this field or make it optional support information |
| Device allowance | Marketing says up to three computers; Terms says a personal license on “your own computer” | State the same three-device allowance in the license |
| Custom Pet species | Landing page says cat or dog; Privacy and Terms repeatedly say dog/photo of dog | Confirm cat, dog, or both and align every page |
| Download protection | “My Download” looks gated, but download pages and binary URLs are public | Decide whether the gate is only a convenience or a security boundary |
| Website pet demo | The page contains hidden “move your mouse” copy but no cursor-following web pet implementation | Build a lightweight demo or remove the dead hint |
| App privacy claim | Privacy says the app transmits no data, while activation is required | Verify what activation sends before publishing this claim |

Do not start production copy or activation work until these decisions are recorded.

### 3.2 High-priority implementation defects

1. **Mobile reflow is broken.** At a tested 390px viewport, the hero grid expands beyond the viewport and clips body copy and buttons. The likely causes are grid/flex min-content sizing and unbreakable pixel-font labels. The rebuild must use `minmax(0, 1fr)`, `min-width: 0`, mobile-width buttons, and explicit overflow tests down to 320px.
2. **Activation is not a security gate.** Anyone can open `/cat.html`, `/puppy.html`, and the installer URLs directly. This is acceptable only if the desktop app is the real licensing boundary.
3. **The login page executes a remote JSONP response as JavaScript.** Replace JSONP with same-origin `POST`/JSON or a CORS-enabled JSON endpoint. Never put activation codes in analytics or logs beyond what is operationally required.
4. **Installer trust is weak.** The current instructions tell macOS users to remove quarantine metadata and Windows users to bypass SmartScreen. Production binaries should be Apple-notarized and Windows code-signed; publish version, file size, and SHA-256 checksums.
5. **The macOS DMG is served as `text/plain`.** Configure `application/x-apple-diskimage` or `application/octet-stream` and preserve range requests.
6. **Reveal content starts hidden.** If JavaScript fails before the observer is installed, core cards remain invisible. Content should be visible by default; JavaScript may add an enhancement class before applying reveal styles.

### 3.3 Accessibility and responsive gaps

- No skip-to-content link
- No consistent `:focus-visible` treatment for links and CTA elements
- Several display labels are only 9–10px and may be difficult to read
- The 390px clipping fails the expected reflow behavior; the implementation must also pass at 320px
- Reduced-motion CSS disables reveal transitions but does not stop the autoplay video
- The video has an `aria-label` but no poster, transcript/adjacent description, or non-motion fallback
- Home-page logo is not a link while inner-page logos are
- Responsive behavior relies on a few broad breakpoints and does not constrain long labels

Target WCAG 2.2 AA, including keyboard operation, visible focus, meaningful headings/labels, and no loss of content at a 320 CSS-pixel width. Reference: [W3C WCAG 2.2](https://www.w3.org/TR/WCAG22/) and [WCAG quick reference](https://www.w3.org/WAI/WCAG22/quickref/).

### 3.4 SEO, sharing, and discoverability gaps

- `robots.txt` returns 404
- `sitemap.xml` returns 404
- No canonical links
- No Open Graph or Twitter sharing metadata
- No social preview image
- No Product or SoftwareApplication structured data
- No visible 404 page
- Page-specific metadata exists and is a good base, but route/index coverage is incomplete

### 3.5 Performance and maintainability gaps

- Roughly 1.27 MB of inspected PNG art plus a 1.68 MB hero video can load on the landing page
- Below-the-fold images lack explicit `width`/`height` and lazy-loading hints
- The 75-second autoplay loop is longer than needed for a quick product demonstration
- Video has no poster and no reduced-data/reduced-motion strategy
- Styles are duplicated across every HTML page
- Inline CSS and JavaScript make a restrictive Content Security Policy harder
- Assets can use immutable, content-hashed filenames and longer cache lifetimes

Performance acceptance targets should use the current Core Web Vitals thresholds at the 75th percentile: LCP ≤ 2.5s, INP ≤ 200ms, and CLS ≤ 0.1. Reference: [web.dev Web Vitals](https://web.dev/articles/vitals).

## 4. Recommended architecture

### 4.1 Baseline choice

Use a static multi-page website with native HTML, CSS, and JavaScript.

Why:

- The content changes infrequently.
- All public pages are document-oriented.
- Search engines and no-JavaScript clients should receive complete HTML.
- The only dynamic behavior is reveal animation, FAQ toggles, analytics, and activation lookup.
- It deploys directly to the existing Hostinger-style environment.
- It introduces no framework, runtime, hydration, or dependency maintenance.

### 4.2 Proposed file layout

```text
/
├── index.html
├── login.html
├── cat.html
├── puppy.html
├── privacy.html
├── terms.html
├── 404.html
├── robots.txt
├── sitemap.xml
├── assets/
│   ├── css/
│   │   └── site.css
│   ├── js/
│   │   ├── site.js
│   │   └── activation.js
│   ├── images/
│   │   ├── brand/
│   │   ├── cat/
│   │   ├── puppy/
│   │   └── social/
│   ├── video/
│   │   ├── demo.mp4
│   │   ├── demo.webm
│   │   └── demo-poster.webp
│   └── downloads/
│       └── release files or redirects to release storage
├── scripts/
│   └── smoke-check.mjs
└── README.md
```

Keep shared visual rules in one stylesheet and shared interaction logic in one script. Do not introduce a templating/build system unless repeated manual page updates become an observed maintenance problem.

### 4.3 Activation/download boundary

Choose one of these modes in Phase 0:

**Mode A — app-enforced licensing (closest to the current product):**

- Installer files may remain public.
- Code lookup only identifies which product the customer bought.
- The desktop app performs the real activation/device-limit enforcement.
- The site must not imply that the download page itself protects the software.

**Mode B — protected binaries:**

- Submit activation code by `POST` to a same-origin API.
- Validate the code, product entitlement, status, and download allowance server-side.
- Return short-lived signed download URLs; never permanent storage URLs.
- Add rate limiting, generic invalid-code responses, audit logging with retention limits, and no JSONP.
- This mode requires a separately deployed backend/serverless function and private object storage.

Do not build Mode B unless protected downloads are a real requirement. App activation still remains necessary if files can be copied after download.

### 4.4 Canonical product contract

Define one small product data contract and use it to review every page and external form:

```text
Product ID: cat | puppy | custom
Display name
Current price and struck-through comparison price
Included behaviors
Supported platforms
Maximum activated devices
Installer version per platform
Installer URL, byte size, and SHA-256 per platform
Custom-pet species and delivery expectation, when applicable
```

Even in a static site, documenting this contract prevents the current marketing/form/legal drift.

## 5. Detailed implementation phases

### Phase 0 — Confirm scope and commercial contract

Tasks:

1. Approve the seven decisions in section 3.1.
2. Confirm whether the target is a faithful rebuild or an improved replacement using the same brand/copy.
3. Confirm ownership/permission for all pixel art, fonts, video, app icons, and installer binaries.
4. Confirm the order model: manual Google Form handoff or direct checkout.
5. Confirm activation Mode A or Mode B.
6. Confirm deployment target, domain/DNS ownership, and analytics property.
7. Have the privacy policy, terms, refund language, business/contact identity, and analytics consent approach reviewed by the appropriate owner or counsel.

Exit criteria:

- A one-page approved content/entitlement matrix exists.
- Form fields, marketing claims, legal claims, and app behavior agree.
- Website scope is explicitly separated from desktop-app scope.

### Phase 1 — Repository and delivery bootstrap

Tasks:

1. Initialize Git with a main branch and a minimal `.gitignore`.
2. Add the proposed static file structure.
3. Add a concise README with local preview and deployment commands.
4. Use a simple static server for local development; no application server is required.
5. Define deployment configuration for Hostinger or the chosen static host.
6. Add a smoke-check script using Node standard library to verify required files, internal links, unique titles, descriptions, canonical URLs, and referenced local assets.

Exit criteria:

- Site can be previewed locally from a clean checkout.
- A deploy produces the same route paths as production.
- Smoke check fails on missing routes/assets and passes on the baseline skeleton.

### Phase 2 — Shared design system and page shell

Tasks:

1. Move the observed palette, type, border, shadow, spacing, and width values into CSS custom properties.
2. Build shared page primitives through semantic class patterns: container, header, logo, button, card, eyebrow, section heading, notice, and footer.
3. Self-host the two fonts if licensing permits; otherwise keep Google Fonts with preconnects and strong fallbacks.
4. Add global normalization, responsive media rules, `min-width: 0` protection, media sizing, and text overflow handling.
5. Add a skip link and consistent `:focus-visible` outlines.
6. Ensure body content is visible without JavaScript.
7. Add optional reveal enhancement only after an `.enhanced` class is set.
8. Stop transitions and video autoplay when `prefers-reduced-motion: reduce` is active.

Exit criteria:

- Header/footer and core elements visually match the approved pixel-art direction.
- Keyboard focus is always visible.
- No component forces horizontal scrolling at 320px.
- Disabling CSS fonts or JavaScript does not make content unusable.

### Phase 3 — Landing page

Tasks:

1. Implement semantic landmarks: header/nav, main, sections, and footer.
2. Build the hero with the approved offer, headline, supporting copy, two CTAs, and three trust markers.
3. Replace the 75-second demo with a short optimized loop, MP4/WebM sources, poster image, explicit dimensions, and accessible adjacent description.
4. Build the six-card feature grid.
5. Build the cat/puppy comparison with real art, descriptive text, and product-specific CTAs.
6. Build pricing from the approved product contract.
7. Use native FAQ disclosure elements and preserve keyboard behavior.
8. Either implement the web cursor-pet demo as a progressive enhancement or remove its dead hint entirely.
9. Add section anchor offsets and verify navigation after font/media load.

Exit criteria:

- All landing content is available without JavaScript.
- CTA labels and destinations correspond to the selected product.
- Hero is two-column on wide screens and single-column without clipping on mobile.
- Feature, pet, and pricing grids collapse cleanly at their defined breakpoints.
- Autoplay/motion behavior respects user preferences.

### Phase 4 — Order handoff

Baseline tasks for the current manual sales model:

1. Update the Google Form to use `cat`, `puppy`, and `custom` product choices.
2. Remove the required platform field if both platforms are included.
3. Keep required name, phone, and email fields; keep Instagram optional only if there is a documented business need.
4. Do not collect a custom-pet photo until the order is accepted and a secure submission route is available.
5. Create product-specific prefilled Google Form URLs so “Get the Cat” and “Get the Puppy” retain intent.
6. Make clear that form submission is a reservation/contact request and not payment.
7. Track only CTA events, not form-entered personal data.

Exit criteria:

- Each CTA opens the form with the correct product selected.
- Form copy, required fields, privacy disclosure, and website pricing agree.
- No sensitive values are included in analytics URLs/events.

Direct payment, inventory, coupons, accounts, and order administration are deliberately out of scope until requested.

### Phase 5 — Activation and download pages

Tasks:

1. Build a labeled activation form with input normalization, pattern help, Enter-key submission, disabled/busy state, and an `aria-live` result region.
2. Submit codes using `POST`/JSON; do not use JSONP.
3. Handle empty, malformed, invalid, expired/revoked, wrong-product, network-error, timeout, and valid responses.
4. Reveal only entitled products.
5. Build separate cat and puppy instruction pages from the same verified content checklist and shared CSS.
6. Display version, supported OS versions, file size, SHA-256 checksum, and release date beside each installer.
7. Serve correct MIME types, `Content-Disposition`, range support, and stable download behavior.
8. Replace quarantine/SmartScreen bypass advice after signing and notarization are complete. Until then, phrase warnings accurately and obtain security/product approval.
9. Ensure activation codes are never sent to Google Analytics or embedded in the page URL.

Exit criteria:

- Every response state is deterministic and recoverable.
- A valid cat code cannot reveal puppy entitlement and vice versa.
- Public-versus-protected download behavior matches the approved Mode A/B decision.
- Mac and Windows files download with correct names and content types.
- Installer metadata matches the actual release artifacts.

### Phase 6 — Legal, analytics, SEO, and security hardening

Tasks:

1. Publish approved, internally consistent Privacy and Terms pages.
2. Include an appropriate support/contact path and clear last-updated dates.
3. Confirm whether analytics consent is required for the target markets; load analytics according to the approved policy.
4. Add unique titles/descriptions, canonical links, Open Graph/Twitter metadata, and a social image.
5. Add valid Product/SoftwareApplication JSON-LD for standard products; do not mark unavailable ratings/reviews.
6. Add `robots.txt`, `sitemap.xml`, and a branded `404.html`.
7. Extract inline styles/scripts and configure a practical Content Security Policy.
8. Add HSTS, `X-Content-Type-Options: nosniff`, a suitable `Referrer-Policy`, and a restrictive `Permissions-Policy` at the host/CDN layer.
9. Cache fingerprinted static assets for one year with `immutable`; keep HTML short-lived/revalidated.
10. Remove comments that expose operational details from production HTML where they add no user value.

Exit criteria:

- Search engines can discover all intended public routes.
- Social previews render the approved title/image.
- Security headers pass the agreed host-level check.
- Analytics behavior matches the published privacy policy.

### Phase 7 — Verification and launch

Functional matrix:

- All header, anchor, pricing, product, order, legal, login, and download links
- FAQ open/close by mouse and keyboard
- Product-specific form preselection
- Activation scenarios listed in Phase 5
- Installer download name, MIME, size, checksum, and range requests
- 404 behavior and canonical redirects (`http`→`https`, preferred host, `/index.html` policy)

Responsive matrix:

- Widths: 320, 360, 375, 390, 768, 1024, 1280, and 1440px
- Browsers: current Chrome, Safari, Firefox, and Edge
- Zoom/text: 200% zoom and increased text spacing
- Assertion: `document.documentElement.scrollWidth <= window.innerWidth` on every public route

Accessibility matrix:

- Keyboard-only navigation and activation submission
- Visible focus and logical focus order
- Landmarks/headings and descriptive links
- Image alternatives and decorative-image handling
- Form label, instructions, error association, and live status
- Reduced motion
- Automated scan plus manual checks against WCAG 2.2 AA

Performance matrix:

- Test landing, login, and a download page on mobile and desktop
- LCP ≤ 2.5s, INP ≤ 200ms, CLS ≤ 0.1 at the 75th percentile after field data exists
- No missing image dimensions or avoidable layout shifts
- No eager loading of below-the-fold product art
- Short, compressed hero media with a usable poster/fallback

Launch procedure:

1. Deploy to a password-protected staging URL.
2. Complete content, entitlement, legal, security, and installer sign-offs.
3. Run smoke, link, responsive, activation, accessibility, and performance checks.
4. Back up the existing production files.
5. Deploy static files and host/CDN configuration.
6. Verify production DNS/TLS, headers, analytics, form, activation, and downloads.
7. Submit the sitemap and monitor 404s, activation errors, download failures, and conversion events.
8. Keep a rollback package for the previous static release.

## 6. Acceptance criteria for the complete website

The website is complete when:

1. All approved routes are deployed and internally linked.
2. Product, platform, device, price, custom-pet, and refund claims are consistent across marketing, order form, activation response, download instructions, Privacy, and Terms.
3. The page has no clipped or horizontally scrolling content at 320px or wider.
4. Core content remains visible and usable without JavaScript.
5. Every interactive element is keyboard-accessible with visible focus.
6. Reduced-motion users do not receive forced reveal motion or autoplay video.
7. Cat, puppy, and custom CTAs preserve product intent through the order flow.
8. Activation behavior matches the explicitly selected security mode.
9. Installers are correctly served and their displayed metadata/checksums are verified.
10. `robots.txt`, sitemap, canonical metadata, social metadata, legal pages, 404 handling, analytics, and security headers are present and tested.
11. Smoke checks and the Phase 7 verification matrix pass on staging and production.

## 7. Explicitly deferred work

Unless separately requested, do not add:

- React/Next.js/Astro or a component library
- A CMS
- Customer accounts
- Direct payment/checkout
- An admin dashboard
- Email/SMS automation
- A database for website content
- A custom analytics platform
- Desktop pet application code
- A protected-download backend when app-enforced licensing is accepted

These should be introduced only when a confirmed requirement exceeds the static-site baseline.
