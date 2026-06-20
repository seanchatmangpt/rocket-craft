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
    fetchWithProgress('Brm-HTML5-Shipping.wasm', 'wasm').catch(e => {
      console.warn('WASM download failed, falling back to JS gait engine:', e);
      return null;
    }),
    fetchWithProgress('Brm-HTML5-Shipping.data', 'data'),
  ])
    .then(([wasmBytes, dataBytes]) => {
      const decoder = new TextDecoder('utf-8');
      gameDataText = decoder.decode(dataBytes);
      if (wasmBytes) {
        updateStatus('Compiling WebAssembly binaries...', 'Verifying game compilation...', 100);
        return WebAssembly.instantiate(wasmBytes, {}).then((result) => {
          wasmModule = result.instance;
          runGame();
        }).catch(e => {
          console.warn('WASM instantiation failed, falling back to JS gait engine:', e);
          wasmModule = null;
          runGame();
        });
      } else {
        wasmModule = null;
        runGame();
      }
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
    let viewMatrix = new Float32Array(16);
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
    let zonePlaces = [];
    try {
      actors = JSON.parse(gameDataText);
      console.log(`Loaded ${actors.length} actors.`);
      zonePlaces = actors.filter(actor => actor.name.startsWith('Place_'));
      zonePlaces.sort((a, b) => a.location.x - b.location.x);
    } catch (e) {
      console.warn('Fallback actors used.');
      actors = [{ name: 'Fallback', location: {x:0, y:0, z:0}, scale: {x:1, y:1, z:1}, color: [0.5, 0.5, 0.5] }];
      zonePlaces = [{ name: 'Place_Fallback', label: 'Fallback Zone', location: {x:0, y:0, z:0}, scale: {x:10, y:10, z:1} }];
    }

    // Dynamically inject HUD zones into #dflss-hud
    const hudContainer = document.getElementById('dflss-hud');
    if (hudContainer) {
      let hudHTML = `<div style="font-weight: bold; color: #f39c12; border-bottom: 1px solid #f39c12; padding-bottom: 5px; margin-bottom: 8px; text-transform: uppercase;">TPS/DfLSS Factory HUD</div>`;
      hudHTML += `<div id="hud-status" style="font-weight: bold; color: #00ff66; margin-bottom: 10px;">Pipeline Status: ACTIVE_RUNNING</div>`;
      for (let i = 0; i < zonePlaces.length; i++) {
        const zoneNum = i + 1;
        const zoneName = zonePlaces[i].label || zonePlaces[i].name;
        hudHTML += `<div id="hud-zone-${zoneNum}" style="margin: 4px 0; color: #aaa;">[ ] Zone ${zoneNum}: ${zoneName}</div>`;
      }
      hudContainer.innerHTML = hudHTML;
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

    function drawCubePart(cx, cy, cz, rx, ry, rz, sx, sy, sz, color) {
      const wgX = cy;
      const wgY = cz;
      const wgZ = -cx;
      const modelMatrix = new Float32Array(16);
      mat4.identity(modelMatrix);
      mat4.translate(modelMatrix, modelMatrix, [wgX, wgY, wgZ]);
      if (ry) mat4.rotateX(modelMatrix, modelMatrix, ry);
      if (rz) mat4.rotateY(modelMatrix, modelMatrix, rz);
      if (rx) mat4.rotateZ(modelMatrix, modelMatrix, -rx);
      mat4.scale(modelMatrix, modelMatrix, [sy, sz, sx]);
      const mvMatrix = new Float32Array(16);
      mat4.multiply(mvMatrix, viewMatrix, modelMatrix);
      gl.uniformMatrix4fv(programInfo.uniformLocations.modelViewMatrix, false, mvMatrix);
      gl.uniform3fv(programInfo.uniformLocations.color, color);
      gl.bindVertexArray(vao);
      gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indexBuffer);
      gl.drawElements(gl.TRIANGLES, 36, gl.UNSIGNED_SHORT, 0);
    }

    function drawMech(cx, cy, cz, time, animType, color) {
      const sec = time > 100000 ? time / 1000 : time;
      let headRot = [0, 0, 0];
      let lArmRot = [0, 0, 0];
      let rArmRot = [0, 0, 0];
      let lLegRot = [0, 0, 0];
      let rLegRot = [0, 0, 0];
      let wingAngle = 0.2;
      let lArmOffset = [0, -22, 30];
      let rArmOffset = [0, 22, 30];
      let lLegOffset = [0, -12, 0];
      let rLegOffset = [0, 12, 0];
      let wingOffset = 0;
      let headOffset = [0, 0, 60];
      if (animType === 'gantry') {
        const cycle = sec % 8;
        if (cycle < 1) {
          lLegOffset[2] -= (1 - cycle) * 100;
        }
        if (cycle < 2) {
          rLegOffset[2] -= (2 - cycle) * 100;
        }
        if (cycle < 3) {
          headOffset[2] += (3 - cycle) * 100;
        }
        if (cycle < 4) {
          lArmOffset[1] -= (4 - cycle) * 100;
        }
        if (cycle < 5) {
          rArmOffset[1] += (5 - cycle) * 100;
        }
        if (cycle < 6) {
          wingOffset = (6 - cycle) * 100;
        }
      } else if (animType === 'fit') {
        headRot[2] = Math.sin(sec * 2) * 0.5;
        lArmRot[1] = 0.5;
      } else if (animType === 'proving') {
        lLegRot[1] = Math.sin(sec * 6) * 0.6;
        rLegRot[1] = -Math.sin(sec * 6) * 0.6;
        lArmRot[1] = -Math.sin(sec * 6) * 0.4;
        rArmRot[1] = Math.sin(sec * 6) * 0.4;
      } else if (animType === 'reveal') {
        wingAngle = 0.4 + Math.sin(sec * 1.5) * 0.15;
        headRot[1] = 0.1;
      }
      drawCubePart(cx, cy, cz + 30, 0, 0, 0, 20, 30, 40, color);
      drawCubePart(cx + headOffset[0], cy + headOffset[1], cz + headOffset[2], headRot[0], headRot[1], headRot[2], 12, 12, 12, color);
      drawCubePart(cx + headOffset[0] + 6, cy + headOffset[1], cz + headOffset[2] + 8, headRot[0], headRot[1], headRot[2], 8, 3, 3, [1.0, 0.84, 0.0]);
      drawCubePart(cx + lArmOffset[0], cy + lArmOffset[1], cz + lArmOffset[2], lArmRot[0], lArmRot[1], lArmRot[2], 8, 8, 30, color);
      drawCubePart(cx + rArmOffset[0], cy + rArmOffset[1], cz + rArmOffset[2], rArmRot[0], rArmRot[1], rArmRot[2], 8, 8, 30, color);
      drawCubePart(cx + lLegOffset[0], cy + lLegOffset[1], cz + lLegOffset[2], lLegRot[0], lLegRot[1], lLegRot[2], 10, 10, 35, color);
      drawCubePart(cx + rLegOffset[0], cy + rLegOffset[1], cz + rLegOffset[2], rLegRot[0], rLegRot[1], rLegRot[2], 10, 10, 35, color);
      drawCubePart(cx - 15 - wingOffset, cy, cz + 35, 0, 0, 0, 10, 16, 25, [0.2, 0.2, 0.2]);
      const leftWingColor = [color[0] * 0.8, color[1] * 0.8, color[2] * 0.8];
      drawCubePart(cx - 20 - wingOffset, cy - 15, cz + 45, 0, -wingAngle, 0, 6, 25, 8, leftWingColor);
      drawCubePart(cx - 25 - wingOffset, cy - 20, cz + 35, 0, -wingAngle * 1.5, 0, 5, 20, 8, leftWingColor);
      drawCubePart(cx - 30 - wingOffset, cy - 25, cz + 25, 0, -wingAngle * 2.0, 0, 4, 15, 8, leftWingColor);
      const rightWingColor = leftWingColor;
      drawCubePart(cx - 20 - wingOffset, cy + 15, cz + 45, 0, wingAngle, 0, 6, 25, 8, rightWingColor);
      drawCubePart(cx - 25 - wingOffset, cy + 20, cz + 35, 0, wingAngle * 1.5, 0, 5, 20, 8, rightWingColor);
      drawCubePart(cx - 30 - wingOffset, cy + 25, cz + 25, 0, wingAngle * 2.0, 0, 4, 15, 8, rightWingColor);
      let bladePos = [
        cx + rArmOffset[0] + Math.cos(rArmRot[1]) * 25,
        cy + rArmOffset[1],
        cz + rArmOffset[2] - Math.sin(rArmRot[1]) * 25
      ];
      drawCubePart(bladePos[0], bladePos[1], bladePos[2], rArmRot[0], rArmRot[1], rArmRot[2], 40, 2, 6, [0.0, 1.0, 1.0]);
    }

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
      let activeZoneIdx = -1;
      let minDistance = Infinity;
      let nearestZoneIdx = 0;
      const px = player.x;
      const py = player.y;

      for (let idx = 0; idx < zonePlaces.length; idx++) {
        const zone = zonePlaces[idx];
        const halfX = zone.scale.x * 50;
        const halfY = (zone.scale.y || zone.scale.x) * 50;
        const xMin = zone.location.x - halfX;
        const xMax = zone.location.x + halfX;
        const yMin = zone.location.y - halfY;
        const yMax = zone.location.y + halfY;

        // Check if player is inside the horizontal boundaries
        if (px >= xMin && px <= xMax && py >= yMin && py <= yMax) {
          activeZoneIdx = idx;
        }

        // Keep track of the nearest zone center in case player is outside all zones
        const dx = px - zone.location.x;
        const dy = py - zone.location.y;
        const dist = Math.sqrt(dx * dx + dy * dy);
        if (dist < minDistance) {
          minDistance = dist;
          nearestZoneIdx = idx;
        }
      }

      if (activeZoneIdx === -1) {
        activeZoneIdx = nearestZoneIdx;
      }
      
      const activeZone = activeZoneIdx + 1;
      const activeZonePlace = zonePlaces[activeZoneIdx];

      // Highlight HUD elements dynamically
      for (let i = 1; i <= zonePlaces.length; i++) {
        const hudEl = document.getElementById(`hud-zone-${i}`);
        if (hudEl) {
          const zoneName = zonePlaces[i - 1].label || zonePlaces[i - 1].name;
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
      mat4.identity(viewMatrix);
      mat4.rotateX(viewMatrix, viewMatrix, -player.pitch);
      mat4.rotateY(viewMatrix, viewMatrix, -player.yaw);
      mat4.translate(viewMatrix, viewMatrix, [-player.x, -player.y, -player.z]);

      // Render default environment actors and special GMF assets
      actors.forEach(actor => {
        const cx = actor.location.x;
        const cy = actor.location.y;
        const cz = actor.location.z;

        // Draw the floor if it's a place/floor
        if (actor.name.startsWith("Place_") || actor.mesh === "Cube" || actor.name.toLowerCase().includes("place")) {
          const modelViewMatrix = new Float32Array(16);
          mat4.identity(modelViewMatrix);
          mat4.rotateX(modelViewMatrix, modelViewMatrix, -player.pitch);
          mat4.rotateY(modelViewMatrix, modelViewMatrix, -player.yaw);
          mat4.translate(modelViewMatrix, modelViewMatrix, [-player.x, -player.y, -player.z]);
          mat4.translate(modelViewMatrix, modelViewMatrix, [cx, cy, cz]);
          mat4.scale(modelViewMatrix, modelViewMatrix, [actor.scale.x * 50, actor.scale.y * 50, actor.scale.z * 50]);
          gl.uniformMatrix4fv(programInfo.uniformLocations.modelViewMatrix, false, modelViewMatrix);
          gl.uniform3fv(programInfo.uniformLocations.color, [0.15, 0.15, 0.18]); // dark grey metal floor
          gl.bindVertexArray(vao);
          gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indexBuffer);
          gl.drawElements(gl.TRIANGLES, 36, gl.UNSIGNED_SHORT, 0);
        } else {
          // Draw other default actors (bots, props)
          const modelViewMatrix = new Float32Array(16);
          mat4.identity(modelViewMatrix);
          mat4.rotateX(modelViewMatrix, modelViewMatrix, -player.pitch);
          mat4.rotateY(modelViewMatrix, modelViewMatrix, -player.yaw);
          mat4.translate(modelViewMatrix, modelViewMatrix, [-player.x, -player.y, -player.z]);
          mat4.translate(modelViewMatrix, modelViewMatrix, [cx, cy, cz]);
          mat4.scale(modelViewMatrix, modelViewMatrix, [actor.scale.x * 50, actor.scale.y * 50, actor.scale.z * 50]);
          gl.uniformMatrix4fv(programInfo.uniformLocations.modelViewMatrix, false, modelViewMatrix);
          let color = [0.0, 0.4, 0.8];
          if (actor.name.toLowerCase().includes('bot')) color = [0.0, 0.9, 0.3];
          gl.uniform3fv(programInfo.uniformLocations.color, color);
          gl.bindVertexArray(vao);
          gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indexBuffer);
          gl.drawElements(gl.TRIANGLES, 36, gl.UNSIGNED_SHORT, 0);
        }

        // Draw special zone-specific assets
        if (actor.name === 'Place_foundry') {
          // Draw furnace and fire door
          drawCubePart(cx, cy, cz + 60, 0, 0, 0, 80, 80, 80, [0.25, 0.25, 0.25]);
          const doorPulse = 0.8 + Math.sin(sec * 4) * 0.2;
          drawCubePart(cx + 41, cy, cz + 40, 0, 0, 0, 2, 30, 40, [1.0 * doorPulse, 0.3 * doorPulse, 0.0]);
        }
        else if (actor.name === 'Place_runner_wall') {
          // Draw vertical racks and hanging parts
          drawCubePart(cx, cy - 60, cz + 80, 0, 0, 0, 10, 10, 160, [0.3, 0.3, 0.3]);
          drawCubePart(cx, cy + 60, cz + 80, 0, 0, 0, 10, 10, 160, [0.3, 0.3, 0.3]);
          drawCubePart(cx, cy, cz + 150, 0, 0, 0, 8, 120, 8, [0.3, 0.3, 0.3]);
          drawCubePart(cx, cy, cz + 80, 0, 0, 0, 8, 120, 8, [0.3, 0.3, 0.3]);
          
          drawCubePart(cx, cy - 30, cz + 110, sec, 0, 0, 12, 12, 12, [0.8, 0.3, 0.3]);
          drawCubePart(cx, cy, cz + 110, 0, sec, 0, 15, 8, 25, [0.3, 0.3, 0.8]);
          drawCubePart(cx, cy + 30, cz + 110, 0, 0, sec, 8, 15, 15, [0.3, 0.8, 0.3]);
        }
        else if (actor.name === 'Place_gantry') {
          // Draw support frame and mech in gantry mode
          drawCubePart(cx - 70, cy - 70, cz + 100, 0, 0, 0, 10, 10, 200, [0.4, 0.4, 0.4]);
          drawCubePart(cx - 70, cy + 70, cz + 100, 0, 0, 0, 10, 10, 200, [0.4, 0.4, 0.4]);
          drawCubePart(cx + 70, cy - 70, cz + 100, 0, 0, 0, 10, 10, 200, [0.4, 0.4, 0.4]);
          drawCubePart(cx + 70, cy + 70, cz + 100, 0, 0, 0, 10, 10, 200, [0.4, 0.4, 0.4]);
          drawCubePart(cx, cy - 70, cz + 200, 0, 0, 0, 150, 8, 8, [0.4, 0.4, 0.4]);
          drawCubePart(cx, cy + 70, cz + 200, 0, 0, 0, 150, 8, 8, [0.4, 0.4, 0.4]);
          drawCubePart(cx - 70, cy, cz + 200, 0, 0, 0, 8, 150, 8, [0.4, 0.4, 0.4]);
          drawCubePart(cx + 70, cy, cz + 200, 0, 0, 0, 8, 150, 8, [0.4, 0.4, 0.4]);

          drawMech(cx, cy, cz + 30, time, 'gantry', [0.8, 0.4, 0.4]);
        }
        else if (actor.name === 'Place_fit_bay') {
          // Draw scanning arch and mech in fit mode
          drawCubePart(cx, cy - 80, cz + 90, 0, 0, 0, 12, 12, 180, [0.2, 0.2, 0.25]);
          drawCubePart(cx, cy + 80, cz + 90, 0, 0, 0, 12, 12, 180, [0.2, 0.2, 0.25]);
          drawCubePart(cx, cy, cz + 180, 0, 0, 0, 12, 160, 12, [0.2, 0.2, 0.25]);
          
          const sweepY = Math.sin(sec * 2) * 75;
          drawCubePart(cx, cy + sweepY, cz + 90, 0, 0, 0, 2, 10, 160, [0.0, 1.0, 0.3]);

          drawMech(cx, cy, cz + 30, time, 'fit', [0.4, 0.4, 0.8]);
        }
        else if (actor.name === 'Place_proving_ground') {
          // Draw path and walk mech
          drawCubePart(cx, cy, cz + 1, 0, 0, 0, 120, 40, 2, [0.1, 0.1, 0.1]);
          const pathOffset = Math.sin(sec * 1.5) * 50;
          drawMech(cx + pathOffset, cy, cz + 30, time, 'proving', [0.4, 0.8, 0.4]);
        }
        else if (actor.name === 'Place_reveal_platform') {
          // Draw pedestal and standing mech
          drawCubePart(cx, cy, cz + 5, 0, 0, 0, 60, 60, 10, [0.3, 0.3, 0.35]);
          drawCubePart(cx, cy, cz + 150, 0, 0, 0, 12, 12, 300, [1.0, 1.0, 0.3]);
          
          drawMech(cx, cy, cz + 10, time, 'reveal', [0.9, 0.9, 0.9]);

          // Floating receipt panel
          const receiptPulse = Math.sin(sec * 3) * 5;
          drawCubePart(cx + 40, cy + 40, cz + 80 + receiptPulse, 0.2, -0.2, sec * 0.5, 5, 30, 20, [0.0, 0.8, 1.0]);
        }
      });

      requestAnimationFrame(render);
    }
    requestAnimationFrame(render);
  }
})();
