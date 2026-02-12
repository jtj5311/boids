# Responsive Canvas: Hybrid Rust + TypeScript Approach

**Created**: 2024-12-30 11:26:44
**Status**: Planning
**Goal**: Make the boid simulation canvas responsive while maintaining quality and ease of maintenance

---

## Project Structure Overview

**IMPORTANT**: This project spans TWO separate repositories!

### Repository 1: Boids Simulation (Rust/WASM)
**Location**: `/home/leahm/Projects/boids/`

```
boids/
├── leah/
│   └── rust/
│       └── boid_simulation/          # Main Rust project
│           ├── Cargo.toml             # Rust dependencies & config
│           ├── .cargo/
│           │   └── config.toml        # WASM build config
│           ├── build_wasm.sh          # Build script (compiles & copies WASM)
│           ├── src/
│           │   ├── lib.rs             # WASM entry point (modify for responsive)
│           │   ├── main.rs            # Native binary entry point
│           │   ├── boid.rs            # Boid behavior logic
│           │   ├── simulation.rs      # Simulation parameters
│           │   ├── sir.rs             # Disease models (SIR, SIS, SEIR)
│           │   ├── spatial.rs         # Spatial hashing for performance
│           │   ├── ui.rs              # egui parameter panel
│           │   ├── visualization.rs   # Graph rendering
│           │   └── constants.rs       # Screen dimensions, etc.
│           └── target/
│               └── wasm32-unknown-unknown/
│                   └── release/
│                       └── boid_simulation.wasm  # Built WASM (2.3 MB)
```

**Purpose**: Contains the Rust simulation code and WASM build infrastructure

### Repository 2: Portfolio Website (Next.js/TypeScript)
**Location**: `/home/leahm/Projects/leahchilders-portfolio/`

```
leahchilders-portfolio/
└── web_app/                           # Next.js application
    ├── app/
    │   └── boids/                     # Boid simulation page
    │       ├── page.tsx               # Page layout & metadata
    │       └── BoidCanvas.tsx         # WASM loader component (modify for responsive)
    ├── components/
    │   └── landing-page/
    │       └── link-menu.tsx          # Right sidebar (has "Boids" link)
    ├── public/
    │   └── wasm/                      # Static WASM assets
    │       ├── boid_simulation.wasm   # Copied here by build_wasm.sh
    │       └── mq_js_bundle.js        # Macroquad WASM loader
    ├── types/
    │   └── wasm.d.ts                  # TypeScript declarations for WASM
    ├── next.config.ts                 # WASM headers configuration
    └── package.json                   # Node dependencies
```

**Purpose**: Portfolio website that embeds the WASM simulation

### How They Connect

1. **Build Process**:
   ```
   Rust code (boids repo)
       ↓ (cargo build --target wasm32-unknown-unknown)
   WASM binary created
       ↓ (build_wasm.sh copies)
   Portfolio public/wasm/ directory
       ↓ (Next.js serves)
   Browser at /boids route
   ```

2. **Development Workflow**:
   ```bash
   # Terminal 1: Modify Rust, rebuild WASM
   cd /home/leahm/Projects/boids/leah/rust/boid_simulation
   # Make changes to src/lib.rs
   ./build_wasm.sh

   # Terminal 2: Next.js auto-reloads
   cd /home/leahm/Projects/leahchilders-portfolio/web_app
   npm run dev
   # Browser auto-refreshes at http://localhost:3000/boids
   ```

3. **File Paths in build_wasm.sh**:
   ```bash
   # From: /home/leahm/Projects/boids/leah/rust/boid_simulation/
   # Relative path to portfolio: ../../../../leahchilders-portfolio/web_app/public/wasm
   # Absolute path: /home/leahm/Projects/leahchilders-portfolio/web_app/public/wasm
   ```

### Key Files for This Plan

**Files to Modify in BOIDS repo:**
- `/home/leahm/Projects/boids/leah/rust/boid_simulation/src/lib.rs`
  - Function: `window_conf()` (around line 18)
  - Change: Add `window_resizable: true, high_dpi: true`

**Files to Modify in PORTFOLIO repo:**
- `/home/leahm/Projects/leahchilders-portfolio/web_app/app/boids/BoidCanvas.tsx`
  - Remove: Complex scaling logic
  - Simplify: Canvas container JSX

**Files to Run:**
- `/home/leahm/Projects/boids/leah/rust/boid_simulation/build_wasm.sh`
  - Compiles WASM and copies to portfolio automatically

### Important Notes for Future You

