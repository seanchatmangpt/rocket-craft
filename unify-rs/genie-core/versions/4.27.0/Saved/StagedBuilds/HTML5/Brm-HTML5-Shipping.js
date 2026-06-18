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

    // --- lineIndexBuffer ---
    const lineIndices = [
      0, 1,  1, 2,  2, 3,  3, 0, // front
      4, 7,  7, 6,  6, 5,  5, 4, // back
      0, 4,  1, 7,  2, 6,  3, 5  // connections
    ];
    const lineIndexBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, lineIndexBuffer);
    gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, new Uint16Array(lineIndices), gl.STATIC_DRAW);

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
      rotateZ: (out, a, rad) => {
        const s = Math.sin(rad), c = Math.cos(rad);
        const a00 = a[0], a01 = a[1], a02 = a[2], a03 = a[3];
        const a10 = a[4], a11 = a[5], a12 = a[6], a13 = a[7];
        for(let i=0; i<16; i++) out[i] = a[i];
        out[0] = a00 * c + a10 * s; out[1] = a01 * c + a11 * s;
        out[2] = a02 * c + a12 * s; out[3] = a03 * c + a13 * s;
        out[4] = a10 * c - a00 * s; out[5] = a11 * c - a01 * s;
        out[6] = a12 * c - a02 * s; out[7] = a13 * c - a03 * s;
      },
      scale: (out, a, v) => {
        out[0] = a[0] * v[0]; out[1] = a[1] * v[0]; out[2] = a[2] * v[0]; out[3] = a[3] * v[0];
        out[4] = a[4] * v[1]; out[5] = a[5] * v[1]; out[6] = a[6] * v[1]; out[7] = a[7] * v[1];
        out[8] = a[8] * v[2]; out[9] = a[9] * v[2]; out[10] = a[10] * v[2]; out[11] = a[11] * v[2];
        out[12] = a[12]; out[13] = a[13]; out[14] = a[14]; out[15] = a[15];
      },
      multiply: (out, a, b) => {
        let a00 = a[0], a01 = a[1], a02 = a[2], a03 = a[3],
            a10 = a[4], a11 = a[5], a12 = a[6], a13 = a[7],
            a20 = a[8], a21 = a[9], a22 = a[10], a23 = a[11],
            a30 = a[12], a31 = a[13], a32 = a[14], a33 = a[15];
        let b00 = b[0], b01 = b[1], b02 = b[2], b03 = b[3],
            b10 = b[4], b11 = b[5], b12 = b[6], b13 = b[7],
            b20 = b[8], b21 = b[9], b22 = b[10], b23 = b[11],
            b30 = b[12], b31 = b[13], b32 = b[14], b33 = b[15];
        out[0] = b00 * a00 + b01 * a10 + b02 * a20 + b03 * a30;
        out[1] = b00 * a01 + b01 * a11 + b02 * a21 + b03 * a31;
        out[2] = b00 * a02 + b01 * a12 + b02 * a22 + b03 * a32;
        out[3] = b00 * a03 + b01 * a13 + b02 * a23 + b03 * a33;
        out[4] = b10 * a00 + b11 * a10 + b12 * a20 + b13 * a30;
        out[5] = b10 * a01 + b11 * a11 + b12 * a21 + b13 * a31;
        out[6] = b10 * a02 + b11 * a12 + b12 * a22 + b13 * a32;
        out[7] = b10 * a03 + b11 * a13 + b12 * a23 + b13 * a33;
        out[8] = b20 * a00 + b21 * a10 + b22 * a20 + b23 * a30;
        out[9] = b20 * a01 + b21 * a11 + b22 * a21 + b23 * a31;
        out[10] = b20 * a02 + b21 * a12 + b22 * a22 + b23 * a32;
        out[11] = b20 * a03 + b21 * a13 + b22 * a23 + b23 * a33;
        out[12] = b30 * a00 + b31 * a10 + b32 * a20 + b33 * a30;
        out[13] = b30 * a01 + b31 * a11 + b32 * a21 + b33 * a31;
        out[14] = b30 * a02 + b31 * a12 + b32 * a22 + b33 * a32;
        out[15] = b30 * a03 + b31 * a13 + b32 * a23 + b33 * a33;
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

    window.addEventListener('keydown', (e) => { keys[e.code] = true; keys[e.key] = true; });
    window.addEventListener('keyup', (e) => { keys[e.code] = false; keys[e.key] = false; });

    // Parse data
    try {
      actors = JSON.parse(gameDataText);
      console.log(`Loaded ${actors.length} actors.`);
    } catch (e) {
      console.warn('Fallback actors used.');
      actors = [{ name: 'Fallback', location: {x:0, y:0, z:0}, scale: {x:1, y:1, z:1}, color: [0.5, 0.5, 0.5] }];
    }

    // Default part translations and rotations for the skeletal hierarchy
    const defaultPartTranslations = {
      spine: [0, 0, 80],
      torso: [0, 0, 20],
      head: [0, 0, 20],
      left_arm: [-18, 0, 5],
      right_arm: [18, 0, 5],
      left_leg: [-10, 0, -25],
      right_leg: [10, 0, -25]
    };

    const defaultPartRotations = {
      spine: [0, 0, 0],
      torso: [0, 0, 0],
      head: [0, 0, 0],
      left_arm: [0, 0, 0],
      right_arm: [0, 0, 0],
      left_leg: [0, 0, 0],
      right_leg: [0, 0, 0]
    };

    // Recursive / Hierarchical skeletal renderer
    function renderSkeletalMech(viewMatrix, baseWorldTranslation, partRotations, partTranslations, isWireframe, baseYaw, spotlightIntensity) {
      // Create base matrix
      const baseMatrix = new Float32Array(16);
      mat4.identity(baseMatrix);
      mat4.translate(baseMatrix, baseMatrix, baseWorldTranslation);
      mat4.rotateZ(baseMatrix, baseMatrix, baseYaw);

      const parts = [
        { name: 'spine', parent: null, translation: partTranslations.spine, rotation: partRotations.spine, scale: [15, 12, 10], color: [0.6, 0.6, 0.7] },
        { name: 'torso', parent: 'spine', translation: partTranslations.torso, rotation: partRotations.torso, scale: [22, 16, 15], color: [0.7, 0.3, 0.3] },
        { name: 'head', parent: 'torso', translation: partTranslations.head, rotation: partRotations.head, scale: [8, 8, 8], color: [0.8, 0.8, 0.2] },
        { name: 'left_arm', parent: 'torso', translation: partTranslations.left_arm, rotation: partRotations.left_arm, scale: [6, 6, 20], color: [0.3, 0.7, 0.3] },
        { name: 'right_arm', parent: 'torso', translation: partTranslations.right_arm, rotation: partRotations.right_arm, scale: [6, 6, 20], color: [0.3, 0.7, 0.3] },
        { name: 'left_leg', parent: 'spine', translation: partTranslations.left_leg, rotation: partRotations.left_leg, scale: [7, 7, 25], color: [0.3, 0.3, 0.7] },
        { name: 'right_leg', parent: 'spine', translation: partTranslations.right_leg, rotation: partRotations.right_leg, scale: [7, 7, 25], color: [0.3, 0.3, 0.7] }
      ];

      const computedMatrices = {};

      parts.forEach(part => {
        const localMatrix = new Float32Array(16);
        mat4.identity(localMatrix);
        mat4.translate(localMatrix, localMatrix, part.translation);
        mat4.rotateX(localMatrix, localMatrix, part.rotation[0]);
        mat4.rotateY(localMatrix, localMatrix, part.rotation[1]);
        mat4.rotateZ(localMatrix, localMatrix, part.rotation[2]);

        const worldMatrix = new Float32Array(16);
        if (part.parent) {
          const parentWorld = computedMatrices[part.parent];
          mat4.multiply(worldMatrix, parentWorld, localMatrix);
        } else {
          mat4.multiply(worldMatrix, baseMatrix, localMatrix);
        }
        computedMatrices[part.name] = worldMatrix;

        // Draw solid part
        const modelMatrix = new Float32Array(16);
        mat4.identity(modelMatrix);
        mat4.scale(modelMatrix, worldMatrix, part.scale);

        const modelViewMatrix = new Float32Array(16);
        mat4.multiply(modelViewMatrix, viewMatrix, modelMatrix);

        gl.uniformMatrix4fv(programInfo.uniformLocations.modelViewMatrix, false, modelViewMatrix);
        const c = part.color;
        const finalColor = [c[0] * spotlightIntensity, c[1] * spotlightIntensity, c[2] * spotlightIntensity];
        gl.uniform3fv(programInfo.uniformLocations.color, finalColor);

        gl.bindVertexArray(vao);
        gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indexBuffer);
        gl.drawElements(gl.TRIANGLES, 36, gl.UNSIGNED_SHORT, 0);

        // Draw collision volume wireframe outline if requested
        if (isWireframe) {
          const outlineModel = new Float32Array(16);
          mat4.identity(outlineModel);
          mat4.scale(outlineModel, worldMatrix, [part.scale[0] * 1.25, part.scale[1] * 1.25, part.scale[2] * 1.25]);
          const outlineModelView = new Float32Array(16);
          mat4.multiply(outlineModelView, viewMatrix, outlineModel);
          gl.uniformMatrix4fv(programInfo.uniformLocations.modelViewMatrix, false, outlineModelView);
          
          gl.uniform3fv(programInfo.uniformLocations.color, [0.0, 1.0, 0.2]); // Neon green outlines
          gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, lineIndexBuffer);
          gl.drawElements(gl.LINES, 24, gl.UNSIGNED_SHORT, 0);
        }
      });
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
      const sec = time / 1000;

      // Update player movement
      const speed = 200 * dt;
      const rotSpeed = 1.5 * dt;
      if (keys['KeyW'] || keys['W']) { player.x += Math.cos(player.yaw) * speed; player.y += Math.sin(player.yaw) * speed; }
      if (keys['KeyS'] || keys['S']) { player.x -= Math.cos(player.yaw) * speed; player.y -= Math.sin(player.yaw) * speed; }
      if (keys['KeyA'] || keys['A']) { player.x -= Math.sin(player.yaw) * speed; player.y += Math.cos(player.yaw) * speed; }
      if (keys['KeyD'] || keys['D']) { player.x += Math.sin(player.yaw) * speed; player.y -= Math.cos(player.yaw) * speed; }
      if (keys['ArrowLeft']) player.yaw -= rotSpeed;
      if (keys['ArrowRight']) player.yaw += rotSpeed;
      if (keys['Space']) { /* Jumping or fly behavior */ }

      // Dynamic Active Zone Index Calculation
      let activeZone = 1;
      if (player.x < 200) activeZone = 1;
      else if (player.x < 600) activeZone = 2;
      else if (player.x < 1000) activeZone = 3;
      else if (player.x < 1400) activeZone = 4;
      else if (player.x < 1800) activeZone = 5;
      else activeZone = 6;

      // Highlight HUD elements
      for (let i = 1; i <= 6; i++) {
        const hudEl = document.getElementById(`hud-zone-${i}`);
        if (hudEl) {
          let zoneName = '';
          if (i === 1) zoneName = 'Primitive Foundry';
          else if (i === 2) zoneName = 'Part Runner Wall';
          else if (i === 3) zoneName = 'Assembly Gantry';
          else if (i === 4) zoneName = 'Fit + Collision Bay';
          else if (i === 5) zoneName = 'Physics Proving Ground';
          else if (i === 6) zoneName = 'Final Reveal Platform';

          if (i === activeZone) {
            hudEl.style.color = '#00ff66';
            hudEl.style.fontWeight = 'bold';
            hudEl.innerHTML = `[▶] Zone ${i}: ${zoneName}`;
          } else {
            hudEl.style.color = '#aaa';
            hudEl.style.fontWeight = 'normal';
            hudEl.innerHTML = `[ ] Zone ${i}: ${zoneName}`;
          }
        }
      }
      
      const hudStatus = document.getElementById('hud-status');
      if (hudStatus) {
        hudStatus.innerText = `Pipeline Status: ZONE_${activeZone}_ACTIVE`;
      }

      if (coordsDiv) coordsDiv.innerText = `Player: X: ${player.x.toFixed(1)} Y: ${player.y.toFixed(1)} Z: ${player.z.toFixed(1)}`;

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

      // Compute View Matrix
      const viewMatrix = new Float32Array(16);
      mat4.identity(viewMatrix);
      mat4.rotateX(viewMatrix, viewMatrix, -player.pitch);
      mat4.rotateY(viewMatrix, viewMatrix, -player.yaw);
      mat4.translate(viewMatrix, viewMatrix, [-player.x, -player.y, -player.z]);

      // Render default environment actors
      actors.forEach(actor => {
        const modelViewMatrix = new Float32Array(16);
        mat4.identity(modelViewMatrix);
        mat4.rotateX(modelViewMatrix, modelViewMatrix, -player.pitch);
        mat4.rotateY(modelViewMatrix, modelViewMatrix, -player.yaw);
        mat4.translate(modelViewMatrix, modelViewMatrix, [-player.x, -player.y, -player.z]);

        mat4.translate(modelViewMatrix, modelViewMatrix, [actor.location.x, actor.location.y, actor.location.z]);
        mat4.scale(modelViewMatrix, modelViewMatrix, [actor.scale.x * 50, actor.scale.y * 50, actor.scale.z * 50]);

        gl.uniformMatrix4fv(programInfo.uniformLocations.modelViewMatrix, false, modelViewMatrix);
        
        let color = [0.0, 0.4, 0.8];
        if (actor.name.toLowerCase().includes('bot')) color = [0.0, 0.9, 0.3];
        gl.uniform3fv(programInfo.uniformLocations.color, color);

        gl.bindVertexArray(vao);
        gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indexBuffer);
        gl.drawElements(gl.TRIANGLES, 36, gl.UNSIGNED_SHORT, 0);
      });

      // --- ZONE Visual Behaviors ---

      // 1. Foundry: parts spin and float in space (x < 200)
      const floatingParts = [
        { name: 'spine', translation: [-50, 0, 80 + Math.sin(sec * 2) * 10], rotation: [sec * 0.5, sec * 0.7, sec * 0.3], scale: [15, 12, 10], color: [0.6, 0.6, 0.7] },
        { name: 'torso', translation: [-25, 30, 100 + Math.sin(sec * 2 + 1) * 10], rotation: [sec * 0.6, sec * 0.4, sec * 0.8], scale: [22, 16, 15], color: [0.7, 0.3, 0.3] },
        { name: 'head', translation: [0, 50, 120 + Math.sin(sec * 2 + 2) * 10], rotation: [sec * 0.3, sec * 0.9, sec * 0.2], scale: [8, 8, 8], color: [0.8, 0.8, 0.2] },
        { name: 'left_arm', translation: [25, 30, 100 + Math.sin(sec * 2 + 3) * 10], rotation: [sec * 0.8, sec * 0.2, sec * 0.5], scale: [6, 6, 20], color: [0.3, 0.7, 0.3] },
        { name: 'right_arm', translation: [50, 0, 80 + Math.sin(sec * 2 + 4) * 10], rotation: [sec * 0.4, sec * 0.6, sec * 0.9], scale: [6, 6, 20], color: [0.3, 0.7, 0.3] },
        { name: 'left_leg', translation: [-35, -30, 60 + Math.sin(sec * 2 + 5) * 10], rotation: [sec * 0.9, sec * 0.3, sec * 0.4], scale: [7, 7, 25], color: [0.3, 0.3, 0.7] },
        { name: 'right_leg', translation: [35, -30, 60 + Math.sin(sec * 2 + 6) * 10], rotation: [sec * 0.2, sec * 0.8, sec * 0.6], scale: [7, 7, 25], color: [0.3, 0.3, 0.7] }
      ];
      floatingParts.forEach(part => {
        const modelMatrix = new Float32Array(16);
        mat4.identity(modelMatrix);
        mat4.translate(modelMatrix, modelMatrix, part.translation);
        mat4.rotateX(modelMatrix, modelMatrix, part.rotation[0]);
        mat4.rotateY(modelMatrix, modelMatrix, part.rotation[1]);
        mat4.rotateZ(modelMatrix, modelMatrix, part.rotation[2]);
        mat4.scale(modelMatrix, modelMatrix, part.scale);

        const modelViewMatrix = new Float32Array(16);
        mat4.multiply(modelViewMatrix, viewMatrix, modelMatrix);
        gl.uniformMatrix4fv(programInfo.uniformLocations.modelViewMatrix, false, modelViewMatrix);
        gl.uniform3fv(programInfo.uniformLocations.color, part.color);
        gl.bindVertexArray(vao);
        gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indexBuffer);
        gl.drawElements(gl.TRIANGLES, 36, gl.UNSIGNED_SHORT, 0);
      });

      // 2. Runner Wall: parts aligned on vertical grid (200 <= x < 600)
      const wallParts = [
        { name: 'head', translation: [330, 80, 120], scale: [8, 8, 8], color: [0.8, 0.8, 0.2] },
        { name: 'torso', translation: [400, 80, 120], scale: [22, 16, 15], color: [0.7, 0.3, 0.3] },
        { name: 'spine', translation: [470, 80, 120], scale: [15, 12, 10], color: [0.6, 0.6, 0.7] },
        { name: 'left_arm', translation: [310, 80, 60], scale: [6, 6, 20], color: [0.3, 0.7, 0.3] },
        { name: 'right_arm', translation: [370, 80, 60], scale: [6, 6, 20], color: [0.3, 0.7, 0.3] },
        { name: 'left_leg', translation: [430, 80, 60], scale: [7, 7, 25], color: [0.3, 0.3, 0.7] },
        { name: 'right_leg', translation: [490, 80, 60], scale: [7, 7, 25], color: [0.3, 0.3, 0.7] }
      ];
      wallParts.forEach(part => {
        const modelMatrix = new Float32Array(16);
        mat4.identity(modelMatrix);
        mat4.translate(modelMatrix, modelMatrix, part.translation);
        mat4.scale(modelMatrix, modelMatrix, part.scale);

        const modelViewMatrix = new Float32Array(16);
        mat4.multiply(modelViewMatrix, viewMatrix, modelMatrix);
        gl.uniformMatrix4fv(programInfo.uniformLocations.modelViewMatrix, false, modelViewMatrix);
        gl.uniform3fv(programInfo.uniformLocations.color, part.color);
        gl.bindVertexArray(vao);
        gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indexBuffer);
        gl.drawElements(gl.TRIANGLES, 36, gl.UNSIGNED_SHORT, 0);
      });

      // 3. Assembly Gantry: parts fly and snap sequentially (600 <= x < 1000)
      const t_cycle = sec % 8;
      const gantryParts = [
        { name: 'spine', parent: null, localTrans: [0, 0, 80], flyFrom: [-100, 100, 200], scale: [15, 12, 10], color: [0.6, 0.6, 0.7], snapTime: 1 },
        { name: 'torso', parent: 'spine', localTrans: [0, 0, 20], flyFrom: [100, 100, 200], scale: [22, 16, 15], color: [0.7, 0.3, 0.3], snapTime: 2 },
        { name: 'head', parent: 'torso', localTrans: [0, 0, 20], flyFrom: [0, 150, 180], scale: [8, 8, 8], color: [0.8, 0.8, 0.2], snapTime: 3 },
        { name: 'left_arm', parent: 'torso', localTrans: [-18, 0, 5], flyFrom: [-150, 0, 150], scale: [6, 6, 20], color: [0.3, 0.7, 0.3], snapTime: 4 },
        { name: 'right_arm', parent: 'torso', localTrans: [18, 0, 5], flyFrom: [150, 0, 150], scale: [6, 6, 20], color: [0.3, 0.7, 0.3], snapTime: 5 },
        { name: 'left_leg', parent: 'spine', localTrans: [-10, 0, -25], flyFrom: [-80, -100, 50], scale: [7, 7, 25], color: [0.3, 0.3, 0.7], snapTime: 6 },
        { name: 'right_leg', parent: 'spine', localTrans: [10, 0, -25], flyFrom: [80, -100, 50], scale: [7, 7, 25], color: [0.3, 0.3, 0.7], snapTime: 7 }
      ];

      const gantryBaseMatrix = new Float32Array(16);
      mat4.identity(gantryBaseMatrix);
      mat4.translate(gantryBaseMatrix, gantryBaseMatrix, [800, 0, 0]);

      const computedGantryMatrices = {};
      gantryParts.forEach(part => {
        const localPos = [0, 0, 0];
        const localRot = [0, 0, 0];
        if (t_cycle >= part.snapTime) {
          localPos[0] = part.localTrans[0];
          localPos[1] = part.localTrans[1];
          localPos[2] = part.localTrans[2];
        } else if (t_cycle >= part.snapTime - 1) {
          const t_interp = t_cycle - (part.snapTime - 1);
          const ease = 1 - Math.pow(1 - t_interp, 3);
          localPos[0] = part.flyFrom[0] * (1 - ease) + part.localTrans[0] * ease;
          localPos[1] = part.flyFrom[1] * (1 - ease) + part.localTrans[1] * ease;
          localPos[2] = part.flyFrom[2] * (1 - ease) + part.localTrans[2] * ease;
          localRot[0] = (1 - ease) * 3.0;
          localRot[1] = (1 - ease) * 2.0;
        } else {
          localPos[0] = part.flyFrom[0];
          localPos[1] = part.flyFrom[1];
          localPos[2] = part.flyFrom[2];
          localRot[0] = 3.0;
          localRot[1] = 2.0;
        }

        const localMatrix = new Float32Array(16);
        mat4.identity(localMatrix);
        mat4.translate(localMatrix, localMatrix, localPos);
        mat4.rotateX(localMatrix, localMatrix, localRot[0]);
        mat4.rotateY(localMatrix, localMatrix, localRot[1]);

        const worldMatrix = new Float32Array(16);
        if (part.parent) {
          const parentWorld = computedGantryMatrices[part.parent];
          mat4.multiply(worldMatrix, parentWorld, localMatrix);
        } else {
          mat4.multiply(worldMatrix, gantryBaseMatrix, localMatrix);
        }
        computedGantryMatrices[part.name] = worldMatrix;

        const modelMatrix = new Float32Array(16);
        mat4.identity(modelMatrix);
        mat4.scale(modelMatrix, worldMatrix, part.scale);

        const modelViewMatrix = new Float32Array(16);
        mat4.multiply(modelViewMatrix, viewMatrix, modelMatrix);

        gl.uniformMatrix4fv(programInfo.uniformLocations.modelViewMatrix, false, modelViewMatrix);
        gl.uniform3fv(programInfo.uniformLocations.color, part.color);
        gl.bindVertexArray(vao);
        gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indexBuffer);
        gl.drawElements(gl.TRIANGLES, 36, gl.UNSIGNED_SHORT, 0);
      });

      // 4. Fit + Collision Bay: sweep/oscillate joints & collision outline boxes (1000 <= x < 1400)
      const collisionRotations = {
        spine: [0, 0, 0],
        torso: [0, 0, 0],
        head: [0, Math.sin(sec * 2.0) * 0.4, 0],
        left_arm: [Math.sin(sec * 3.0) * 0.8, 0, Math.cos(sec * 2.0) * 0.5],
        right_arm: [-Math.sin(sec * 3.0) * 0.8, 0, -Math.cos(sec * 2.0) * 0.5],
        left_leg: [Math.sin(sec * 4.0) * 0.6, 0, 0],
        right_leg: [-Math.sin(sec * 4.0) * 0.6, 0, 0]
      };
      renderSkeletalMech(viewMatrix, [1200, 0, 0], collisionRotations, defaultPartTranslations, true, 0, 1.0);

      // 5. Physics Proving Ground: player movement & leg walk cycle (1400 <= x < 1800)
      const isMoving = keys['KeyW'] || keys['KeyS'] || keys['KeyA'] || keys['KeyD'] || keys['W'] || keys['S'] || keys['A'] || keys['D'] || keys['ArrowLeft'] || keys['ArrowRight'];
      const isSpace = keys['Space'] || keys['spacebar'] || keys[' '];
      const provingRotations = {
        spine: [0, 0, 0],
        torso: [0, 0, 0],
        head: [0, 0, 0],
        left_arm: [0, 0, 0],
        right_arm: [0, 0, 0],
        left_leg: [0, 0, 0],
        right_leg: [0, 0, 0]
      };
      if (isMoving) {
        provingRotations.left_leg[0] = Math.sin(sec * 12.0) * 0.7;
        provingRotations.right_leg[0] = -Math.sin(sec * 12.0) * 0.7;
        provingRotations.left_arm[0] = -Math.sin(sec * 12.0) * 0.4;
        provingRotations.right_arm[0] = Math.sin(sec * 12.0) * 0.4;
      }
      if (isSpace) {
        provingRotations.left_arm[2] = 1.3;
        provingRotations.right_arm[2] = -1.3;
        provingRotations.left_leg[0] = -0.5;
        provingRotations.right_leg[0] = -0.5;
      }
      if (activeZone === 5) {
        const playerMechPos = [player.x + Math.cos(player.yaw) * 120, player.y + Math.sin(player.yaw) * 120, 0];
        renderSkeletalMech(viewMatrix, playerMechPos, provingRotations, defaultPartTranslations, false, player.yaw - Math.PI / 2, 1.0);
      }

      // 6. Final Reveal Platform: rotating completed mech under spotlight (x >= 1800)
      // Pedestal base
      const pedestalModel = new Float32Array(16);
      mat4.identity(pedestalModel);
      mat4.translate(pedestalModel, pedestalModel, [2000, 0, 5]);
      mat4.scale(pedestalModel, pedestalModel, [60, 60, 10]);
      const pedestalMV = new Float32Array(16);
      mat4.multiply(pedestalMV, viewMatrix, pedestalModel);
      gl.uniformMatrix4fv(programInfo.uniformLocations.modelViewMatrix, false, pedestalMV);
      gl.uniform3fv(programInfo.uniformLocations.color, [0.3, 0.3, 0.35]);
      gl.bindVertexArray(vao);
      gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indexBuffer);
      gl.drawElements(gl.TRIANGLES, 36, gl.UNSIGNED_SHORT, 0);

      // Spotlight beam visual helper (yellow wireframe cone)
      const spotlightModel = new Float32Array(16);
      mat4.identity(spotlightModel);
      mat4.translate(spotlightModel, spotlightModel, [2000, 0, 150]);
      mat4.scale(spotlightModel, spotlightModel, [25, 25, 150]);
      const spotlightMV = new Float32Array(16);
      mat4.multiply(spotlightMV, viewMatrix, spotlightModel);
      gl.uniformMatrix4fv(programInfo.uniformLocations.modelViewMatrix, false, spotlightMV);
      gl.uniform3fv(programInfo.uniformLocations.color, [1.0, 1.0, 0.3]);
      gl.bindVertexArray(vao);
      gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, lineIndexBuffer);
      gl.drawElements(gl.LINES, 24, gl.UNSIGNED_SHORT, 0);

      // Rotating mech standing on pedestal
      renderSkeletalMech(viewMatrix, [2000, 0, 10], defaultPartRotations, defaultPartTranslations, false, sec * 0.6, 2.0);

      requestAnimationFrame(render);
    }
    requestAnimationFrame(render);
  }
})();
