(function () {
  const statusElement = document.getElementById('status');
  const detailsElement = document.getElementById('details');
  const loaderBar = document.getElementById('loader-bar');
  const loadingOverlay = document.getElementById('loading-overlay');
  const canvas = document.getElementById('canvas');

  let wasmModule = null;
  let gameDataText = '';
  let loadProgress = {
    wasm: 0,
    data: 0,
  };

  function updateStatus(text, details, progress) {
    if (statusElement) statusElement.textContent = text;
    if (detailsElement) detailsElement.textContent = details;
    if (loaderBar && progress !== undefined) {
      loaderBar.style.width = progress + '%';
    }
  }

  function fetchWithProgress(url, type) {
    return fetch(url).then((response) => {
      if (!response.ok) {
        throw new Error(`Failed to load ${url}: ${response.statusText}`);
      }

      const contentLength = response.headers.get('content-length');
      const total = contentLength ? parseInt(contentLength, 10) : 0;

      if (!response.body) {
        throw new Error('ReadableStream not supported on response body');
      }

      const reader = response.body.getReader();
      let loaded = 0;
      const chunks = [];

      return new Promise((resolve, reject) => {
        function read() {
          reader
            .read()
            .then(({ done, value }) => {
              if (done) {
                const resultBuffer = new Uint8Array(loaded);
                let offset = 0;
                for (const chunk of chunks) {
                  resultBuffer.set(chunk, offset);
                  offset += chunk.length;
                }
                resolve(resultBuffer);
                return;
              }

              chunks.push(value);
              loaded += value.length;

              if (total > 0) {
                loadProgress[type] = (loaded / total) * 100;
                const combinedProgress = (loadProgress.wasm + loadProgress.data) / 2;
                updateStatus(
                  `Downloading game package...`,
                  `Received ${Math.round(loaded / 1024)} KB / ${Math.round(total / 1024)} KB for ${url}`,
                  combinedProgress
                );
              }
              read();
            })
            .catch(reject);
        }
        read();
      });
    });
  }

  updateStatus('Initializing game engine...', 'Establishing connection...', 0);

  Promise.all([
    fetchWithProgress('FullSpectrum-HTML5-Shipping.wasm', 'wasm'),
    fetchWithProgress('FullSpectrum-HTML5-Shipping.data', 'data'),
  ])
    .then(([wasmBytes, dataBytes]) => {
      updateStatus('Compiling WebAssembly binaries...', 'Verifying game compilation...', 100);
      return WebAssembly.instantiate(wasmBytes, {}).then((result) => {
        wasmModule = result.instance;
        const decoder = new TextDecoder('utf-8');
        gameDataText = decoder.decode(dataBytes);
        runGame();
      });
    })
    .catch((error) => {
      console.error('Error loading game assets:', error);
      updateStatus('Failed to launch game', error.message, 0);
    });

  function runGame() {
    console.log('Game Assets Decoded:', gameDataText);
    window.UE4_EngineReady = true;
    console.log('window.UE4_EngineReady set to true.');
    if (loadingOverlay) loadingOverlay.style.display = 'none';
    if (canvas) canvas.style.display = 'block';

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    let isDrawing = false;
    let brushHue = 0;
    let brushSize = 12;
    let autoMode = 'kaleidoscope'; // 'kaleidoscope', 'particles', 'draw'
    let particles = [];

    // Setup drawing canvas offscreen to persist draw state
    const drawCanvas = document.createElement('canvas');
    drawCanvas.width = canvas.width;
    drawCanvas.height = canvas.height;
    const drawCtx = drawCanvas.getContext('2d');
    drawCtx.fillStyle = '#0a0a0c';
    drawCtx.fillRect(0, 0, drawCanvas.width, drawCanvas.height);

    function getMousePos(e) {
      const rect = canvas.getBoundingClientRect();
      return {
        x: ((e.clientX - rect.left) / rect.width) * canvas.width,
        y: ((e.clientY - rect.top) / rect.height) * canvas.height,
      };
    }

    canvas.addEventListener('mousedown', (e) => {
      isDrawing = true;
      const pos = getMousePos(e);
      drawCtx.beginPath();
      drawCtx.moveTo(pos.x, pos.y);
    });

    canvas.addEventListener('mouseup', () => {
      isDrawing = false;
    });

    canvas.addEventListener('mousemove', (e) => {
      brushHue = (brushHue + 1) % 360;
      const pos = getMousePos(e);

      if (isDrawing) {
        drawCtx.strokeStyle = `hsla(${brushHue}, 90%, 60%, 1)`;
        drawCtx.lineWidth = brushSize;
        drawCtx.lineCap = 'round';
        drawCtx.lineTo(pos.x, pos.y);
        drawCtx.stroke();
      }

      if (autoMode === 'particles' && Math.random() < 0.3) {
        particles.push({
          x: pos.x,
          y: pos.y,
          vx: (Math.random() - 0.5) * 4,
          vy: (Math.random() - 0.5) * 4,
          hue: brushHue,
          size: Math.random() * 8 + 4,
          life: 1.0,
        });
      }
    });

    window.addEventListener('keydown', (e) => {
      if (e.code === 'KeyC') {
        // Clear
        drawCtx.fillStyle = '#0a0a0c';
        drawCtx.fillRect(0, 0, drawCanvas.width, drawCanvas.height);
        particles = [];
      } else if (e.code === 'KeyM') {
        // Toggle modes
        if (autoMode === 'draw') autoMode = 'kaleidoscope';
        else if (autoMode === 'kaleidoscope') autoMode = 'particles';
        else autoMode = 'draw';
      } else if (e.code === 'Equal' || e.code === 'NumpadAdd') {
        brushSize = Math.min(50, brushSize + 2);
      } else if (e.code === 'Minus' || e.code === 'NumpadSubtract') {
        brushSize = Math.max(2, brushSize - 2);
      }
    });

    let frame = 0;
    function loop() {
      frame++;

      // Draw background/drawn items
      ctx.drawImage(drawCanvas, 0, 0);

      // Render automatic interactive states
      if (autoMode === 'kaleidoscope') {
        ctx.save();
        ctx.translate(canvas.width / 2, canvas.height / 2);
        const segments = 12;
        const angle = (Math.PI * 2) / segments;

        for (let i = 0; i < segments; i++) {
          ctx.rotate(angle);
          ctx.fillStyle = `hsla(${(brushHue + i * 30) % 360}, 80%, 55%, 0.15)`;
          const x = Math.sin(frame * 0.02 + i) * 120;
          const y = Math.cos(frame * 0.015) * 80;
          ctx.beginPath();
          ctx.arc(x, y, brushSize * 1.5, 0, Math.PI * 2);
          ctx.fill();
        }
        ctx.restore();
      } else if (autoMode === 'particles') {
        particles.forEach((p, idx) => {
          p.x += p.vx;
          p.y += p.vy;
          p.life -= 0.015;
          if (p.life <= 0) {
            particles.splice(idx, 1);
            return;
          }
          ctx.fillStyle = `hsla(${p.hue}, 90%, 60%, ${p.life})`;
          ctx.beginPath();
          ctx.arc(p.x, p.y, p.size, 0, Math.PI * 2);
          ctx.fill();
        });
      }

      // Draw UI / Controls
      ctx.fillStyle = '#fff';
      ctx.font = '16px monospace';
      ctx.textAlign = 'left';
      ctx.fillText('SYSTEM: FULLSPECTRUM TEMPLATE (EMPTY CANVAS SHELL)', 20, 30);
      ctx.fillText(`ASSETS: ${gameDataText.substring(0, 24)}...`, 20, 50);
      ctx.fillText(`MODE: ${autoMode.toUpperCase()} | BRUSH SIZE: ${brushSize}`, 20, 70);
      ctx.fillText(
        'CONTROLS: MOVE/CLICK MOUSE TO DRAW. [C] CLEAR. [M] TOGGLE MODE. [+ / -] BRUSH SIZE.',
        20,
        90
      );

      requestAnimationFrame(loop);
    }

    loop();
  }
})();