1. **Two repos are separate** - changes in one don't affect git status of the other
2. **WASM must be rebuilt** after Rust changes (run `./build_wasm.sh`)
3. **Portfolio auto-reloads** WASM when file changes (if dev server running)
4. **Don't commit WASM binary** to git (it's in .gitignore, ~2.3 MB)
5. **The build script handles copying** - don't manually copy WASM files

---

## Problem Statement

Current implementation uses CSS `transform: scale()` which causes the simulation to not display at all. We need a robust responsive solution that:

1. **Scales properly** on all screen sizes (mobile, tablet, desktop)
2. **Maintains aspect ratio** (3:2 ratio, 1200x800)
3. **Preserves rendering quality** (no pixelation or artifacts)
4. **Handles input correctly** (mouse/touch coordinates)
5. **Is maintainable** by both Rust and web developers

---

## Current Issues

### What's Working:
- ✅ WASM builds successfully (2.3 MB)
- ✅ Simulation runs at full 1200x800 on desktop
- ✅ All features work (parameters, graph, keyboard shortcuts)
- ✅ Disease dynamics render correctly

### What's Broken:
- ❌ CSS transform approach breaks rendering entirely
- ❌ Canvas doesn't scale content on smaller screens
- ❌ Container scaling cuts off simulation edges

### Root Cause:
- WASM renders at fixed 1200x800 resolution
- CSS scaling only affects display size, not internal rendering
- No communication between browser size and WASM canvas size

---

## Solution: Hybrid Approach

**Philosophy**: Use the best tool for each job
- **Rust/WASM**: Handle rendering quality and DPI scaling
- **TypeScript/CSS**: Handle responsive layout and constraints

### How They Work Together:

```
Browser Container (responsive)
    ↓ (tells size via CSS)
Canvas Element (adapts to container)
    ↓ (reports size)
Macroquad (renders at proper resolution + DPI)
```

---

## Implementation Plan

### Part 1: Rust Side Changes

**File**: `/home/leahm/Projects/boids/leah/rust/boid_simulation/src/lib.rs`

**Changes**:

1. **Enable window resizing**:
   ```rust
   fn window_conf() -> Conf {
       Conf {
           window_title: "Boid Simulation with Disease Models - Press Enter to Restart".to_owned(),
           window_width: 1200,        // Max/default width
           window_height: 800,        // Max/default height
           window_resizable: true,    // ← ADD THIS
           high_dpi: true,            // ← ADD THIS
           ..Default::default()
       }
   }
   ```

2. **Why these changes**:
   - `window_resizable: true` - Allows canvas to adapt to container size
   - `high_dpi: true` - Enables device pixel ratio scaling (crisp on retina displays)
   - Macroquad automatically handles resize events via the JS loader

**Impact**:
- Minimal code change (2 lines)
- No logic changes needed
- Macroquad handles the rest automatically

---

### Part 2: TypeScript Side Changes

**File**: `/home/leahm/Projects/leahchilders-portfolio/web_app/app/boids/BoidCanvas.tsx`

**Changes**:

1. **Remove complex scaling logic**:
   - Remove `scale` state
   - Remove resize observer useEffect
   - Remove CSS transforms
   - Remove constants (or keep for reference)

2. **Simplify to basic responsive container**:
   ```tsx
   <div
     className="relative mx-auto w-full max-w-[1200px]"
     style={{
       aspectRatio: "3/2",
       display: isLoading || error ? "none" : "block",
     }}
   >
     <canvas
       ref={canvasRef}
       width={1200}
       height={800}
       className="block w-full h-full border border-border rounded-lg shadow-lg"
     />
   </div>
   ```

3. **How it works**:
   - `max-w-[1200px]` - Never exceeds original size
   - `w-full` - Takes full width up to max
   - `aspectRatio: "3/2"` - Maintains 1200:800 ratio
   - Canvas stretches to fill container
   - Macroquad's high DPI handles quality

**Impact**:
- Much simpler code (remove ~30 lines)
- No resize observers needed
- Browser + WASM work together naturally

---

### Part 3: Verify WASM Loader Support

**File**: `/home/leahm/Projects/leahchilders-portfolio/web_app/public/wasm/mq_js_bundle.js`

**Check that it has**:
- `setup_canvas_size(high_dpi)` function ✓
- `window.onresize` handler ✓
- `resize(canvas, callback)` support ✓

*Note: mq_js_bundle.js already has all this built-in for macroquad 0.4*

---

## How The Hybrid Solution Works

### On Page Load:

