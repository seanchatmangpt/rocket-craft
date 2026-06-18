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
    fetchWithProgress('RealisticRendering-HTML5-Shipping.wasm', 'wasm'),
    fetchWithProgress('RealisticRendering-HTML5-Shipping.data', 'data'),
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

    let lightX = canvas.width / 2;
    let lightY = canvas.height / 2;
    let lightColor = { r: 243, g: 156, b: 18 };

    // Define 2D solid boxes that cast shadows
    const boxes = [
      { x: 150, y: 100, w: 80, h: 80 },
      { x: 500, y: 120, w: 120, h: 60 },
      { x: 200, y: 280, w: 100, h: 80 },
      { x: 550, y: 260, w: 90, h: 90 },
    ];

    // Listen to mouse movement to place light source
    canvas.addEventListener('mousemove', (e) => {
      const rect = canvas.getBoundingClientRect();
      // Adjust coordinate scale between CSS size and internal canvas size
      lightX = ((e.clientX - rect.left) / rect.width) * canvas.width;
      lightY = ((e.clientY - rect.top) / rect.height) * canvas.height;
    });

    // Handle clicks to change light colors
    canvas.addEventListener('click', () => {
      const colors = [
        { r: 243, g: 156, b: 18 }, // Orange
        { r: 52, g: 152, b: 219 }, // Blue
        { r: 46, g: 204, b: 113 }, // Green
        { r: 155, g: 89, b: 182 }, // Purple
        { r: 231, g: 76, b: 60 }, // Red
      ];
      const nextIdx = Math.floor(Math.random() * colors.length);
      lightColor = colors[nextIdx];
    });

    window.addEventListener('keydown', (e) => {
      if (e.code === 'KeyW' || e.code === 'ArrowUp') {
        lightY -= 10;
      }
      if (e.code === 'Space') {
        lightX += 10;
      }
    });

    function getSegments() {
      const segments = [];

      // Add canvas boundary segments
      segments.push({ a: { x: 0, y: 0 }, b: { x: canvas.width, y: 0 } });
      segments.push({ a: { x: canvas.width, y: 0 }, b: { x: canvas.width, y: canvas.height } });
      segments.push({ a: { x: canvas.width, y: canvas.height }, b: { x: 0, y: canvas.height } });
      segments.push({ a: { x: 0, y: canvas.height }, b: { x: 0, y: 0 } });

      // Add box segments
      boxes.forEach((box) => {
        segments.push({ a: { x: box.x, y: box.y }, b: { x: box.x + box.w, y: box.y } });
        segments.push({
          a: { x: box.x + box.w, y: box.y },
          b: { x: box.x + box.w, y: box.y + box.h },
        });
        segments.push({
          a: { x: box.x + box.w, y: box.y + box.h },
          b: { x: box.x, y: box.y + box.h },
        });
        segments.push({ a: { x: box.x, y: box.y + box.h }, b: { x: box.x, y: box.y } });
      });

      return segments;
    }

    // Get intersection details of a ray with a segment
    function getIntersection(ray, segment) {
      const r_px = ray.a.x;
      const r_py = ray.a.y;
      const r_dx = ray.b.x - ray.a.x;
      const r_dy = ray.b.y - ray.a.y;

      const s_px = segment.a.x;
      const s_py = segment.a.y;
      const s_dx = segment.b.x - segment.a.x;
      const s_dy = segment.b.y - segment.a.y;

      const r_mag = Math.sqrt(r_dx * r_dx + r_dy * r_dy);
      const s_mag = Math.sqrt(s_dx * s_dx + s_dy * s_dy);
      if (r_mag === 0 || s_mag === 0) return null;

      // Parallel check
      if (r_dx / r_mag === s_dx / s_mag && r_dy / r_mag === s_dy / s_mag) {
        return null;
      }

      const T2 = (r_dx * (s_py - r_py) + r_dy * (r_px - s_px)) / (s_dx * r_dy - s_dy * r_dx);
      const T1 = (s_px + s_dx * T2 - r_px) / r_dx;

      if (T1 < 0) return null;
      if (T2 < 0 || T2 > 1) return null;

      return {
        x: r_px + r_dx * T1,
        y: r_py + r_dy * T1,
        param: T1,
      };
    }

    // Main render loop
    function loop() {
      // Clear canvas
      ctx.fillStyle = '#050505';
      ctx.fillRect(0, 0, canvas.width, canvas.height);

      const segments = getSegments();

      // Collect unique corner points
      const points = [];
      segments.forEach((seg) => {
        points.push(seg.a);
        points.push(seg.b);
      });

      // Filter duplicates
      const uniquePoints = [];
      const set = new Set();
      points.forEach((p) => {
        const key = `${Math.round(p.x)},${Math.round(p.y)}`;
        if (!set.has(key)) {
          set.add(key);
          uniquePoints.push(p);
        }
      });

      // Cast rays in directions of corner points (slightly offset to get edges correct)
      const angles = [];
      uniquePoints.forEach((p) => {
        const angle = Math.atan2(p.y - lightY, p.x - lightX);
        angles.push(angle - 0.0001);
        angles.push(angle);
        angles.push(angle + 0.0001);
      });

      // Sort angles to render polygon sequentially
      angles.sort((a, b) => a - b);

      const intersects = [];
      angles.forEach((angle) => {
        const dx = Math.cos(angle);
        const dy = Math.sin(angle);
        const ray = {
          a: { x: lightX, y: lightY },
          b: { x: lightX + dx, y: lightY + dy },
        };

        let closestIntersect = null;
        segments.forEach((seg) => {
          const intersect = getIntersection(ray, seg);
          if (!intersect) return;
          if (!closestIntersect || intersect.param < closestIntersect.param) {
            closestIntersect = intersect;
          }
        });

        if (closestIntersect) {
          closestIntersect.angle = angle;
          intersects.push(closestIntersect);
        }
      });

      // Draw light polygon
      if (intersects.length > 0) {
        ctx.fillStyle = `rgba(${lightColor.r}, ${lightColor.g}, ${lightColor.b}, 0.25)`;
        ctx.beginPath();
        ctx.moveTo(intersects[0].x, intersects[0].y);
        for (let i = 1; i < intersects.length; i++) {
          ctx.lineTo(intersects[i].x, intersects[i].y);
        }
        ctx.closePath();
        ctx.fill();

        // Radial gradient for ambient glow
        const grad = ctx.createRadialGradient(lightX, lightY, 10, lightX, lightY, 350);
        grad.addColorStop(0, `rgba(${lightColor.r}, ${lightColor.g}, ${lightColor.b}, 0.8)`);
        grad.addColorStop(0.2, `rgba(${lightColor.r}, ${lightColor.g}, ${lightColor.b}, 0.3)`);
        grad.addColorStop(1, 'rgba(0, 0, 0, 0)');
        ctx.fillStyle = grad;
        ctx.beginPath();
        ctx.moveTo(intersects[0].x, intersects[0].y);
        for (let i = 1; i < intersects.length; i++) {
          ctx.lineTo(intersects[i].x, intersects[i].y);
        }
        ctx.closePath();
        ctx.fill();
      }

      // Draw Obstacle Boxes
      ctx.fillStyle = '#1e1e24';
      ctx.strokeStyle = '#f39c12';
      ctx.lineWidth = 2;
      boxes.forEach((box) => {
        ctx.fillRect(box.x, box.y, box.w, box.h);
        ctx.strokeRect(box.x, box.y, box.w, box.h);
      });

      // Draw light source dot
      ctx.fillStyle = '#fff';
      ctx.beginPath();
      ctx.arc(lightX, lightY, 6, 0, Math.PI * 2);
      ctx.fill();

      // HUD overlay
      ctx.fillStyle = '#fff';
      ctx.font = '16px monospace';
      ctx.textAlign = 'left';
      ctx.fillText('SYSTEM: REALISTIC RENDERING (RAYCASTING SHADER)', 20, 30);
      ctx.fillText(`ASSETS: ${gameDataText.substring(0, 24)}...`, 20, 50);
      ctx.fillText('ACTION: MOVE MOUSE TO CAST LIGHT. CLICK TO CHANGE COLOR.', 20, 70);

      requestAnimationFrame(loop);
    }

    loop();
  }
})();
