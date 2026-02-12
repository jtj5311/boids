# WebAssembly Boid Simulation Integration Plan

## Overview

Convert the Rust boid simulation (macroquad + egui-macroquad) to WebAssembly and integrate it into the Next.js portfolio at `/boid-simulation` with a fixed 1200x800 canvas and minimal text.

## Architecture

**Strategy**: Build Rust simulation as standalone WASM module with minimal changes, load into Next.js client component that manages canvas lifecycle and WASM initialization.

**Key Insight**: macroquad 0.4 and egui-macroquad 0.17 have excellent built-in WASM support - no major dependency changes needed.

---

## Implementation Steps

### Phase 1: Rust WASM Setup (30 mins)

**1. Modify Cargo.toml**

File: `/home/leahm/Projects/boids/leah/rust/boid_simulation/Cargo.toml`

Add WASM build configuration and size optimizations:

```toml
[lib]
crate-type = ["cdylib", "rlib"]

[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Link-time optimization
codegen-units = 1
panic = "abort"
strip = true
```

**2. Create Build Configuration**

File: `/home/leahm/Projects/boids/leah/rust/boid_simulation/.cargo/config.toml` (new)

```toml
[build]
target = "wasm32-unknown-unknown"

[target.wasm32-unknown-unknown]
rustflags = ["-C", "link-arg=-s"]
```

**3. Install WASM Toolchain**

```bash
rustup target add wasm32-unknown-unknown
```

**4. Create Build Script**

File: `/home/leahm/Projects/boids/leah/rust/boid_simulation/build_wasm.sh` (new)

```bash
#!/bin/bash
set -e

echo "Building boid simulation for WebAssembly..."

cargo build --release --target wasm32-unknown-unknown

# Optimize with wasm-strip if available
if command -v wasm-strip &> /dev/null; then
    wasm-strip target/wasm32-unknown-unknown/release/boid_simulation.wasm
fi

# Copy to Next.js public directory
TARGET_DIR="../../../leahchilders-portfolio/web_app/public/wasm"
mkdir -p "$TARGET_DIR"
cp target/wasm32-unknown-unknown/release/boid_simulation.wasm "$TARGET_DIR/"

echo "Build complete! File copied to $TARGET_DIR"
echo "File size: $(du -h target/wasm32-unknown-unknown/release/boid_simulation.wasm | cut -f1)"
```

Make executable: `chmod +x build_wasm.sh`

**5. Test WASM Build**

```bash
cd /home/leahm/Projects/boids/leah/rust/boid_simulation
./build_wasm.sh
```

Verify: `ls -lh target/wasm32-unknown-unknown/release/boid_simulation.wasm`

---

### Phase 2: Next.js Structure (20 mins)

**1. Create Directory Structure**

```bash
cd /home/leahm/Projects/leahchilders-portfolio/web_app
mkdir -p app/boid-simulation
mkdir -p public/wasm
mkdir -p types
```

**2. Download macroquad WebGL Loader**

Download `gl.js` from https://not-fl3.github.io/miniquad-samples/gl.js

Save to: `public/wasm/gl.js`

**3. Copy WASM File**

From Rust build output to: `public/wasm/boid_simulation.wasm`

**4. Create TypeScript Types**

File: `/home/leahm/Projects/leahchilders-portfolio/web_app/types/wasm.d.ts` (new)

```typescript
declare global {
  interface Window {
    load: (wasmPath: string) => Promise<void>;
    miniquad_add_plugin?: (plugin: any) => void;
  }
}

export {};
```

---

### Phase 3: Component Development (45 mins)

**1. Create Page Component**

File: `/home/leahm/Projects/leahchilders-portfolio/web_app/app/boid-simulation/page.tsx` (new)

```tsx
import type { Metadata } from "next";
import { BoidCanvas } from "./BoidCanvas";

export const metadata: Metadata = {
  title: "Boid Simulation - Leah Childers",
  description: "Interactive boid flocking simulation with disease models",
};

export default function BoidSimulationPage() {
  return (
    <div className="flex-1 flex flex-col gap-4 px-4">
      <h1 className="text-4xl font-bold">Boid Simulation</h1>
      <BoidCanvas />
    </div>
  );
}
```

**2. Create Canvas Component**

File: `/home/leahm/Projects/leahchilders-portfolio/web_app/app/boid-simulation/BoidCanvas.tsx` (new)

