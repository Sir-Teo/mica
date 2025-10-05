# 🎮 Mica Playground - Setup & Deployment Guide

This guide covers everything needed to set up, test, and deploy the Mica playground on GitHub Pages.

## 📋 What Has Been Created

### 1. **Playground HTML Page** (`docs/playground.html`)
   - Interactive code editor powered by CodeMirror
   - Example selector with all 21+ examples
   - Compiler stage buttons (Tokens, AST, Resolve, IR, Run)
   - Responsive design matching the main site's futuristic theme
   - Real-time code editing and syntax highlighting

### 2. **WebAssembly Support** 
   - **`src/wasm.rs`** - WASM bindings for the compiler
   - **`Cargo.toml`** - Updated with WASM dependencies and build config
   - **`build-wasm.sh`** - Script to build WASM module
   - Exposes compiler functions: tokenize, parse, resolve, check, lower, generate_ir

### 3. **Examples Infrastructure**
   - **`scripts/generate-examples-manifest.js`** - Node.js script to scan examples
   - **`scripts/generate-examples-manifest.sh`** - Shell script alternative
   - **`docs/examples-manifest.json`** - Generated JSON with all examples (auto-created)
   - Automatically includes code, descriptions, line counts, file sizes

### 4. **GitHub Actions Workflow** (`.github/workflows/pages.yml`)
   - Automatically builds WASM module
   - Generates examples manifest
   - Deploys to GitHub Pages on every push to main
   - Configured for optimal build performance

### 5. **Documentation**
   - **`docs/PLAYGROUND.md`** - User guide and architecture documentation
   - **`PLAYGROUND_SETUP.md`** (this file) - Setup and deployment guide
   - Updated **`README.md`** with playground links and quickstart

### 6. **Site Integration**
   - Updated `docs/index.html` with playground links in:
     - Hero CTA buttons
     - Quick links section
     - Footer navigation
   - Prominent "Try Playground" button on homepage

## 🚀 Quick Start

### For Local Development

1. **Generate the examples manifest** (required):
   ```bash
   node scripts/generate-examples-manifest.js
   ```

2. **Start a local server**:
   ```bash
   cd docs
   python3 -m http.server 8000
   # Or: npx serve .
   ```

3. **Open the playground**:
   ```
   http://localhost:8000/playground.html
   ```

### For WASM Compilation (Optional)

1. **Install wasm-pack**:
   ```bash
   cargo install wasm-pack
   ```

2. **Build the WASM module**:
   ```bash
   bash build-wasm.sh
   ```

3. **Verify the build**:
   ```bash
   ls -lh docs/wasm/
   # Should show mica.js, mica_bg.wasm, etc.
   ```

## 📦 Deployment to GitHub Pages

### Automatic Deployment (Recommended)

The GitHub Actions workflow automatically deploys on every push to `main`:

1. **Push your changes**:
   ```bash
   git add .
   git commit -m "Add Mica playground"
   git push origin main
   ```

2. **Monitor the deployment**:
   - Go to your repository on GitHub
   - Click **Actions** tab
   - Watch the "Deploy GitHub Pages" workflow

3. **Access the playground**:
   ```
   https://sir-teo.github.io/mica/playground.html
   ```

### Manual Deployment

If you need to deploy manually:

1. **Enable GitHub Pages** in repository settings:
   - Settings → Pages
   - Source: GitHub Actions

2. **Run the workflow manually**:
   - Actions → Deploy GitHub Pages → Run workflow

## 🧪 Testing Checklist

Before deploying, verify these work locally:

- [ ] Examples manifest loads correctly
- [ ] All 21+ examples appear in dropdown
- [ ] Selecting an example loads the code
- [ ] Code editor is functional and editable
- [ ] Clear button works
- [ ] All compiler stage buttons are visible
- [ ] Output panel displays messages
- [ ] Page is responsive on mobile
- [ ] Dark/light theme toggle works
- [ ] Links to other pages work

### Test Commands

```bash
# 1. Generate manifest
node scripts/generate-examples-manifest.js

# 2. Verify manifest
cat docs/examples-manifest.json | jq '.[0]'

# 3. Count examples
cat docs/examples-manifest.json | jq 'length'

# 4. Start local server
cd docs && python3 -m http.server 8000

# 5. Test in browser
# Open: http://localhost:8000/playground.html
```

## 🔧 Configuration

### Adding New Examples

1. Add `.mica` file to `examples/` directory
2. Regenerate manifest:
   ```bash
   node scripts/generate-examples-manifest.js
   ```
3. Commit and push (auto-deploys)

### Customizing the Playground

**Editor Settings** (`docs/playground.html`):
```javascript
const editor = CodeMirror(document.getElementById('editor'), {
    lineNumbers: true,
    mode: 'rust',      // Change syntax highlighting mode
    theme: 'monokai',  // Change color theme
    tabSize: 2,        // Adjust tab size
    // ... more options
});
```

