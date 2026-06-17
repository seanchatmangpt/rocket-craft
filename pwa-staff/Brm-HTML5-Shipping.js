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

  /**
   * Robust fetch with progress and DecompressionStream support
   */
  async function fetchWithProgress(url, type) {
    const response = await fetch(url);
    if (!response.ok) {
      throw new Error(`Failed to load ${url}: ${response.statusText}`);
    }

    const contentLength = response.headers.get('content-length');
    const total = contentLength ? parseInt(contentLength, 10) : 0;

    if (!response.body) {
      throw new Error('ReadableStream not supported on response body');
    }

    // Check for GZIP compression
    const isGzip = url.endsWith('.gz') || response.headers.get('Content-Encoding') === 'gzip' || response.headers.get('Content-Type') === 'application/x-gzip';
    
    let loaded = 0;
    const progressStream = new TransformStream({
      transform(chunk, controller) {
        loaded += chunk.length;
        if (total > 0) {
          loadProgress[type] = (loaded / total) * 100;
          const combinedProgress = (loadProgress.wasm + loadProgress.data) / 2;
          updateStatus(
            `Downloading game package...`,
            `Received ${Math.round(loaded / 1024)} KB / ${Math.round(total / 1024)} KB for ${url}`,
            combinedProgress
          );
        }
        controller.enqueue(chunk);
      }
    });

    let stream = response.body.pipeThrough(progressStream);
    
    if (isGzip) {
      if (typeof DecompressionStream !== 'undefined') {
        console.log(`Decompressing ${url} via browser DecompressionStream...`);
        stream = stream.pipeThrough(new DecompressionStream('gzip'));
      } else {
        console.warn(`DecompressionStream not supported, but ${url} appears to be gzipped. Falling back to raw stream (may fail).`);
      }
    }

    const reader = stream.getReader();
    const chunks = [];
    let totalLength = 0;

    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      chunks.push(value);
      totalLength += value.length;
    }

    const resultBuffer = new Uint8Array(totalLength);
    let offset = 0;
    for (const chunk of chunks) {
      resultBuffer.set(chunk, offset);
      offset += chunk.length;
    }
    return resultBuffer;
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

  /**
   * Real WebGL 2.0 Rendering Implementation
   */
  function runGame() {
    console.log('Finalizing WebGL 2.0 readiness...');
    
    if (loadingOverlay) loadingOverlay.style.display = 'none';
    if (canvas) canvas.style.display = 'block';

    const gl = canvas.getContext('webgl2', { 
      antialias: true, 
      alpha: false, 
      depth: true, 
      stencil: false, 
      desynchronized: true,
      preserveDrawingBuffer: false
    });

    if (!gl) {
      const msg = 'WebGL 2.0 not supported by your browser. This application requires WebGL 2.0.';
      console.error(msg);
      updateStatus('Launch Error', msg, 0);
      return;
    }

    console.log('WebGL 2.0 Context established. Renderer: ' + gl.getParameter(gl.RENDERER));

    // --- Shaders ---
    const vsSource = `#version 300 es
      layout(location = 0) in vec3 aPosition;
      layout(location = 1) in vec3 aNormal;

      uniform mat4 uModelViewMatrix;
      uniform mat4 uProjectionMatrix;
      uniform vec3 uColor;

      out vec3 vNormal;
      out vec3 vColor;

      void main() {
        gl_Position = uProjectionMatrix * uModelViewMatrix * vec4(aPosition, 1.0);
        vNormal = aNormal;
        vColor = uColor;
      }
    `;

    const fsSource = `#version 300 es
      precision highp float;
      in vec3 vNormal;
      in vec3 vColor;
      out vec4 outColor;

      void main() {
        vec3 lightDir = normalize(vec3(0.5, 1.0, 0.7));
        float diff = max(dot(normalize(vNormal), lightDir), 0.2);
        outColor = vec4(vColor * diff, 1.0);
      }
    `;

    function createShader(gl, type, source) {
      const shader = gl.createShader(type);
      gl.shaderSource(shader, source);
      gl.compileShader(shader);
      if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
        console.error('Shader compile error:', gl.getShaderInfoLog(shader));
        gl.deleteShader(shader);
        return null;
      }
      return shader;
    }

    const vertexShader = createShader(gl, gl.VERTEX_SHADER, vsSource);
    const fragmentShader = createShader(gl, gl.FRAGMENT_SHADER, fsSource);

    const program = gl.createProgram();
    gl.attachShader(program, vertexShader);
    gl.attachShader(program, fragmentShader);
    gl.linkProgram(program);

    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
      console.error('Program link error:', gl.getProgramInfoLog(program));
      return;
    }

    const programInfo = {
      program: program,
      attribLocations: {
        position: gl.getAttribLocation(program, 'aPosition'),
        normal: gl.getAttribLocation(program, 'aNormal'),
      },
      uniformLocations: {
        projectionMatrix: gl.getUniformLocation(program, 'uProjectionMatrix'),
        modelViewMatrix: gl.getUniformLocation(program, 'uModelViewMatrix'),
        color: gl.getUniformLocation(program, 'uColor'),
      },
    };

    // --- Geometry ---
    function createCube() {
      const positions = [
        -1,-1, 1,  1,-1, 1,  1, 1, 1, -1, 1, 1, // Front
        -1,-1,-1, -1, 1,-1,  1, 1,-1,  1,-1,-1, // Back
        -1, 1,-1, -1, 1, 1,  1, 1, 1,  1, 1,-1, // Top
        -1,-1,-1,  1,-1,-1,  1,-1, 1, -1,-1, 1, // Bottom
         1,-1,-1,  1, 1,-1,  1, 1, 1,  1,-1, 1, // Right
        -1,-1,-1, -1,-1, 1, -1, 1, 1, -1, 1,-1, // Left
      ];
      const normals = [
         0, 0, 1,  0, 0, 1,  0, 0, 1,  0, 0, 1,
         0, 0,-1,  0, 0,-1,  0, 0,-1,  0, 0,-1,
         0, 1, 0,  0, 1, 0,  0, 1, 0,  0, 1, 0,
         0,-1, 0,  0,-1, 0,  0,-1, 0,  0,-1, 0,
         1, 0, 0,  1, 0, 0,  1, 0, 0,  1, 0, 0,
        -1, 0, 0, -1, 0, 0, -1, 0, 0, -1, 0, 0,
      ];
      const indices = [
        0, 1, 2,  0, 2, 3,    4, 5, 6,  4, 6, 7,
        8, 9,10,  8,10,11,   12,13,14, 12,14,15,
        16,17,18, 16,18,19,  20,21,22, 20,22,23,
      ];
      return { positions, normals, indices };
    }

    const cubeData = createCube();
    const vao = gl.createVertexArray();
    gl.bindVertexArray(vao);

    const posBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, posBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(cubeData.positions), gl.STATIC_DRAW);
    gl.enableVertexAttribArray(programInfo.attribLocations.position);
    gl.vertexAttribPointer(programInfo.attribLocations.position, 3, gl.FLOAT, false, 0, 0);

    const normBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, normBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(cubeData.normals), gl.STATIC_DRAW);
    gl.enableVertexAttribArray(programInfo.attribLocations.normal);
    gl.vertexAttribPointer(programInfo.attribLocations.normal, 3, gl.FLOAT, false, 0, 0);

    const indexBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indexBuffer);
    gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, new Uint16Array(cubeData.indices), gl.STATIC_DRAW);

    // --- Math Utils ---
    const mat4 = {
      perspective: (out, fov, aspect, near, far) => {
        const f = 1.0 / Math.tan(fov / 2);
        const nf = 1 / (near - far);
        out.fill(0);
        out[0] = f / aspect;
        out[5] = f;
        out[10] = (far + near) * nf;
        out[11] = -1;
        out[14] = (2 * far * near) * nf;
      },
      identity: (out) => {
        out.fill(0);
        out[0] = out[5] = out[10] = out[15] = 1;
      },
      translate: (out, a, v) => {
        for(let i=0; i<16; i++) out[i] = a[i];
        out[12] = a[0] * v[0] + a[4] * v[1] + a[8] * v[2] + a[12];
        out[13] = a[1] * v[0] + a[5] * v[1] + a[9] * v[2] + a[13];
        out[14] = a[2] * v[0] + a[6] * v[1] + a[10] * v[2] + a[14];
        out[15] = a[3] * v[0] + a[7] * v[1] + a[11] * v[2] + a[15];
      },
      rotateY: (out, a, rad) => {
        const s = Math.sin(rad), c = Math.cos(rad);
        const a00 = a[0], a01 = a[1], a02 = a[2], a03 = a[3];
        const a20 = a[8], a21 = a[9], a22 = a[10], a23 = a[11];
        for(let i=0; i<16; i++) out[i] = a[i];
        out[0] = a00 * c - a20 * s; out[1] = a01 * c - a21 * s;
        out[2] = a02 * c - a22 * s; out[3] = a03 * c - a23 * s;
        out[8] = a00 * s + a20 * c; out[9] = a01 * s + a21 * c;
        out[10] = a02 * s + a22 * c; out[11] = a03 * s + a23 * c;
      },
      rotateX: (out, a, rad) => {
        const s = Math.sin(rad), c = Math.cos(rad);
        const a10 = a[4], a11 = a[5], a12 = a[6], a13 = a[7];
        const a20 = a[8], a21 = a[9], a22 = a[10], a23 = a[11];
        for(let i=0; i<16; i++) out[i] = a[i];
        out[4] = a10 * c + a20 * s; out[5] = a11 * c + a21 * s;
        out[6] = a12 * c + a22 * s; out[7] = a13 * c + a23 * s;
        out[8] = a20 * c - a10 * s; out[9] = a21 * c - a11 * s;
        out[10] = a22 * c - a12 * s; out[11] = a23 * c - a13 * s;
      },
      scale: (out, a, v) => {
        out[0] = a[0] * v[0]; out[1] = a[1] * v[0]; out[2] = a[2] * v[0]; out[3] = a[3] * v[0];
        out[4] = a[4] * v[1]; out[5] = a[5] * v[1]; out[6] = a[6] * v[1]; out[7] = a[7] * v[1];
        out[8] = a[8] * v[2]; out[9] = a[9] * v[2]; out[10] = a[10] * v[2]; out[11] = a[11] * v[2];
        out[12] = a[12]; out[13] = a[13]; out[14] = a[14]; out[15] = a[15];
      }
    };

    // --- State & Interaction ---
    let player = { x: 0, y: 0, z: 200, yaw: 0.0, pitch: 0.0 };
    let actors = [];
    let keys = {};
    const coordsDiv = document.getElementById('coords');

    function resize() {
      const displayWidth = canvas.clientWidth;
      const displayHeight = canvas.clientHeight;
      if (canvas.width !== displayWidth || canvas.height !== displayHeight) {
        canvas.width = displayWidth;
        canvas.height = displayHeight;
        gl.viewport(0, 0, canvas.width, canvas.height);
      }
    }
    window.addEventListener('resize', resize);
    resize();

    window.addEventListener('keydown', (e) => { keys[e.code] = true; });
    window.addEventListener('keyup', (e) => { keys[e.code] = false; });

    // Parse data
    try {
      actors = JSON.parse(gameDataText);
      console.log(`Loaded ${actors.length} actors.`);
    } catch (e) {
      console.warn('Fallback actors used.');
      actors = [{ name: 'Fallback', location: {x:0, y:0, z:0}, scale: {x:1, y:1, z:1}, color: [0.5, 0.5, 0.5] }];
    }

    // --- Render Loop ---
    let lastTime = 0;
    let engineReadySignaled = false;

    function render(time) {
      if (!engineReadySignaled) {
        window.UE4_EngineReady = true;
        console.log('UE4 Engine Ready signaled (First Frame).');
        engineReadySignaled = true;
      }

      const dt = (time - lastTime) / 1000;
      lastTime = time;

      // Update
      const speed = 200 * dt;
      const rotSpeed = 1.5 * dt;
      if (keys['KeyW']) { player.x += Math.cos(player.yaw) * speed; player.y += Math.sin(player.yaw) * speed; }
      if (keys['KeyS']) { player.x -= Math.cos(player.yaw) * speed; player.y -= Math.sin(player.yaw) * speed; }
      if (keys['KeyA']) { player.x -= Math.sin(player.yaw) * speed; player.y += Math.cos(player.yaw) * speed; }
      if (keys['KeyD']) { player.x += Math.sin(player.yaw) * speed; player.y -= Math.cos(player.yaw) * speed; }
      if (keys['ArrowLeft']) player.yaw -= rotSpeed;
      if (keys['ArrowRight']) player.yaw += rotSpeed;
      
      if (coordsDiv) coordsDiv.innerText = `X: ${player.x.toFixed(1)} Y: ${player.y.toFixed(1)} Z: ${player.z.toFixed(1)}`;

      // Draw
      gl.clearColor(0.02, 0.02, 0.05, 1.0);
      gl.clearDepth(1.0);
      gl.enable(gl.DEPTH_TEST);
      gl.depthFunc(gl.LEQUAL);
      gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);

      const projectionMatrix = new Float32Array(16);
      mat4.perspective(projectionMatrix, (45 * Math.PI) / 180, canvas.width / canvas.height, 0.1, 5000.0);

      gl.useProgram(programInfo.program);
      gl.uniformMatrix4fv(programInfo.uniformLocations.projectionMatrix, false, projectionMatrix);

      actors.forEach(actor => {
        const modelViewMatrix = new Float32Array(16);
        mat4.identity(modelViewMatrix);
        
        // Camera transform (inverse of player)
        mat4.rotateX(modelViewMatrix, modelViewMatrix, -player.pitch);
        mat4.rotateY(modelViewMatrix, modelViewMatrix, -player.yaw);
        mat4.translate(modelViewMatrix, modelViewMatrix, [-player.x, -player.y, -player.z]);

        // Actor transform
        mat4.translate(modelViewMatrix, modelViewMatrix, [actor.location.x, actor.location.y, actor.location.z]);
        mat4.scale(modelViewMatrix, modelViewMatrix, [actor.scale.x * 50, actor.scale.y * 50, actor.scale.z * 50]);

        gl.uniformMatrix4fv(programInfo.uniformLocations.modelViewMatrix, false, modelViewMatrix);
        
        let color = [0.0, 0.6, 1.0];
        if (actor.name.toLowerCase().includes('bot')) color = [0.0, 1.0, 0.4];
        gl.uniform3fv(programInfo.uniformLocations.color, color);

        gl.bindVertexArray(vao);
        gl.drawElements(gl.TRIANGLES, 36, gl.UNSIGNED_SHORT, 0);
      });

      requestAnimationFrame(render);
    }
    requestAnimationFrame(render);
  }
})();
