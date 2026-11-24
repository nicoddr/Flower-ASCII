(() => {
  'use strict';

  const stage = document.getElementById('stage');
  const canvas = document.getElementById('ascii-canvas');
  const ctx = canvas.getContext('2d', { alpha: false });
  const telemetryEl = document.getElementById('telemetry');
  const statusOverlay = document.getElementById('status-overlay');

  const btnToggle = document.getElementById('btn-toggle');
  const btnBloom = document.getElementById('btn-bloom');
  const btnPhosphor = document.getElementById('btn-phosphor');
  const btnCopy = document.getElementById('btn-copy');
  const windRange = document.getElementById('wind-range');
  const windLabel = document.getElementById('wind-label');

  const FONT_FAMILY = "'Cascadia Mono', 'JetBrains Mono', 'Fira Code', 'Consolas', monospace";
  const GLYPH_TABLE = Array.from({ length: 128 }, (_, i) => String.fromCharCode(i));
  const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  let wasm = null;
  let paused = false;
  let phosphor = false;

  let cols = 118;
  let rows = 58;
  let fontSize = 14;
  let cellW = 8;
  let cellH = 14;
  let originX = 0;
  let originY = 0;

  let lastTime = performance.now();
  let fpsAccum = 0;
  let fpsFrames = 0;
  let displayFps = 60;

  let idleTimer = null;
  function bumpActivity() {
    stage.classList.remove('idle');
    clearTimeout(idleTimer);
    idleTimer = setTimeout(() => stage.classList.add('idle'), 4800);
  }

  function showError(message) {
    statusOverlay.textContent = message;
    statusOverlay.hidden = false;
  }

  function isPhoneLayout() {
    return window.innerWidth <= 640;
  }

  function updateTelemetry() {
    if (isPhoneLayout()) {
      telemetryEl.textContent = paused ? 'PAUSED' : `${cols}×${rows}`;
    } else {
      telemetryEl.textContent = `${cols}×${rows} · ${paused ? 'PAUSED' : displayFps + ' FPS'}`;
    }
  }

  async function loadWasmModule() {
    // 1. Standard static hosting (GitHub Pages or any static host)
    if (window.location.protocol !== 'file:') {
      try {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
          const { instance } = await WebAssembly.instantiateStreaming(fetch('./bouquet.wasm'), {});
          return instance.exports;
        }
      } catch (_err) {
        // Fallback to ArrayBuffer if the host does not send application/wasm
      }
      try {
        const res = await fetch('./bouquet.wasm');
        if (res.ok) {
          const buf = await res.arrayBuffer();
          const { instance } = await WebAssembly.instantiate(buf, {});
          return instance.exports;
        }
      } catch (_err) {
        // Continue to embedded fallback
      }
    }

    // 2. Embedded base64 fallback (exact same compiled Rust binary) for direct file:// opening
    if (typeof window.__BOUQUET_WASM_B64__ === 'string' && window.__BOUQUET_WASM_B64__.length > 0) {
      const binaryStr = atob(window.__BOUQUET_WASM_B64__);
      const bytes = new Uint8Array(binaryStr.length);
      for (let i = 0; i < binaryStr.length; i++) {
        bytes[i] = binaryStr.charCodeAt(i);
      }
      const { instance } = await WebAssembly.instantiate(bytes.buffer, {});
      return instance.exports;
    }

    throw new Error(
      'Unable to load bouquet.wasm. Run build.ps1 to compile the Rust/WebAssembly module.'
    );
  }

  function configureGrid() {
    const phone = isPhoneLayout();
    const dpr = Math.min(window.devicePixelRatio || 1, phone ? 3 : 2);
    const vw = window.innerWidth;
    const vh = window.innerHeight;

    canvas.width = Math.max(1, Math.floor(vw * dpr));
    canvas.height = Math.max(1, Math.floor(vh * dpr));
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

    // Reserve dedicated safe space for the top bar and mobile/desktop bottom controls
    const topReserve = phone ? 44 : 44;
    const bottomReserve = phone ? 104 : 48;
    const sideMargin = phone ? 12 : 24;

    const usableH = Math.max(220, vh - topReserve - bottomReserve);
    const usableW = Math.max(220, vw - sideMargin);

    if (phone) {
      // On phones, render a fine ~86-92 column ASCII grid so every peony ruffle, sepal & gypsophila shines
      const desiredCols = vw < 360 ? 78 : 88;
      const maxFontByW = usableW / (desiredCols * 0.61);
      const maxFontByH = usableH / 76;
      fontSize = Math.max(6.5, Math.min(10.0, Math.floor(Math.min(maxFontByW, maxFontByH) * 2) / 2));
    } else {
      const targetRows = vw < 1024 ? 66 : 76;
      fontSize = Math.max(8.5, Math.min(15.5, Math.floor((usableH / targetRows) * 2) / 2));
    }

    ctx.font = `600 ${fontSize}px ${FONT_FAMILY}`;
    const measuredW = ctx.measureText('@').width;

    // Align cell pitch cleanly to physical pixels for sharp monospace rendering on Retina displays
    cellW = Math.max(4.0, Math.round(measuredW * 1.04 * dpr) / dpr);
    cellH = Math.max(7.0, Math.round(fontSize * 1.12 * dpr) / dpr);

    rows = Math.max(32, Math.min(165, Math.floor(usableH / cellH)));
    cols = Math.max(48, Math.min(265, Math.floor(usableW / cellW)));

    originX = Math.floor((vw - cols * cellW) * 0.5);
    originY = Math.floor(topReserve + (usableH - rows * cellH) * 0.5);

    if (wasm) {
      wasm.init(cols, rows);
      wasm.set_aspect(cellW / cellH);
      cols = wasm.cols();
      rows = wasm.rows();
      updateTelemetry();
    }
  }

  function drawFrame() {
    const vw = window.innerWidth;
    const vh = window.innerHeight;

    ctx.fillStyle = '#000000';
    ctx.fillRect(0, 0, vw, vh);

    const ptr = wasm.buffer_ptr();
    const len = wasm.buffer_len();
    const buf = new Uint8Array(wasm.memory.buffer, ptr, len);

    ctx.font = `600 ${fontSize}px ${FONT_FAMILY}`;
    ctx.textBaseline = 'top';
    ctx.textAlign = 'left';
    ctx.shadowBlur = phosphor ? (isPhoneLayout() ? 5 : 7) : 0;

    let lastColor = '';
    for (let row = 0; row < rows; row++) {
      const y = originY + row * cellH;
      const rowOffset = row * cols;
      for (let col = 0; col < cols; col++) {
        const i = (rowOffset + col) * 4;
        const code = buf[i];
        if (code === 32) continue;

        const r = buf[i + 1];
        const g = buf[i + 2];
        const b = buf[i + 3];
        const color = `rgb(${r},${g},${b})`;

        if (color !== lastColor) {
          ctx.fillStyle = color;
          if (phosphor) ctx.shadowColor = color;
          lastColor = color;
        }

        ctx.fillText(GLYPH_TABLE[code] || ' ', originX + col * cellW, y);
      }
    }

    if (phosphor) ctx.shadowBlur = 0;
  }

  function tick(now) {
    const dt = Math.min(0.05, Math.max(0.0, (now - lastTime) / 1000));
    lastTime = now;

    if (!paused) {
      wasm.render(dt);
      drawFrame();
    }

    fpsAccum += dt;
    fpsFrames += 1;
    if (fpsAccum >= 0.45) {
      displayFps = Math.round(fpsFrames / fpsAccum);
      updateTelemetry();
      fpsAccum = 0;
      fpsFrames = 0;
    }

    requestAnimationFrame(tick);
  }

  function updateWindLabel(val) {
    if (val <= 0.05) windLabel.textContent = 'Calm';
    else if (val < 1.35) windLabel.textContent = 'Breeze';
    else windLabel.textContent = 'Gust';
  }

  function bindControls() {
    const togglePause = () => {
      paused = !paused;
      btnToggle.textContent = paused ? 'Resume' : 'Pause';
      btnToggle.setAttribute('aria-pressed', String(paused));
      updateTelemetry();
      bumpActivity();
    };
    btnToggle.addEventListener('click', togglePause);

    window.addEventListener('keydown', (e) => {
      if (e.code === 'Space' && document.activeElement !== windRange) {
        e.preventDefault();
        togglePause();
      }
    });

    windRange.addEventListener('input', () => {
      const val = parseFloat(windRange.value);
      wasm.set_wind(val);
      updateWindLabel(val);
      if (paused) {
        wasm.render(0.0);
        drawFrame();
      }
      bumpActivity();
    });

    btnBloom.addEventListener('click', () => {
      if (paused) togglePause();
      wasm.trigger_bloom();
      bumpActivity();
    });

    btnPhosphor.addEventListener('click', () => {
      phosphor = !phosphor;
      stage.classList.toggle('phosphor-on', phosphor);
      btnPhosphor.setAttribute('aria-pressed', String(phosphor));
      drawFrame();
      bumpActivity();
    });

    btnCopy.addEventListener('click', async () => {
      const ptr = wasm.copy_ascii_ptr();
      const len = wasm.copy_ascii_len();
      const bytes = new Uint8Array(wasm.memory.buffer, ptr, len);
      const text = new TextDecoder('utf-8').decode(bytes);
      try {
        await navigator.clipboard.writeText(text);
        btnCopy.textContent = 'Copied';
      } catch (_err) {
        btnCopy.textContent = 'Unavailable';
      }
      setTimeout(() => {
        btnCopy.textContent = 'Copy ASCII';
      }, 1500);
      bumpActivity();
    });

    const onPointerMove = (clientX, clientY) => {
      const nx = (clientX / Math.max(1, window.innerWidth)) * 2 - 1;
      const ny = 1 - (clientY / Math.max(1, window.innerHeight)) * 2;
      wasm.set_pointer(nx, ny, 1);
      bumpActivity();
    };

    window.addEventListener('pointermove', (e) => onPointerMove(e.clientX, e.clientY), { passive: true });
    window.addEventListener('pointerleave', () => wasm.set_pointer(0, 0, 0), { passive: true });
    canvas.addEventListener('touchstart', (e) => {
      if (e.touches.length > 0) {
        onPointerMove(e.touches[0].clientX, e.touches[0].clientY);
      }
    }, { passive: true });
    canvas.addEventListener('touchmove', (e) => {
      if (e.touches.length > 0) {
        onPointerMove(e.touches[0].clientX, e.touches[0].clientY);
      }
    }, { passive: true });
    canvas.addEventListener('touchend', () => wasm.set_pointer(0, 0, 0), { passive: true });
    canvas.addEventListener('touchcancel', () => wasm.set_pointer(0, 0, 0), { passive: true });

    let resizeTimer = null;
    const handleResize = () => {
      clearTimeout(resizeTimer);
      resizeTimer = setTimeout(() => {
        configureGrid();
        wasm.render(0.0);
        drawFrame();
      }, 90);
    };
    window.addEventListener('resize', handleResize);
    window.addEventListener('orientationchange', handleResize);
  }

  async function boot() {
    try {
      wasm = await loadWasmModule();
      configureGrid();

      if (reducedMotion) {
        windRange.value = '0.2';
        wasm.set_wind(0.2);
        updateWindLabel(0.2);
        wasm.set_bloom_instant();
      } else {
        wasm.set_wind(1.0);
        updateWindLabel(1.0);
        wasm.trigger_bloom();
      }

      bindControls();
      bumpActivity();
      wasm.render(0.016);
      drawFrame();

      lastTime = performance.now();
      requestAnimationFrame(tick);
    } catch (err) {
      showError(err instanceof Error ? err.message : String(err));
    }
  }

  boot();
})();