1. **Browser**: Creates responsive container (max 1200px wide, 3:2 aspect)
2. **Browser**: Canvas element fills container
3. **TypeScript**: Loads mq_js_bundle.js and WASM
4. **Rust**: Window config sets `high_dpi: true` and `window_resizable: true`
5. **Macroquad**: Reads canvas size from DOM, renders at proper resolution

### On Window Resize:

1. **Browser**: Container width changes (CSS handles this)
2. **Browser**: Canvas element resizes (fills container)
3. **JS Loader**: Detects canvas size change
4. **JS Loader**: Calls `wasm_exports.resize(width, height)` if available
5. **Macroquad**: Adjusts rendering accordingly

### Result:
- **Desktop (1920px wide)**: Full 1200x800 canvas
- **Laptop (1366px wide)**: Full 1200x800 canvas (fits in max-width)
- **Tablet (768px wide)**: Canvas scales to 768px wide × 512px tall (maintains ratio)
- **Mobile (375px wide)**: Canvas scales to 375px wide × 250px tall (maintains ratio)

**Quality**: High DPI ensures crisp rendering at all sizes

---

## Implementation Steps

### Step 1: Backup Current State ✓
Already in git, no additional backup needed

### Step 2: Modify Rust Code (5 min)

```bash
cd /home/leahm/Projects/boids/leah/rust/boid_simulation
# Edit src/lib.rs - modify window_conf()
```

Changes:
- Line ~18-26: Add `window_resizable: true, high_dpi: true,` to Conf

### Step 3: Rebuild WASM (2 min)

```bash
cd /home/leahm/Projects/boids/leah/rust/boid_simulation
./build_wasm.sh
```

Expected output: `Build complete! File copied... File size: 2.3M`

### Step 4: Simplify TypeScript Component (5 min)

```bash
cd /home/leahm/Projects/leahchilders-portfolio/web_app
# Edit app/boids/BoidCanvas.tsx
```

Changes:
- Remove: `const CANVAS_WIDTH/HEIGHT` (optional, can keep)
- Remove: `containerRef` ref
- Remove: `scale` state
- Remove: First useEffect (resize observer)
- Simplify: Canvas container JSX (see Part 2 above)

### Step 5: Test Locally (10 min)

1. Open http://localhost:3000/boids
2. Verify simulation loads and runs
3. Resize browser window
4. Check responsive behavior
5. Test on mobile (browser DevTools)

### Step 6: Deploy (User handles)

---

## Testing Checklist

### Desktop (1920px+):
- [ ] Canvas displays at full 1200x800
- [ ] Simulation runs smoothly
- [ ] All boids visible
- [ ] Parameters panel works
- [ ] Graph toggles (G key)
- [ ] Parameters toggle (P key)
- [ ] Restart works (Enter key)

### Laptop (1366px):
- [ ] Canvas displays at full 1200x800 (should fit)
- [ ] No horizontal scroll
- [ ] All features work

### Tablet (768px):
- [ ] Canvas scales proportionally
- [ ] Maintains 3:2 aspect ratio
- [ ] All boids visible (scaled down)
- [ ] No cutoff edges
- [ ] Touch/mouse input works

### Mobile (375px):
- [ ] Canvas scales to fit screen
- [ ] Maintains 3:2 aspect ratio
- [ ] Simulation visible and readable
- [ ] "Better on larger screens/desktop" message visible
- [ ] Touch input works

### Quality:
- [ ] No pixelation on retina displays
- [ ] Crisp rendering at all sizes
- [ ] Colors accurate
- [ ] Graph readable

### Performance:
- [ ] 60 FPS on desktop
- [ ] Acceptable FPS on mobile
- [ ] No memory leaks
- [ ] Smooth resize transitions

---

## Rollback Plan

If the hybrid approach doesn't work:

1. **Revert Rust changes**:
   ```bash
   cd /home/leahm/Projects/boids/leah/rust/boid_simulation
   git checkout src/lib.rs
   ./build_wasm.sh
   ```

2. **Revert TypeScript changes**:
   ```bash
   cd /home/leahm/Projects/leahchilders-portfolio/web_app
   git checkout app/boids/BoidCanvas.tsx
   ```

3. **Alternative**: Fixed size on mobile, full size on desktop
   ```tsx
   className="hidden sm:block w-[1200px] h-[800px]"
   // + mobile message: "Please view on desktop"
   ```

---

## Future Enhancements (Post-Implementation)

### Phase 1 (Easy):
- Add loading progress indicator
- Preload screen with instructions
- Keyboard shortcut reference card

