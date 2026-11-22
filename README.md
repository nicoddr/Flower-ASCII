# ASCII Garden — Tulips, Freesias & Peonies (Rust / WebAssembly)

A 100% static client-side website ready for **GitHub Pages** (no backend server required): a real-time animated pastel flower bouquet of **tulips, freesias, and peonies** rendered in colorized ASCII art on a starry black terminal canvas, powered by a zero-dependency **Rust / WebAssembly** 3D engine.

## Highlights

- **Pastel Garden Bouquet**: Two multi-layered ruffled peonies (blush rose & pearl blush), two sculptural tulips (pastel lilac & peach), and two arching freesia combs (butter yellow & pearl white) framed by sage-emerald foliage.
- **Regular-Grid ASCII Shading**: Uses `@`, `#`, `%`, `*`, `+`, `=`, `:`, `;` ordered by density with a 2×2 supersampled Z-buffer, petal-overlap crease shadows, luminous white/light-gray highlights, and twinkling white starfield particles on `#000000` black.
- **Interactive Motion**: Opening bloom animation, multi-harmonic breeze, subtle cursor wind interaction, pause/resume (`Space`), optional CRT phosphor glow, and one-click ASCII copy.

## Repository Structure

```text
.
├── .github/workflows/pages.yml  # CI/CD: tests & compiles Rust -> WASM and deploys to GitHub Pages
├── .nojekyll                    # Serves static files directly on GitHub Pages
├── Cargo.toml                   # Zero-dependency Rust crate targeting wasm32-unknown-unknown
├── src/lib.rs                   # 3D ASCII bouquet engine (peonies, tulips, freesias, wind, stars)
├── index.html                   # Static entry point (relative paths)
├── style.css                    # Minimalist pure-black terminal styling
├── app.js                       # Regular-grid Canvas renderer & controls
├── bouquet.wasm                 # Prebuilt Rust/WebAssembly binary
├── wasm_fallback.js             # Embedded WASM fallback for direct file:// opening
└── build.ps1                    # Local WASM rebuild script
```

## Deploy to GitHub Pages

1. Push this folder to a GitHub repository (`main` branch).
2. Open **Settings → Pages** and choose either:
   - **GitHub Actions** *(recommended)*: `.github/workflows/pages.yml` automatically tests, compiles `bouquet.wasm`, and deploys on every push.
   - **Deploy from a branch** → `main` / `/ (root)`: works immediately using the prebuilt `bouquet.wasm` at the repository root.

## Local Rebuild

```powershell
powershell -ExecutionPolicy Bypass -File .\build.ps1
cargo test
```

*(You can also open `index.html` directly in your browser without any local server thanks to `wasm_fallback.js`.)*