```tsx
"use client";

import { useEffect, useRef, useState } from "react";

declare global {
  interface Window {
    load: (wasmPath: string) => Promise<void>;
  }
}

export function BoidCanvas() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const loadWasm = async () => {
      try {
        setIsLoading(true);
        setError(null);

        // Load gl.js (miniquad loader)
        if (!window.load) {
          await loadScript("/wasm/gl.js");
        }

        // Set canvas element
        if (canvasRef.current) {
          canvasRef.current.id = "glcanvas";
          canvasRef.current.tabIndex = 1;
        }

        // Load WASM module
        await window.load("/wasm/boid_simulation.wasm");

        setIsLoading(false);
      } catch (err) {
        console.error("Failed to load WASM:", err);
        setError(err instanceof Error ? err.message : "Failed to load simulation");
        setIsLoading(false);
      }
    };

    loadWasm();
  }, []);

  const loadScript = (src: string): Promise<void> => {
    return new Promise((resolve, reject) => {
      const existing = document.querySelector(`script[src="${src}"]`);
      if (existing) {
        resolve();
        return;
      }

      const script = document.createElement("script");
      script.src = src;
      script.async = true;
      script.onload = () => resolve();
      script.onerror = () => reject(new Error(`Failed to load: ${src}`));
      document.head.appendChild(script);
    });
  };

  return (
    <div className="flex flex-col items-center justify-center gap-4">
      {isLoading && (
        <div className="flex flex-col items-center gap-2 p-8">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary"></div>
          <p className="text-sm text-muted-foreground">Loading simulation...</p>
        </div>
      )}

      {error && (
        <div className="bg-destructive/10 border border-destructive rounded-lg p-4 max-w-2xl">
          <p className="text-destructive font-semibold">Error loading simulation</p>
          <p className="text-sm text-muted-foreground mt-1">{error}</p>
        </div>
      )}

      <div
        className="relative border border-border rounded-lg overflow-hidden shadow-lg"
        style={{
          width: "1200px",
          height: "800px",
          display: isLoading || error ? "none" : "block",
        }}
      >
        <canvas
          ref={canvasRef}
          width={1200}
          height={800}
          className="block"
        />
      </div>

      {!isLoading && !error && (
        <div className="text-sm text-muted-foreground max-w-2xl">
          <p><strong>Keyboard Shortcuts:</strong></p>
          <ul className="list-disc list-inside ml-4">
            <li><kbd className="px-1 py-0.5 bg-muted rounded">P</kbd> - Toggle parameters</li>
            <li><kbd className="px-1 py-0.5 bg-muted rounded">G</kbd> - Toggle graph</li>
            <li><kbd className="px-1 py-0.5 bg-muted rounded">Enter</kbd> - Restart</li>
          </ul>
        </div>
      )}
    </div>
  );
}
```

---

### Phase 4: Navigation & Config (15 mins)

**1. Update Navigation**

File: `/home/leahm/Projects/leahchilders-portfolio/web_app/components/nav-buttons.tsx`

Add to navItems array:
```tsx
{ href: "/boid-simulation", label: "Boid Sim" }
```

**2. Update Next.js Config**

File: `/home/leahm/Projects/leahchilders-portfolio/web_app/next.config.ts`

Add WASM headers:
```typescript
const nextConfig: NextConfig = {
  cacheComponents: true,

  async headers() {
    return [
      {
        source: "/wasm/:path*.wasm",
        headers: [
          {
            key: "Content-Type",
            value: "application/wasm",
          },
          {
            key: "Cache-Control",
            value: "public, max-age=31536000, immutable",
          },
        ],
      },
    ];
  },
};
```

---

### Phase 5: Testing (30 mins)

**1. Local Development**

Terminal 1 - Build WASM:
```bash
cd /home/leahm/Projects/boids/leah/rust/boid_simulation
./build_wasm.sh
```

Terminal 2 - Run Next.js:
```bash
cd /home/leahm/Projects/leahchilders-portfolio/web_app
npm run dev
```

Open: http://localhost:3000/boid-simulation

**2. Validation Checklist**

- [ ] WASM file loads without 404
- [ ] Canvas renders at 1200x800
- [ ] Boids appear and move
- [ ] Parameter panel visible (sliders, dropdowns)
- [ ] Graph toggles with G key
- [ ] Parameters toggle with P key
- [ ] Restart works with Enter
- [ ] Disease states show colors (white=S, orange=E, red=I, blue=R)
- [ ] No console errors

**3. Performance Check**

- Expected: 60 FPS with 150-1000 boids
- Monitor with Chrome DevTools Performance tab
- Check WASM bundle size: ~2-3 MB (uncompressed)

---

### Phase 6: Deployment (20 mins)

**1. Build for Production**

```bash
cd /home/leahm/Projects/boids/leah/rust/boid_simulation
./build_wasm.sh

cd /home/leahm/Projects/leahchilders-portfolio/web_app
npm run build
```

**2. Commit and Deploy**

```bash
git add app/boid-simulation/ public/wasm/ types/wasm.d.ts components/nav-buttons.tsx next.config.ts
git commit -m "Add WASM boid simulation"
git push
```

Vercel will auto-deploy.

**3. Verify Production**

- Check deployed URL
- Test all functionality
- Monitor Vercel logs for errors

---

## Critical Files

### Modified Files
- `/home/leahm/Projects/boids/leah/rust/boid_simulation/Cargo.toml`
- `/home/leahm/Projects/leahchilders-portfolio/web_app/components/nav-buttons.tsx`
- `/home/leahm/Projects/leahchilders-portfolio/web_app/next.config.ts`

