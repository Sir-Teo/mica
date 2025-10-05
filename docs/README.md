# Mica Documentation

This directory contains the full documentation for the Mica programming language, optimized for GitHub Pages deployment.

## 📚 Documentation Structure

### Core Documentation

#### HTML Pages (Main Site)
- **[index.html](index.html)** — Main landing page with dark mode, animations
- **[getting-started.html](getting-started.html)** — Installation and quickstart guide
- **[tour.html](tour.html)** — Interactive language tour (also available as .md)
- **[examples.html](examples.html)** — Gallery of 20+ runnable examples
- **[features.html](features.html)** — Comprehensive feature overview with comparison table
- **[architecture.html](architecture.html)** — Compiler pipeline and design
- **[faq.html](faq.html)** ⭐ NEW — Interactive FAQ with search functionality
- **[contributing.html](contributing.html)** ⭐ NEW — Complete contribution guide
- **[snippets.html](snippets.html)** — CLI output examples (also available as .md)

#### Markdown Documentation
- **[tour.md](tour.md)** — Language tour with examples and syntax guide
- **[status.md](status.md)** — Current project status and health report
- **[status_summary.md](status_summary.md)** — Condensed Phase 3 status
- **[snippets.md](snippets.md)** — CLI output examples (auto-generated)
- **[module_reference.md](module_reference.md)** — Index of compiler subsystems

### Module Documentation (`modules/`)
Deep dives into each compiler subsystem:
- CLI and Developer Tooling
- Compiler Pipeline Entry Points
- Diagnostics Infrastructure
- Documentation and Examples
- Lowering Pipeline
- Runtime and Capability Providers
- Semantic Analysis
- SSA Intermediate Representation
- Syntax Front-End
- Testing Harness

### Roadmap (`roadmap/`)
Development plans and milestones:
- **[index.md](roadmap/index.md)** — Roadmap overview
- **[milestones.md](roadmap/milestones.md)** — Phase-by-phase execution plan
- **[compiler.md](roadmap/compiler.md)** — Compiler module roadmap
- **[tooling.md](roadmap/tooling.md)** — CLI, formatter, and IDE plans
- **[ecosystem.md](roadmap/ecosystem.md)** — Standard library and package manager
- **[next-step.md](roadmap/next-step.md)** — Immediate action items

## 🚀 GitHub Pages Setup

### Prerequisites
1. Repository must be public or have GitHub Pages enabled for private repos
2. GitHub Pages must be enabled in repository settings

### Deployment Steps

#### Option 1: Using GitHub Actions (Recommended)
Create `.github/workflows/pages.yml`:

```yaml
name: Deploy GitHub Pages

on:
  push:
    branches: ["main"]
  workflow_dispatch:

permissions:
  contents: read
  pages: write
  id-token: write

jobs:
  deploy:
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      
      - name: Setup Pages
        uses: actions/configure-pages@v4
      
      - name: Upload artifact
        uses: actions/upload-pages-artifact@v3
        with:
          path: 'docs'
      
      - name: Deploy to GitHub Pages
        id: deployment
        uses: actions/deploy-pages@v4
```

#### Option 2: Manual Deployment
1. Go to repository **Settings** → **Pages**
2. Under "Source", select **Deploy from a branch**
3. Choose branch: `main` (or your default branch)
4. Choose folder: `/docs`
5. Click **Save**

Your site will be available at: `https://sir-teo.github.io/mica/`

## 🎨 Features

### Complete Site Structure
- **9 Main HTML Pages** — Comprehensive coverage of all aspects
  - Landing page with hero section, dark mode, animations
  - Getting Started guide
  - Language Tour
  - Examples Gallery (20+ programs)
  - Features Overview with comparison table
  - Architecture Deep Dive
  - Interactive FAQ with search
  - Contributing Guide
  - CLI Reference

### Modern Design & Interactivity
- **Dark mode toggle** — Persistent theme with localStorage
- **Smooth scroll animations** — Fade-in on scroll with Intersection Observer
- **Scroll-to-top button** — Appears after scrolling
- **Responsive layout** — Works on all screen sizes
- **Beautiful gradient headers** — Eye-catching design
- **Feature cards** — Highlight key capabilities with hover effects
- **Code syntax highlighting** — Styled Mica code examples
- **Interactive FAQ** — Accordion with search functionality
- **Professional typography** — Easy to read
- **Consistent branding** — Unified color scheme
- **Social media meta tags** — Open Graph and Twitter cards
- **Accessibility** — ARIA labels, semantic HTML

### Comprehensive Navigation
- Every page has breadcrumb navigation back to home
- Related documentation links at the bottom of each page
- Consistent cross-linking between related topics
- Module documentation cross-references other modules

