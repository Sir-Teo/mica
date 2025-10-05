# 🚀 Playground Quick Start

Get the Mica playground running in **under 5 minutes**!

## Option 1: Deploy to GitHub Pages (Recommended)

```bash
# 1. Generate examples manifest
node scripts/generate-examples-manifest.js

# 2. Commit and push
git add .
git commit -m "Add Mica playground with all examples"
git push origin main

# 3. Enable GitHub Pages (if not already enabled)
# Go to: Settings → Pages → Source: GitHub Actions

# 4. Wait 2-3 minutes and visit:
# https://YOUR-USERNAME.github.io/mica/playground.html
```

That's it! GitHub Actions will automatically build and deploy everything.

---

## Option 2: Test Locally First

```bash
# 1. Generate examples manifest
node scripts/generate-examples-manifest.js

# 2. Start local server
cd docs
python3 -m http.server 8000

# 3. Open in browser
open http://localhost:8000/playground.html
```

---

## What You Get

✅ **Interactive code editor** with syntax highlighting  
✅ **21+ examples** from your repository  
✅ **6 compiler stages** to explore  
✅ **Beautiful UI** matching your site theme  
✅ **Fully responsive** works on mobile  
✅ **Auto-deployed** via GitHub Actions  

---

## File Overview

```
✨ NEW FILES CREATED:
├── docs/
│   ├── playground.html              ← Main playground page
│   ├── examples-manifest.json       ← All examples (auto-generated)
│   ├── PLAYGROUND.md                ← User documentation
│   └── wasm/ (optional)             ← WebAssembly module
├── scripts/
│   ├── generate-examples-manifest.js ← Generate manifest
│   └── generate-examples-manifest.sh ← Shell alternative
├── src/
│   └── wasm.rs                       ← WebAssembly bindings
├── .github/workflows/
│   └── pages.yml                     ← Auto-deployment
├── build-wasm.sh                     ← WASM build script
├── PLAYGROUND_SETUP.md               ← Detailed setup guide
├── PLAYGROUND_SUMMARY.md             ← Implementation summary
└── PLAYGROUND_QUICKSTART.md          ← This file!

📝 MODIFIED FILES:
├── Cargo.toml        ← Added WASM dependencies
├── src/lib.rs        ← Added WASM module
├── docs/index.html   ← Added playground links
└── README.md         ← Added playground section
```

---

## Testing Checklist

After deployment, verify:

- [ ] Visit the playground URL
- [ ] Select an example from dropdown
- [ ] Code appears in editor
- [ ] Edit the code
- [ ] Click "Tokens" button
- [ ] Output appears in right panel
- [ ] Try other compiler stages
- [ ] Test on mobile device
- [ ] Check theme toggle works

---

## Troubleshooting

**Examples not loading?**
```bash
node scripts/generate-examples-manifest.js
```

**GitHub Pages not working?**
- Check Actions tab for errors
- Enable GitHub Pages: Settings → Pages
- Wait 2-3 minutes after first push

**Need help?**
- See `PLAYGROUND_SETUP.md` for detailed troubleshooting
- Check GitHub Actions logs
- Test locally first

---

## Next Steps

1. **Test it** - Verify everything works
2. **Share it** - Tell others about the playground
3. **Enhance it** - Add WASM for real-time compilation:
   ```bash
   bash build-wasm.sh
   git add docs/wasm/
   git commit -m "Add WASM module"
   git push
   ```

---

## 🎯 Quick Commands

```bash
# Regenerate examples when you add new ones
node scripts/generate-examples-manifest.js

# Build WASM (optional, for browser compilation)
bash build-wasm.sh

# Test locally
cd docs && python3 -m http.server 8000

# Deploy (just push to main)
git push origin main
```

---

**Ready? Run the commands above and your playground will be live!** 🎉

For detailed documentation, see:
- `PLAYGROUND_SUMMARY.md` - What was built
- `PLAYGROUND_SETUP.md` - Complete setup guide
- `docs/PLAYGROUND.md` - User documentation