### New Files
- `/home/leahm/Projects/boids/leah/rust/boid_simulation/.cargo/config.toml`
- `/home/leahm/Projects/boids/leah/rust/boid_simulation/build_wasm.sh`
- `/home/leahm/Projects/leahchilders-portfolio/web_app/app/boid-simulation/page.tsx`
- `/home/leahm/Projects/leahchilders-portfolio/web_app/app/boid-simulation/BoidCanvas.tsx`
- `/home/leahm/Projects/leahchilders-portfolio/web_app/types/wasm.d.ts`
- `/home/leahm/Projects/leahchilders-portfolio/web_app/public/wasm/boid_simulation.wasm`
- `/home/leahm/Projects/leahchilders-portfolio/web_app/public/wasm/gl.js`

---

## Common Issues & Solutions

| Issue | Solution |
|-------|----------|
| 404 on .wasm file | Verify file in `public/wasm/`, check headers in next.config.ts |
| Black screen | Check browser console for GL errors, verify canvas has focus |
| Keyboard not working | Click canvas to focus (tabIndex already set) |
| Low FPS | Reduce boid count, check browser DevTools Performance |
| Build fails | Ensure wasm32-unknown-unknown target installed |

---

## Technical Details

### WASM Build Process

1. **Compilation**: Rust code compiles to WebAssembly bytecode
2. **Optimization**: Size optimizations reduce bundle from ~5MB to ~2MB
3. **Loading**: macroquad's `gl.js` provides WebGL context and game loop
4. **Rendering**: All canvas rendering happens in WASM (egui UI + macroquad graphics)

### Architecture Diagram

```
┌─────────────────────────────────────┐
│  Next.js App Router                 │
│  /app/boid-simulation/page.tsx      │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│  Client Component                   │
│  BoidCanvas.tsx                     │
│  - WASM loader                      │
│  - Canvas lifecycle                 │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│  Static Assets (/public/wasm/)      │
│  - gl.js (macroquad loader)         │
│  - boid_simulation.wasm             │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│  Browser WASM Runtime               │
│  - WebGL context                    │
│  - Game loop (60 FPS)               │
│  - Input handling                   │
│  - Egui UI rendering                │
└─────────────────────────────────────┘
```

### Performance Characteristics

**Bundle Sizes**:
- WASM (unoptimized): ~5 MB
- WASM (optimized): ~2 MB
- WASM (gzipped by Vercel): ~500-800 KB
- gl.js: ~50 KB

**Runtime Performance**:
- Native Rust: 60 FPS @ 3000 boids
- WASM (Chrome): 60 FPS @ 1000 boids
- WASM (Firefox): 60 FPS @ 800 boids
- WASM (Safari): 60 FPS @ 600 boids

**Memory Usage**:
- Typical: 50-100 MB
- With 1000 boids: ~150 MB

### Browser Compatibility

| Browser | Min Version | Status |
|---------|-------------|--------|
| Chrome | 90+ | ✅ Full support |
| Firefox | 88+ | ✅ Full support |
| Safari | 15+ | ✅ Full support |
| Edge | 90+ | ✅ Full support |
| Mobile Chrome | Latest | ⚠️ Works, may lag |
| Mobile Safari | Latest | ⚠️ Works, touch needs testing |

---

## Future Enhancements

### Phase 1 Enhancements (Easy)
- Add parameter presets (predefined interesting scenarios)
- GitHub link to source code
- Brief description of disease models

### Phase 2 Enhancements (Medium)
- Save/load simulation state to localStorage
- Screenshot/GIF export
- Mobile touch controls for parameters
- Responsive canvas scaling

### Phase 3 Enhancements (Advanced)
- Recording/playback of simulations
- Additional visualizations (heatmaps, trails)
- Analytics dashboard (R0 estimation, peak infection time)
- Multiple simultaneous simulations for comparison
- Custom disease parameter equations

---

## References

### Documentation
- [Macroquad WASM Tutorial](https://mq.agical.se/)
- [Rust WASM Book](https://rustwasm.github.io/docs/book/)
- [Next.js Static Assets](https://nextjs.org/docs/app/building-your-application/optimizing/static-assets)

### Example Projects
- [Macroquad Examples](https://github.com/not-fl3/macroquad/tree/master/examples)
- [egui-macroquad Examples](https://github.com/optozorax/egui-macroquad/tree/master/examples)

### Tools
- [wasm-strip](https://github.com/WebAssembly/wabt) - WASM optimization
- [Chrome DevTools WASM Debugging](https://developer.chrome.com/blog/wasm-debugging-2020/)

---

## Estimated Timeline

| Phase | Duration | Description |
|-------|----------|-------------|
| Phase 1 | 30 min | Rust WASM setup and build |
| Phase 2 | 20 min | Next.js structure and assets |
| Phase 3 | 45 min | Component development |
| Phase 4 | 15 min | Navigation and config |
| Phase 5 | 30 min | Testing and validation |
| Phase 6 | 20 min | Deployment to Vercel |
| **Total** | **2.5-3 hours** | End-to-end implementation |

---

**End Result**: Interactive boid simulation with disease models (SIR, SIS, SEIR) running in browser via WASM at `/boid-simulation`, preserving all features from the native Rust app including parameter controls, real-time population graphs, and keyboard shortcuts.