**Styling** (`docs/assets/futuristic.css`):
- Shared styles for all pages
- CSS custom properties for theming
- Dark/light theme variables

### WASM Build Options

**Cargo.toml** - Optimization settings:
```toml
[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Link-time optimization
codegen-units = 1   # Single codegen unit
```

**Build command** - Additional flags:
```bash
wasm-pack build \
  --target web \
  --out-dir docs/wasm \
  --release \
  -- --features wasm-only  # Add custom features
```

## 🐛 Troubleshooting

### Examples Not Loading

**Problem**: Dropdown is empty or examples don't load

**Solutions**:
1. Verify manifest exists:
   ```bash
   ls -la docs/examples-manifest.json
   ```
2. Check manifest is valid JSON:
   ```bash
   cat docs/examples-manifest.json | jq .
   ```
3. Regenerate if needed:
   ```bash
   node scripts/generate-examples-manifest.js
   ```
4. Check browser console for errors

### WASM Build Fails

**Problem**: `wasm-pack build` errors

**Solutions**:
1. Update wasm-pack:
   ```bash
   cargo install wasm-pack --force
   ```
2. Update Rust toolchain:
   ```bash
   rustup update
   ```
3. Add wasm target:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```
4. Check dependencies in Cargo.toml

### GitHub Pages Not Updating

**Problem**: Changes don't appear on live site

**Solutions**:
1. Check Actions tab for deployment status
2. Clear browser cache (Cmd+Shift+R)
3. Verify GitHub Pages is enabled:
   - Settings → Pages → Source: GitHub Actions
4. Check workflow logs for errors
5. Wait 2-3 minutes for CDN propagation

### CodeMirror Not Loading

**Problem**: Editor doesn't appear

**Solutions**:
1. Check internet connection (CDN dependency)
2. Verify CDN URLs in HTML:
   ```html
   <script src="https://cdnjs.cloudflare.com/ajax/libs/codemirror/5.65.2/codemirror.min.js"></script>
   ```
3. Try alternative CDN or local hosting
4. Check browser console for 404 errors

## 📊 Monitoring

### GitHub Actions

Monitor deployments:
```bash
# View workflow runs
gh run list --workflow=pages.yml

# View specific run
gh run view <run-id>

# Watch live logs
gh run watch
```

### Analytics

Track playground usage (optional):
1. Add Google Analytics to `playground.html`
2. Track button clicks with custom events
3. Monitor example selection frequency

## 🔮 Future Enhancements

Ready to implement:
- [ ] Full WASM compilation in browser
- [ ] Syntax highlighting for Mica language (custom CodeMirror mode)
- [ ] Share code via URL parameters
- [ ] Save/load custom snippets to localStorage
- [ ] Step-by-step debugger
- [ ] LSP integration for autocomplete
- [ ] Export to GitHub Gist
- [ ] Embedded tutorials

## 📝 Maintenance

### Regular Tasks

**Weekly**:
- Check GitHub Actions for failures
- Test playground on latest browsers
- Monitor example manifest updates

**When adding examples**:
```bash
node scripts/generate-examples-manifest.js
git add docs/examples-manifest.json
git commit -m "Update examples manifest"
git push
```

**When updating dependencies**:
```bash
cargo update
wasm-pack build --target web --out-dir docs/wasm --release
```

## 🤝 Contributing

To contribute to the playground:

1. **Fork the repository**
2. **Make your changes** (follow code style)
3. **Test locally** (all checklist items)
4. **Update manifest** if examples changed
5. **Submit pull request** with description

## 📚 Additional Resources

- [CodeMirror Documentation](https://codemirror.net/doc/manual.html)
- [wasm-bindgen Guide](https://rustwasm.github.io/wasm-bindgen/)
- [GitHub Pages Docs](https://docs.github.com/en/pages)
- [wasm-pack Book](https://rustwasm.github.io/wasm-pack/)

## 🎯 Success Metrics

The playground is working correctly when:
- ✅ All 21+ examples load without errors
- ✅ Code editor is responsive and smooth
- ✅ Page loads in < 3 seconds
- ✅ Works on Chrome, Firefox, Safari, Edge
- ✅ Responsive on mobile devices
- ✅ No console errors in browser
- ✅ GitHub Actions deploys successfully
- ✅ WASM module (when built) is < 2MB

## 📧 Support

For issues or questions:
- Open an issue on GitHub
- Check existing issues for solutions
- Review this guide and PLAYGROUND.md
- Test locally before reporting bugs

---

**Status**: ✅ Ready for deployment
**Last Updated**: 2025-10-05
**Version**: 1.0.0
