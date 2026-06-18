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
    fetchWithProgress('SurvivalGame-HTML5-Shipping.wasm', 'wasm'),
    fetchWithProgress('SurvivalGame-HTML5-Shipping.data', 'data'),
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

    let playerX = canvas.width / 2;
    let playerY = canvas.height / 2;
    let playerSpeed = 3;
    let playerRadius = 15;

    let keys = {};
    let bullets = [];
    let zombies = [];
    let score = 0;
    let isGameOver = false;
    let frameCount = 0;

    // Game loop
    function loop() {
      if (isGameOver) {
        ctx.fillStyle = 'rgba(0, 0, 0, 0.8)';
        ctx.fillRect(0, 0, canvas.width, canvas.height);

        ctx.fillStyle = '#e74c3c';
        ctx.font = 'bold 36px Arial';
        ctx.textAlign = 'center';
        ctx.fillText('YOU DIED!', canvas.width / 2, canvas.height / 2 - 20);

        ctx.fillStyle = '#eee';
        ctx.font = '20px Arial';
        ctx.fillText(
          `Zombies Survived: ${score / 100} - Final Score: ${score}`,
          canvas.width / 2,
          canvas.height / 2 + 20
        );
        ctx.font = '16px Arial';
        ctx.fillText('Press SPACE to restart', canvas.width / 2, canvas.height / 2 + 60);
        return;
      }

      frameCount++;

      // Player Movement
      if (keys['KeyW'] || keys['ArrowUp']) playerY = Math.max(playerRadius, playerY - playerSpeed);
      if (keys['KeyS'] || keys['ArrowDown'])
        playerY = Math.min(canvas.height - playerRadius, playerY + playerSpeed);
      if (keys['KeyA'] || keys['ArrowLeft'])
        playerX = Math.max(playerRadius, playerX - playerSpeed);
      if (keys['KeyD'] || keys['ArrowRight'])
        playerX = Math.min(canvas.width - playerRadius, playerX + playerSpeed);

      // Bullet Spawn / Shoot logic
      if (keys['Space'] && frameCount % 12 === 0) {
        bullets.push({
          x: playerX,
          y: playerY,
          vx: 0,
          vy: -6,
        });
      }

      // Update Bullets
      bullets.forEach((b, i) => {
        b.x += b.vx;
        b.y += b.vy;
        if (b.y < 0) {
          bullets.splice(i, 1);
        }
      });

      // Spawn Zombies
      if (frameCount % 60 === 0) {
        const side = Math.floor(Math.random() * 4);
        let zx, zy;
        if (side === 0) {
          zx = Math.random() * canvas.width;
          zy = -20;
        } else if (side === 1) {
          zx = canvas.width + 20;
          zy = Math.random() * canvas.height;
        } else if (side === 2) {
          zx = Math.random() * canvas.width;
          zy = canvas.height + 20;
        } else {
          zx = -20;
          zy = Math.random() * canvas.height;
        }

        zombies.push({
          x: zx,
          y: zy,
          speed: 1.2 + Math.random() * 0.8,
          radius: 12,
        });
      }

      // Update Zombies
      zombies.forEach((z, i) => {
        const dx = playerX - z.x;
        const dy = playerY - z.y;
        const dist = Math.hypot(dx, dy);

        if (dist > 0) {
          z.x += (dx / dist) * z.speed;
          z.y += (dy / dist) * z.speed;
        }

        // Collision: Zombie - Player
        if (dist < playerRadius + z.radius) {
          isGameOver = true;
        }

        // Collision: Zombie - Bullets
        bullets.forEach((b, bi) => {
          const bdist = Math.hypot(b.x - z.x, b.y - z.y);
          if (bdist < z.radius + 4) {
            zombies.splice(i, 1);
            bullets.splice(bi, 1);
            score += 100;
          }
        });
      });

      // Clear & Draw
      ctx.fillStyle = '#111';
      ctx.fillRect(0, 0, canvas.width, canvas.height);

      // Draw grid lines
      ctx.strokeStyle = '#222';
      ctx.lineWidth = 1;
      for (let i = 0; i < canvas.width; i += 40) {
        ctx.beginPath();
        ctx.moveTo(i, 0);
        ctx.lineTo(i, canvas.height);
        ctx.stroke();
      }
      for (let i = 0; i < canvas.height; i += 40) {
        ctx.beginPath();
        ctx.moveTo(0, i);
        ctx.lineTo(canvas.width, i);
        ctx.stroke();
      }

      // Draw Player (Green marine)
      ctx.fillStyle = '#2ecc71';
      ctx.beginPath();
      ctx.arc(playerX, playerY, playerRadius, 0, Math.PI * 2);
      ctx.fill();

      // Draw gun barrel
      ctx.fillStyle = '#27ae60';
      ctx.fillRect(playerX - 4, playerY - playerRadius - 8, 8, playerRadius);

      // Draw Bullets
      ctx.fillStyle = '#f1c40f';
      bullets.forEach((b) => {
        ctx.beginPath();
        ctx.arc(b.x, b.y, 4, 0, Math.PI * 2);
        ctx.fill();
      });

      // Draw Zombies
      ctx.fillStyle = '#e74c3c';
      zombies.forEach((z) => {
        ctx.beginPath();
        ctx.arc(z.x, z.y, z.radius, 0, Math.PI * 2);
        ctx.fill();
        // Eyes
        ctx.fillStyle = '#fff';
        ctx.beginPath();
        ctx.arc(z.x - 4, z.y - 3, 2, 0, Math.PI * 2);
        ctx.arc(z.x + 4, z.y - 3, 2, 0, Math.PI * 2);
        ctx.fill();
        ctx.fillStyle = '#e74c3c';
      });

      // Draw UI / HUD
      ctx.fillStyle = '#fff';
      ctx.font = '16px monospace';
      ctx.textAlign = 'left';
      ctx.fillText('SYSTEM: SURVIVAL GAME (WASM CLIENT)', 20, 30);
      ctx.fillText(`ASSETS: ${gameDataText.substring(0, 24)}...`, 20, 50);
      ctx.fillText(`SCORE: ${score}`, 20, 70);

      requestAnimationFrame(loop);
    }

    // Keyboard handlers
    window.addEventListener('keydown', (e) => {
      keys[e.code] = true;

      if (e.code === 'Space' && isGameOver) {
        isGameOver = false;
        score = 0;
        playerX = canvas.width / 2;
        playerY = canvas.height / 2;
        bullets = [];
        zombies = [];
        frameCount = 0;
        loop();
      }
    });

    window.addEventListener('keyup', (e) => {
      keys[e.code] = false;
    });

    loop();
  }
})();
