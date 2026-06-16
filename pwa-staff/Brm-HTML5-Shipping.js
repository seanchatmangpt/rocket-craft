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
    fetchWithProgress('Brm-HTML5-Shipping.wasm', 'wasm'),
    fetchWithProgress('Brm-HTML5-Shipping.data', 'data'),
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
    if (loadingOverlay) loadingOverlay.style.display = 'none';
    if (canvas) canvas.style.display = 'block';

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    let carX = 100;
    let carY = 300;
    let speed = 4;
    let obstacles = [
      { x: 400, y: 300, width: 30, height: 40, passed: false },
      { x: 700, y: 300, width: 30, height: 40, passed: false },
    ];
    let score = 0;
    let isGameOver = false;

    // Game loop
    function loop() {
      if (isGameOver) {
        ctx.fillStyle = 'rgba(0, 0, 0, 0.7)';
        ctx.fillRect(0, 0, canvas.width, canvas.height);

        ctx.fillStyle = '#f39c12';
        ctx.font = 'bold 36px Arial';
        ctx.textAlign = 'center';
        ctx.fillText('CRASHED!', canvas.width / 2, canvas.height / 2 - 20);

        ctx.fillStyle = '#eee';
        ctx.font = '20px Arial';
        ctx.fillText(
          `Final Score: ${score} - Press SPACE to restart`,
          canvas.width / 2,
          canvas.height / 2 + 20
        );
        return;
      }

      // Physics / Updates
      carX += speed;
      if (carX > canvas.width - 150) {
        carX = 50;
        obstacles.forEach((o) => {
          o.x = canvas.width + Math.random() * 300;
          o.passed = false;
        });
      }

      // Move obstacles
      obstacles.forEach((o) => {
        if (carX > o.x && !o.passed) {
          o.passed = true;
          score += 100;
        }
      });

      // Collision Check
      obstacles.forEach((o) => {
        if (carX + 60 > o.x && carX < o.x + o.width && carY + 30 > o.y && carY < o.y + o.height) {
          isGameOver = true;
        }
      });

      // Render
      ctx.fillStyle = '#222';
      ctx.fillRect(0, 0, canvas.width, canvas.height);

      // Draw Horizon/Sky
      ctx.fillStyle = '#111';
      ctx.fillRect(0, 0, canvas.width, 220);

      // Draw Stars/Grid
      ctx.strokeStyle = '#333';
      ctx.lineWidth = 1;
      for (let i = 0; i < canvas.width; i += 40) {
        ctx.beginPath();
        ctx.moveTo(i, 220);
        ctx.lineTo(i - 100, canvas.height);
        ctx.stroke();
      }

      // Draw Road
      ctx.fillStyle = '#333';
      ctx.fillRect(0, 220, canvas.width, canvas.height - 220);
      ctx.fillStyle = '#f39c12';
      ctx.fillRect(0, 218, canvas.width, 4);

      // Draw Car (Barbarian Road Mashine)
      ctx.fillStyle = '#e74c3c';
      ctx.fillRect(carX, carY, 60, 20);
      ctx.fillStyle = '#f39c12';
      ctx.fillRect(carX + 45, carY - 10, 15, 10);
      ctx.fillStyle = '#000';
      ctx.beginPath();
      ctx.arc(carX + 15, carY + 20, 8, 0, Math.PI * 2);
      ctx.arc(carX + 45, carY + 20, 8, 0, Math.PI * 2);
      ctx.fill();

      // Draw Obstacles
      ctx.fillStyle = '#95a5a6';
      obstacles.forEach((o) => {
        ctx.fillRect(o.x, o.y, o.width, o.height);
        ctx.fillStyle = '#7f8c8d';
        ctx.fillRect(o.x + 5, o.y + 5, o.width - 10, o.height - 10);
      });

      // Draw UI HUD
      ctx.fillStyle = '#fff';
      ctx.font = '16px monospace';
      ctx.textAlign = 'left';
      ctx.fillText(`SYSTEM: BRM ENGINE (WASM CLIENT)`, 20, 30);
      ctx.fillText(`ASSETS: ${gameDataText.substring(0, 24)}...`, 20, 50);
      ctx.fillText(`SCORE: ${score}`, 20, 70);

      requestAnimationFrame(loop);
    }

    // Keyboard handlers
    window.addEventListener('keydown', (e) => {
      if (e.code === 'Space') {
        if (isGameOver) {
          isGameOver = false;
          score = 0;
          carX = 100;
          obstacles = [
            { x: 400, y: 300, width: 30, height: 40, passed: false },
            { x: 700, y: 300, width: 30, height: 40, passed: false },
          ];
          loop();
        } else {
          // Jump simulation
          if (carY === 300) {
            let jumpHeight = 0;
            let jumpingUp = true;
            function jump() {
              if (jumpingUp) {
                carY -= 5;
                jumpHeight += 5;
                if (jumpHeight >= 80) jumpingUp = false;
              } else {
                carY += 5;
                jumpHeight -= 5;
                if (jumpHeight <= 0) {
                  carY = 300;
                  return;
                }
              }
              setTimeout(jump, 16);
            }
            jump();
          }
        }
      }
    });

    loop();
  }
})();