### Phase 2 (Medium):
- Touch controls for mobile parameter adjustment
- Responsive parameter panel (collapsible on mobile)
- Pinch-to-zoom on mobile

### Phase 3 (Advanced):
- Multiple canvas sizes (small/medium/large buttons)
- Fullscreen mode
- Screenshot/download functionality
- Share simulation state via URL

---

## Technical Notes

### Why High DPI Works:

Macroquad's `high_dpi: true` does this automatically:
```
Browser says: Canvas is 600px × 400px
Device pixel ratio: 2 (retina display)
Macroquad renders: 1200px × 800px internally
Browser displays: Scaled to 600px × 400px
Result: Crisp, high-quality rendering
```

### Why Window Resizable Works:

When `window_resizable: true`:
- Macroquad reads canvas dimensions from DOM
- Adjusts viewport automatically
- Handles resize events via JS loader
- No manual coordinate transformation needed

### Browser Support:

- `aspect-ratio` CSS: Chrome 88+, Firefox 89+, Safari 15+
- High DPI support: All modern browsers
- Canvas scaling: Universal support

---

## Success Criteria

✅ Simulation displays correctly on all screen sizes
✅ No code complexity (simple CSS, minimal Rust change)
✅ High quality rendering (no pixelation)
✅ Maintainable by web developers
✅ Input coordinates work correctly
✅ Performance acceptable on mobile
✅ No breaking changes to existing features

---

## References

- [Macroquad Window Config](https://docs.rs/macroquad/latest/macroquad/conf/struct.Conf.html)
- [MDN: aspect-ratio CSS](https://developer.mozilla.org/en-US/docs/Web/CSS/aspect-ratio)
- [Canvas High DPI](https://developer.mozilla.org/en-US/docs/Web/API/Window/devicePixelRatio)
- [Macroquad WASM Examples](https://github.com/not-fl3/macroquad/tree/master/examples)

---

## Appendix: Code Diffs

### A. Rust Changes

**File**: `src/lib.rs`

```diff
 fn window_conf() -> Conf {
     Conf {
         window_title: "Boid Simulation with Disease Models - Press Enter to Restart".to_owned(),
         window_width: SCREEN_WIDTH as i32,
         window_height: SCREEN_HEIGHT as i32,
-        window_resizable: false,
+        window_resizable: true,
+        high_dpi: true,
         ..Default::default()
     }
 }
```

### B. TypeScript Changes

**File**: `app/boids/BoidCanvas.tsx`

```diff
-const CANVAS_WIDTH = 1200;
-const CANVAS_HEIGHT = 800;
-
 export function BoidCanvas() {
   const canvasRef = useRef<HTMLCanvasElement>(null);
-  const containerRef = useRef<HTMLDivElement>(null);
   const [isLoading, setIsLoading] = useState(true);
   const [error, setError] = useState<string | null>(null);
-  const [scale, setScale] = useState(1);
-
-  // Handle responsive scaling
-  useEffect(() => {
-    const updateScale = () => {
-      if (containerRef.current) {
-        const containerWidth = containerRef.current.offsetWidth;
-        const newScale = Math.min(1, containerWidth / CANVAS_WIDTH);
-        setScale(newScale);
-      }
-    };
-
-    updateScale();
-    window.addEventListener("resize", updateScale);
-    return () => window.removeEventListener("resize", updateScale);
-  }, []);

   // ... (WASM loading useEffect stays the same)

   return (
     <div className="flex flex-col items-center justify-center gap-4">
       {/* Loading and error states stay the same */}

       <div
-        ref={containerRef}
-        className="relative mx-auto w-full max-w-[1200px]"
+        className="relative mx-auto w-full max-w-[1200px]"
         style={{
+          aspectRatio: "3/2",
           display: isLoading || error ? "none" : "block",
         }}
       >
-        <div
-          className="border border-border rounded-lg overflow-hidden shadow-lg origin-top-left"
-          style={{
-            width: `${CANVAS_WIDTH}px`,
-            height: `${CANVAS_HEIGHT}px`,
-            transform: `scale(${scale})`,
-            transformOrigin: "top left",
-          }}
-        >
-          <canvas
-            ref={canvasRef}
-            width={CANVAS_WIDTH}
-            height={CANVAS_HEIGHT}
-            className="block"
-          />
-        </div>
+        <canvas
+          ref={canvasRef}
+          width={1200}
+          height={800}
+          className="block w-full h-full border border-border rounded-lg shadow-lg"
+        />
       </div>
     </div>
   );
 }
```

---

**End of Plan**

*Ready for implementation when you give the go-ahead!*
