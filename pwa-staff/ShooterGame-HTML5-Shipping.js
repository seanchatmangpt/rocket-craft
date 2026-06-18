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
    fetchWithProgress('ShooterGame-HTML5-Shipping.wasm', 'wasm'),
    fetchWithProgress('ShooterGame-HTML5-Shipping.data', 'data'),
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

    // Map: 1 represents a wall, 0 represents empty space
    const map = [
      [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
      [1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
      [1, 0, 1, 1, 0, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1],
      [1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1],
      [1, 0, 1, 0, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1, 0, 1],
      [1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 1],
      [1, 1, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 1],
      [1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1],
      [1, 0, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 1],
      [1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 0, 0, 1],
      [1, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1, 1, 1],
      [1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1],
      [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    ];

    const MAP_WIDTH = map[0].length;
    const MAP_HEIGHT = map.length;
    const TILE_SIZE = 64;

    let posX = 1.5 * TILE_SIZE;
    let posY = 1.5 * TILE_SIZE;
    let dirX = 1.0;
    let dirY = 0.0;
    let angle = 0.0; // Rotation angle

    const FOV = Math.PI / 3; // 60 degrees
    const HALF_FOV = FOV / 2;
    const NUM_RAYS = canvas.width;
    const STEP_ANGLE = FOV / NUM_RAYS;
    const MAX_DEPTH = 800;

    const keys = {};

    window.addEventListener('keydown', (e) => {
      keys[e.code] = true;
    });

    window.addEventListener('keyup', (e) => {
      keys[e.code] = false;
    });

    function loop() {
      // Movement controls
      const moveSpeed = 3.0;
      const rotSpeed = 0.04;

      if (keys['KeyA'] || keys['ArrowLeft']) {
        angle -= rotSpeed;
      }
      if (keys['KeyD'] || keys['ArrowRight']) {
        angle += rotSpeed;
      }

      dirX = Math.cos(angle);
      dirY = Math.sin(angle);

      let newX = posX;
      let newY = posY;

      if (keys['KeyW'] || keys['ArrowUp']) {
        newX += dirX * moveSpeed;
        newY += dirY * moveSpeed;
      }
      if (keys['KeyS'] || keys['ArrowDown']) {
        newX -= dirX * moveSpeed;
        newY -= dirY * moveSpeed;
      }

      // Simple collision check
      const checkMapX = Math.floor(newX / TILE_SIZE);
      const checkMapY = Math.floor(newY / TILE_SIZE);
      if (checkMapX >= 0 && checkMapX < MAP_WIDTH && checkMapY >= 0 && checkMapY < MAP_HEIGHT) {
        if (map[checkMapY][checkMapX] === 0) {
          posX = newX;
          posY = newY;
        }
      }

      // Draw Ceiling and Floor
      ctx.fillStyle = '#111'; // Ceiling
      ctx.fillRect(0, 0, canvas.width, canvas.height / 2);
      ctx.fillStyle = '#222'; // Floor
      ctx.fillRect(0, canvas.height / 2, canvas.width, canvas.height / 2);

      // Raycasting
      let startAngle = angle - HALF_FOV;

      for (let i = 0; i < NUM_RAYS; i++) {
        const rayAngle = startAngle + i * STEP_ANGLE;
        const cosRay = Math.cos(rayAngle);
        const sinRay = Math.sin(rayAngle);

        let distance = 0;
        let hitWall = false;

        while (!hitWall && distance < MAX_DEPTH) {
          distance += 1.5;
          const rayX = posX + cosRay * distance;
          const rayY = posY + sinRay * distance;

          const mapX = Math.floor(rayX / TILE_SIZE);
          const mapY = Math.floor(rayY / TILE_SIZE);

          if (mapX < 0 || mapX >= MAP_WIDTH || mapY < 0 || mapY >= MAP_HEIGHT) {
            hitWall = true;
            distance = MAX_DEPTH;
          } else if (map[mapY][mapX] > 0) {
            hitWall = true;
          }
        }

        // Correct fish-eye effect
        const correctedDist = distance * Math.cos(rayAngle - angle);

        // Wall height
        const wallHeight = Math.min(
          canvas.height,
          (TILE_SIZE * canvas.height) / (correctedDist || 1)
        );

        // Wall colors based on depth (shading)
        const intensity = Math.max(0, 1 - distance / MAX_DEPTH);
        const red = Math.floor(190 * intensity + 20);
        const green = Math.floor(30 * intensity + 5);
        const blue = Math.floor(30 * intensity + 5);

        ctx.fillStyle = `rgb(${red}, ${green}, ${blue})`;
        ctx.fillRect(i, (canvas.height - wallHeight) / 2, 1, wallHeight);
      }

      // Draw minimap in the corner
      const minimapScale = 0.15;
      const mmTileSize = TILE_SIZE * minimapScale;
      ctx.fillStyle = 'rgba(0, 0, 0, 0.6)';
      ctx.fillRect(10, 10, MAP_WIDTH * mmTileSize, MAP_HEIGHT * mmTileSize);

      for (let y = 0; y < MAP_HEIGHT; y++) {
        for (let x = 0; x < MAP_WIDTH; x++) {
          if (map[y][x] > 0) {
            ctx.fillStyle = '#555';
            ctx.fillRect(10 + x * mmTileSize, 10 + y * mmTileSize, mmTileSize - 1, mmTileSize - 1);
          }
        }
      }

      // Draw player on minimap
      ctx.fillStyle = '#f39c12';
      ctx.beginPath();
      ctx.arc(10 + posX * minimapScale, 10 + posY * minimapScale, 3, 0, Math.PI * 2);
      ctx.fill();

      // Draw player direction on minimap
      ctx.strokeStyle = '#f39c12';
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(10 + posX * minimapScale, 10 + posY * minimapScale);
      ctx.lineTo(
        10 + posX * minimapScale + Math.cos(angle) * 8,
        10 + posY * minimapScale + Math.sin(angle) * 8
      );
      ctx.stroke();

      // Draw HUD
      ctx.fillStyle = '#fff';
      ctx.font = '16px monospace';
      ctx.textAlign = 'left';
      ctx.fillText('SYSTEM: HANG3D NIGHTMARE (3D RAYCASTER)', 20, canvas.height - 70);
      ctx.fillText(`ASSETS: ${gameDataText.substring(0, 24)}...`, 20, canvas.height - 50);
      ctx.fillText(
        'CONTROLS: W/S TO MOVE. A/D OR ARROWS TO ROTATE CAMERA.',
        20,
        canvas.height - 30
      );

      requestAnimationFrame(loop);
    }

    loop();
  }
})();
