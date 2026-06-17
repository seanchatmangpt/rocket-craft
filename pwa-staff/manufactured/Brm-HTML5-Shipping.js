// Barbarian Road Mashines - HTML5 Shipping WebGL/Canvas 3D Client
(function() {
  console.log("Initializing Brm-HTML5-Shipping Client...");

  // Flag the simulated engine as ready
  window.UE4_EngineReady = true;
  console.log("window.UE4_EngineReady set to true.");

  let wasmInstance = null;

  // Game state
  let player = {
    x: 0,
    y: 0,
    z: 200,
    yaw: 0.0,
    pitch: 0.0
  };

  let actors = [];
  let keys = {};

  // DOM Elements
  const canvas = document.getElementById('canvas');
  const ctx = canvas ? canvas.getContext('2d') : null;
  const coordsDiv = document.getElementById('coords');

  if (!canvas || !ctx) {
    console.error("Canvas element or 2D context not found!");
    return;
  }

  // Handle resizing
  function resize() {
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
  }
  window.addEventListener('resize', resize);
  resize();

  // Keyboard Event Listeners
  window.addEventListener('keydown', function(e) {
    keys[e.key] = true;
    keys[e.code] = true;
    if (e.key && typeof e.key === 'string') {
      keys[e.key.toLowerCase()] = true;
    }
  });

  window.addEventListener('keyup', function(e) {
    keys[e.key] = false;
    keys[e.code] = false;
    if (e.key && typeof e.key === 'string') {
      keys[e.key.toLowerCase()] = false;
    }
  });

  // 3D to 2D projection
  function project(x, y, z) {
    if (wasmInstance && wasmInstance.exports && wasmInstance.exports.project_depth) {
      let depth = wasmInstance.exports.project_depth(x, y, z, player.x, player.y, player.z, player.yaw, player.pitch);
      if (depth <= 10) return null;
      let fov = Math.max(canvas.width, canvas.height) * 0.8;
      let screenX = wasmInstance.exports.project_x(x, y, z, player.x, player.y, player.z, player.yaw, player.pitch, fov, canvas.width, canvas.height);
      let screenY = wasmInstance.exports.project_y(x, y, z, player.x, player.y, player.z, player.yaw, player.pitch, fov, canvas.width, canvas.height);
      return { x: screenX, y: screenY, depth: depth };
    }

    // Translate point relative to camera
    let dx = x - player.x;
    let dy = y - player.y;
    let dz = z - player.z;

    // Rotate around Yaw (horizontal angle)
    let cosYaw = Math.cos(-player.yaw);
    let sinYaw = Math.sin(-player.yaw);
    let x1 = dx * cosYaw - dy * sinYaw;
    let y1 = dx * sinYaw + dy * cosYaw;

    // Rotate around Pitch (vertical angle)
    let cosPitch = Math.cos(-player.pitch);
    let sinPitch = Math.sin(-player.pitch);
    let x2 = x1 * cosPitch - dz * sinPitch;
    let z2 = x1 * sinPitch + dz * cosPitch;
    let y2 = y1;

    // Depth check - must be in front of the camera
    if (x2 <= 10) {
      return null;
    }

    let fov = Math.max(canvas.width, canvas.height) * 0.8;
    let screenX = canvas.width / 2 + (y2 / x2) * fov;
    let screenY = canvas.height / 2 - (z2 / x2) * fov;

    return { x: screenX, y: screenY, depth: x2 };
  }

  // Draw 3D Box (Place / Room)
  function drawBox(actor) {
    let loc = actor.location;
    let scale = actor.scale;
    // Base size is 100x100x100
    let hx = (scale.x || 1.0) * 50;
    let hy = (scale.y || 1.0) * 50;
    let hz = (scale.z || 1.0) * 50;

    let pts = [
      {x: -hx, y: -hy, z: -hz},
      {x:  hx, y: -hy, z: -hz},
      {x:  hx, y:  hy, z: -hz},
      {x: -hx, y:  hy, z: -hz},
      {x: -hx, y: -hy, z:  hz},
      {x:  hx, y: -hy, z:  hz},
      {x:  hx, y:  hy, z:  hz},
      {x: -hx, y:  hy, z:  hz}
    ];

    let projPts = pts.map(p => project(loc.x + p.x, loc.y + p.y, loc.z + p.z));

    let faces = [
      [0, 1, 2, 3], // Bottom
      [4, 5, 6, 7], // Top
      [0, 1, 5, 4], // Front
      [2, 3, 7, 6], // Back
      [0, 3, 7, 4], // Left
      [1, 2, 6, 5]  // Right
    ];

    ctx.fillStyle = actor.color || 'rgba(0, 120, 240, 0.12)';
    ctx.strokeStyle = actor.borderColor || 'rgba(0, 160, 255, 0.8)';
    ctx.lineWidth = 2;

    for (let face of faces) {
      if (face.every(idx => projPts[idx] !== null)) {
        ctx.beginPath();
        ctx.moveTo(projPts[face[0]].x, projPts[face[0]].y);
        for (let i = 1; i < face.length; i++) {
          ctx.lineTo(projPts[face[i]].x, projPts[face[i]].y);
        }
        ctx.closePath();
        ctx.fill();
        ctx.stroke();
      }
    }
  }

  // Draw 3D Cylinder (Actor Bot)
  function drawCylinder(actor) {
    let loc = actor.location;
    let scale = actor.scale;
    // Base radius 40, height 120
    let R = (scale.x || 1.0) * 35;
    let hz = (scale.z || 1.0) * 60;

    let numSegments = 12;
    let topPts = [];
    let botPts = [];

    for (let i = 0; i < numSegments; i++) {
      let angle = (i / numSegments) * Math.PI * 2;
      let cx = Math.cos(angle) * R;
      let cy = Math.sin(angle) * R;
      topPts.push(project(loc.x + cx, loc.y + cy, loc.z + hz));
      botPts.push(project(loc.x + cx, loc.y + cy, loc.z - hz));
    }

    ctx.fillStyle = actor.color || 'rgba(0, 220, 100, 0.18)';
    ctx.strokeStyle = actor.borderColor || 'rgba(0, 255, 120, 0.8)';
    ctx.lineWidth = 1.5;

    // Bottom Circle
    if (botPts.every(p => p !== null)) {
      ctx.beginPath();
      ctx.moveTo(botPts[0].x, botPts[0].y);
      for (let i = 1; i < numSegments; i++) ctx.lineTo(botPts[i].x, botPts[i].y);
      ctx.closePath();
      ctx.fill();
      ctx.stroke();
    }

    // Top Circle
    if (topPts.every(p => p !== null)) {
      ctx.beginPath();
      ctx.moveTo(topPts[0].x, topPts[0].y);
      for (let i = 1; i < numSegments; i++) ctx.lineTo(topPts[i].x, topPts[i].y);
      ctx.closePath();
      ctx.fill();
      ctx.stroke();
    }

    // Connect them
    for (let i = 0; i < numSegments; i++) {
      let pt = topPts[i];
      let pb = botPts[i];
      if (pt && pb) {
        ctx.beginPath();
        ctx.moveTo(pb.x, pb.y);
        ctx.lineTo(pt.x, pt.y);
        ctx.stroke();
      }
    }
  }

  // Draw 3D Sphere (Object Prop)
  function drawSphere(actor) {
    let loc = actor.location;
    let scale = actor.scale;
    let R = (scale.x || 1.0) * 45;

    ctx.strokeStyle = actor.borderColor || 'rgba(255, 140, 0, 0.8)';
    ctx.fillStyle = actor.color || 'rgba(255, 140, 0, 0.18)';
    ctx.lineWidth = 1.5;

    let numSegments = 16;
    
    // Draw 3 perpendicular rings
    let xyRing = [], xzRing = [], yzRing = [];
    for (let i = 0; i <= numSegments; i++) {
      let angle = (i / numSegments) * Math.PI * 2;
      xyRing.push(project(loc.x + Math.cos(angle) * R, loc.y + Math.sin(angle) * R, loc.z));
      xzRing.push(project(loc.x + Math.cos(angle) * R, loc.y, loc.z + Math.sin(angle) * R));
      yzRing.push(project(loc.x, loc.y + Math.cos(angle) * R, loc.z + Math.sin(angle) * R));
    }

    function drawRing(pts) {
      ctx.beginPath();
      let first = true;
      for (let p of pts) {
        if (!p) continue;
        if (first) {
          ctx.moveTo(p.x, p.y);
          first = false;
        } else {
          ctx.lineTo(p.x, p.y);
        }
      }
      ctx.stroke();
    }

    drawRing(xyRing);
    drawRing(xzRing);
    drawRing(yzRing);

    // Draw shaded facing circle
    let centerProj = project(loc.x, loc.y, loc.z);
    if (centerProj) {
      let fov = Math.max(canvas.width, canvas.height) * 0.8;
      let r2d = (R / centerProj.depth) * fov;
      if (r2d > 0) {
        ctx.beginPath();
        ctx.arc(centerProj.x, centerProj.y, r2d, 0, Math.PI * 2);
        ctx.fill();
      }
    }
  }

  // Draw grounding grid
  function drawGrid() {
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.05)';
    ctx.lineWidth = 1;
    let size = 2000;
    let step = 100;
    
    for (let x = -size; x <= size; x += step) {
      let p1 = project(x, -size, -100);
      let p2 = project(x, size, -100);
      if (p1 && p2) {
        ctx.beginPath();
        ctx.moveTo(p1.x, p1.y);
        ctx.lineTo(p2.x, p2.y);
        ctx.stroke();
      }
    }
    for (let y = -size; y <= size; y += step) {
      let p1 = project(-size, y, -100);
      let p2 = project(size, y, -100);
      if (p1 && p2) {
        ctx.beginPath();
        ctx.moveTo(p1.x, p1.y);
        ctx.lineTo(p2.x, p2.y);
        ctx.stroke();
      }
    }
  }

  // Draw actor labels
  function drawLabels(actor) {
    let loc = actor.location;
    let scale = actor.scale;
    let topZ = loc.z;
    if (actor.mesh === 'Cube') topZ += (scale.z || 1.0) * 50;
    else if (actor.mesh === 'Cylinder') topZ += (scale.z || 1.0) * 60;
    else topZ += (scale.x || 1.0) * 45;

    let proj = project(loc.x, loc.y, topZ + 15);
    if (proj) {
      ctx.fillStyle = '#ffffff';
      ctx.font = 'bold 12px "Courier New", monospace';
      ctx.textAlign = 'center';
      ctx.shadowColor = 'black';
      ctx.shadowBlur = 4;
      ctx.fillText(actor.label, proj.x, proj.y);
      ctx.shadowBlur = 0; // reset
    }
  }

  // Draw everything
  function drawScene() {
    ctx.fillStyle = '#05050a';
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    drawGrid();

    // Sort actors by depth (furthest to closest)
    let sorted = actors.map(actor => {
      let dx = actor.location.x - player.x;
      let dy = actor.location.y - player.y;
      let dz = actor.location.z - player.z;
      let dist = dx*dx + dy*dy + dz*dz;
      return { actor, dist };
    }).sort((a, b) => b.dist - a.dist);

    for (let s of sorted) {
      let actor = s.actor;
      if (actor.mesh === 'Cube') {
        drawBox(actor);
      } else if (actor.mesh === 'Cylinder') {
        drawCylinder(actor);
      } else if (actor.mesh === 'Sphere') {
        drawSphere(actor);
      }
      drawLabels(actor);
    }
  }

  // Frame update
  let lastTime = performance.now();
  function update(time) {
    let dt = (time - lastTime) / 1000.0;
    lastTime = time;

    // Cap dt to prevent huge jumps
    if (dt > 0.1) dt = 0.1;

    let speed = 250.0 * dt; // Units/sec
    let rotSpeed = 1.8 * dt; // Rad/sec

    let cosYaw = Math.cos(player.yaw);
    let sinYaw = Math.sin(player.yaw);

    let moved = false;

    // W/S: forward/backward
    if (keys['w'] || keys['W'] || keys['KeyW']) {
      player.x += cosYaw * speed;
      player.y += sinYaw * speed;
      moved = true;
    }
    if (keys['s'] || keys['S'] || keys['KeyS']) {
      player.x -= cosYaw * speed;
      player.y -= sinYaw * speed;
      moved = true;
    }

    // A/D: strafe left/right
    if (keys['a'] || keys['A'] || keys['KeyA']) {
      player.x -= sinYaw * speed;
      player.y += cosYaw * speed;
      moved = true;
    }
    if (keys['d'] || keys['D'] || keys['KeyD']) {
      player.x += sinYaw * speed;
      player.y -= cosYaw * speed;
      moved = true;
    }

    // Space/Shift: fly up/down
    if (keys[' '] || keys['spacebar'] || keys['Space']) {
      player.z += speed;
      moved = true;
    }
    if (keys['shift'] || keys['Shift'] || keys['ShiftLeft'] || keys['ShiftRight']) {
      player.z -= speed;
      moved = true;
    }

    // Arrow keys: turn
    if (keys['ArrowLeft'] || keys['arrowleft']) {
      player.yaw -= rotSpeed;
      moved = true;
    }
    if (keys['ArrowRight'] || keys['arrowright']) {
      player.yaw += rotSpeed;
      moved = true;
    }
    if (keys['ArrowUp'] || keys['arrowup']) {
      player.pitch = Math.max(-Math.PI/2.2, Math.min(Math.PI/2.2, player.pitch + rotSpeed));
      moved = true;
    }
    if (keys['ArrowDown'] || keys['arrowdown']) {
      player.pitch = Math.max(-Math.PI/2.2, Math.min(Math.PI/2.2, player.pitch - rotSpeed));
      moved = true;
    }

    if (moved) {
      console.log(`Player moved: X=${player.x.toFixed(2)}, Y=${player.y.toFixed(2)}, Z=${player.z.toFixed(2)}, Yaw=${player.yaw.toFixed(2)}, Pitch=${player.pitch.toFixed(2)}`);
      if (coordsDiv) {
        coordsDiv.innerText = `Player: X:${player.x.toFixed(1)} Y:${player.y.toFixed(1)} Z:${player.z.toFixed(1)}`;
      }
    }

    drawScene();
    requestAnimationFrame(update);
  }

  // Load level layout data
  console.log("Fetching layout data from Brm-HTML5-Shipping.data...");
  fetch('/manufactured/Brm-HTML5-Shipping.data')
    .then(response => {
      if (!response.ok) {
        throw new Error("HTTP error " + response.status);
      }
      return response.text();
    })
    .then(text => {
      actors = JSON.parse(text);
      console.log(`Successfully loaded ${actors.length} actors from Brm-HTML5-Shipping.data.`);
      
      // Assign custom styling based on actor roles
      actors.forEach(actor => {
        if (actor.name.toLowerCase().includes('place') || actor.class.includes('StaticMeshActor')) {
          actor.mesh = 'Cube';
          actor.color = 'rgba(0, 100, 200, 0.12)';
          actor.borderColor = 'rgba(0, 160, 255, 0.8)';
        } else if (actor.name.toLowerCase().includes('actor_bot') || actor.label.toLowerCase().includes('bot') || actor.label.toLowerCase().includes('officer') || actor.label.toLowerCase().includes('instructor') || actor.label.toLowerCase().includes('welder')) {
          actor.mesh = 'Cylinder';
          actor.color = 'rgba(0, 220, 100, 0.18)';
          actor.borderColor = 'rgba(0, 255, 120, 0.8)';
        } else {
          actor.mesh = 'Sphere';
          actor.color = 'rgba(255, 140, 0, 0.18)';
          actor.borderColor = 'rgba(255, 160, 0, 0.8)';
        }
      });

      // Align player to view the first floor
      let floor = actors.find(a => a.mesh === 'Cube');
      if (floor) {
        player.x = floor.location.x - 350;
        player.y = floor.location.y - 350;
        player.z = floor.location.z + 250;
        player.yaw = Math.atan2(floor.location.y - player.y, floor.location.x - player.x);
        player.pitch = -0.35;
      } else {
        player.x = 0; player.y = -400; player.z = 200;
        player.yaw = Math.PI / 2; player.pitch = -0.2;
      }

      if (coordsDiv) {
        coordsDiv.innerText = `Player: X:${player.x.toFixed(1)} Y:${player.y.toFixed(1)} Z:${player.z.toFixed(1)}`;
      }

      // Load WASM as required
      fetch('/manufactured/Brm-HTML5-Shipping.wasm')
        .then(res => res.arrayBuffer())
        .then(bytes => WebAssembly.instantiate(bytes))
        .then(results => {
          console.log("WASM module compiled successfully!");
          wasmInstance = results.instance;
        })
        .catch(err => {
          console.warn("WASM module compile skipped/mocked: ", err);
        });

      // Start update loop
      requestAnimationFrame(update);
    })
    .catch(err => {
      console.warn("Could not fetch Brm-HTML5-Shipping.data, starting with fallback actor:", err);
      actors = [
        {
          name: "Place_fallback",
          class: "StaticMeshActor",
          label: "Fallback World Room",
          location: {x: 0, y: 0, z: -100},
          scale: {x: 3, y: 3, z: 1},
          mesh: "Cube",
          color: "rgba(120, 120, 120, 0.15)",
          borderColor: "rgba(180, 180, 180, 0.8)"
        }
      ];
      if (coordsDiv) {
        coordsDiv.innerText = `Player: X:${player.x.toFixed(1)} Y:${player.y.toFixed(1)} Z:${player.z.toFixed(1)}`;
      }
      requestAnimationFrame(update);
    });
})();