### Jekyll Support
- `_config.yml` configured for GitHub Pages
- Supports both HTML and Markdown files
- Automatic conversion of `.md` to `.html` URLs
- SEO tags and sitemap generation

## 📝 Content Updates

### Auto-Generated Content
The CLI snippets are automatically generated:
```bash
cargo run --bin gen_snippets              # Regenerate
cargo run --bin gen_snippets -- --check   # Verify
```

### Manual Updates
When updating documentation:
1. Fix dates to current year (2025)
2. Use `func` keyword consistently (not `fn`)
3. Add navigation links to new pages
4. Cross-link related documentation
5. Test locally before pushing

### Local Testing
To test locally with Jekyll:
```bash
cd docs
bundle install
bundle exec jekyll serve
```

Then visit `http://localhost:4000/mica/`

## 🔗 URL Structure

All URLs are relative and work both locally and on GitHub Pages:
- Landing: `/` or `/index.html`
- Tour: `/tour.html`
- Status: `/status.html`
- Modules: `/modules/[name].html`
- Roadmap: `/roadmap/[name].html`

## ✅ Quality Checklist

Before deploying updates:
- [x] All dates are current (2025)
- [x] Consistent keyword usage (`func` not `fn`)
- [x] Navigation links work
- [x] No broken internal links
- [x] CLI snippets are regenerated if needed
- [x] All new pages have breadcrumbs
- [x] Related docs section added to new pages
- [x] Cross-references updated
- [x] Sitemap.xml created
- [x] All 9 main pages completed
- [x] Dark mode implemented
- [x] Scroll animations added
- [x] Interactive FAQ with search
- [x] Contributing guide created
- [x] Social media meta tags
- [x] 10 module pages enhanced
- [x] Roadmap pages cross-linked
- [x] Mobile-responsive design

## 🎯 Maintenance

### Regular Updates
- Update status.md monthly or after major milestones
- Regenerate snippets.md when CLI changes
- Review roadmap quarterly
- Update dates in status files

### Adding New Pages
1. Create the file in appropriate directory
2. Add navigation header with back links
3. Add related documentation footer
4. Update parent index/reference file
5. Add to `_config.yml` if needed

## 🎯 Page Descriptions

### Main Pages

**index.html** — Beautiful landing page with hero section, feature highlights, quick links, module cards, and project stats. Fully responsive with modern gradient design.

**getting-started.html** — Step-by-step installation guide covering prerequisites, clone/build, verification, first program, and development workflow. Includes CLI examples and next steps.

**tour.html** (and tour.md) — Comprehensive language tour covering modules, ADTs, pattern matching, effects, concurrency, generics, and more. Links to runnable examples.

**examples.html** — Visual gallery of 20+ examples with descriptions, difficulty badges, feature lists, and direct GitHub links. Organized by complexity.

**features.html** — Deep dive into language features including effect system, deterministic concurrency, ADTs, generics, memory safety, and design philosophy. Includes comparison table.

**architecture.html** — Complete compiler pipeline overview with interactive stage-by-stage breakdown, source organization, development phases, and design principles.

**faq.html** ⭐ NEW — Interactive FAQ page with 20+ questions organized by category. Features accordion UI, live search, and smooth animations.

**contributing.html** ⭐ NEW — Comprehensive guide for contributors with step-by-step instructions, commit guidelines, code review process, and community standards.

**snippets.html** (and snippets.md) — Real compiler output for all CLI flags. Auto-generated and CI-verified to stay in sync with codebase.

## 📊 Analytics (Optional)

To add Google Analytics, insert in `index.html` before `</head>`:
```html
<!-- Google Analytics -->
<script async src="https://www.googletagmanager.com/gtag/js?id=GA_MEASUREMENT_ID"></script>
<script>
  window.dataLayer = window.dataLayer || [];
  function gtag(){dataLayer.push(arguments);}
  gtag('js', new Date());
  gtag('config', 'GA_MEASUREMENT_ID');
</script>
```

## 🐛 Troubleshooting

### Pages not updating
- Check GitHub Actions tab for deployment status
- Clear browser cache
- Wait 5-10 minutes for CDN propagation

### 404 errors
- Ensure branch and folder are correct in settings
- Check file extensions (`.md` becomes `.html`)
- Verify relative paths are correct

### CSS not loading
- Check `baseurl` in `_config.yml`
- Ensure paths don't have leading `/` for relative links

## 📞 Support

For issues with the documentation:
1. Check existing [GitHub Issues](https://github.com/Sir-Teo/mica/issues)
2. Review this README for deployment steps
3. Open a new issue with details

---

**Last Updated:** 2025-10-04  
**Maintained by:** Mica Project Contributors
