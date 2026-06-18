/**
 * @file app.js
 * @description Genie 26 Web Simulator Core Logic. Renders a 3D representation of the world specification using Three.js.
 */

let scene, camera, renderer, controls;
let container;
let gridHelper;
let entities = { places: [], actors: [], objects: [] };
let threeObjects = new Map(); // maps entity ID -> Three.js Object3D
let selectedEntityId = null;
let currentSpec = null;

// Walkthrough Mode Variables
let isWalkMode = false;
let yaw = 0;
let pitch = 0;
let keysPressed = { w: false, a: false, s: false, d: false };

/**
 * Initializes the Three.js scene, camera, renderer, controls, and lighting.
 * @returns {void}
 */
function initThree() {
    container = document.getElementById('canvas-container');
    
    // Scene
    scene = new THREE.Scene();
    scene.background = new THREE.Color(0x0a0c10);
    scene.fog = new THREE.FogExp2(0x0a0c10, 0.0015);

    // Camera
    camera = new THREE.PerspectiveCamera(60, container.clientWidth / container.clientHeight, 1, 10000);
    camera.position.set(150, 150, 250);

    // Renderer
    renderer = new THREE.WebGLRenderer({ antialias: true });
    renderer.setSize(container.clientWidth, container.clientHeight);
    renderer.setPixelRatio(window.devicePixelRatio);
    renderer.shadowMap.enabled = true;
    container.appendChild(renderer.domElement);

    // Controls
    controls = new THREE.OrbitControls(camera, renderer.domElement);
    controls.enableDamping = true;
    controls.dampingFactor = 0.05;
    controls.maxPolarAngle = Math.PI / 2 - 0.05; // Don't go below ground

    // Lights
    const ambientLight = new THREE.AmbientLight(0x1a2030);
    scene.add(ambientLight);

    const dirLight = new THREE.DirectionalLight(0xffffff, 0.8);
    dirLight.position.set(200, 400, 200);
    dirLight.castShadow = true;
    scene.add(dirLight);

    const pointLight = new THREE.PointLight(0x7928ca, 1.5, 500);
    pointLight.position.set(0, 100, 0);
    scene.add(pointLight);

    // Grid
    gridHelper = new THREE.GridHelper(2000, 100, 0x24292e, 0x161b22);
    gridHelper.position.y = -0.5;
    scene.add(gridHelper);

    // Handle Window Resize
    window.addEventListener('resize', () => {
        camera.aspect = container.clientWidth / container.clientHeight;
        camera.updateProjectionMatrix();
        renderer.setSize(container.clientWidth, container.clientHeight);
    });

    // Setup Raycaster for Mouse Clicking
    const raycaster = new THREE.Raycaster();
    const mouse = new THREE.Vector2();

    renderer.domElement.addEventListener('pointerdown', (event) => {
        // Calculate mouse position in normalized device coordinates
        const rect = renderer.domElement.getBoundingClientRect();
        mouse.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
        mouse.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;

        raycaster.setFromCamera(mouse, camera);

        // Filter out mesh objects that belong to our entities
        const checkObjects = [];
        threeObjects.forEach((obj, id) => {
            // Find child meshes
            obj.traverse((child) => {
                if (child.isMesh) {
                    child.userData.entityId = id;
                    checkObjects.push(child);
                }
            });
        });

        const intersects = raycaster.intersectObjects(checkObjects);
        if (intersects.length > 0) {
            const hitMesh = intersects[0].object;
            const entityId = hitMesh.userData.entityId;
            selectEntity(entityId);
        }
    });

    animate();
}

/**
 * Main animation and rendering loop.
 * Updates controls and renders the scene.
 * @returns {void}
 */
function animate() {
    requestAnimationFrame(animate);
    if (isWalkMode) {
        updateWalkthroughMovement();
    } else {
        controls.update();
    }
    renderer.render(scene, camera);
}

/**
 * Creates a 2D canvas text sprite to overlay on top of 3D objects.
 * @param {string} text - The text label to display.
 * @param {string} [color='#f0f3f6'] - The hex color code of the text.
 * @returns {THREE.Sprite} The generated text sprite.
 */
function createTextSprite(text, color = '#f0f3f6') {
    const canvas = document.createElement('canvas');
    canvas.width = 256;
    canvas.height = 64;
    const ctx = canvas.getContext('2d');
    
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.font = 'Bold 24px Outfit';
    ctx.fillStyle = color;
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText(text, canvas.width / 2, canvas.height / 2);

    const texture = new THREE.CanvasTexture(canvas);
    const material = new THREE.SpriteMaterial({ map: texture, transparent: true });
    const sprite = new THREE.Sprite(material);
    sprite.scale.set(60, 15, 1);
    return sprite;
}

/**
 * Rebuilds the entire 3D world representation based on the loaded world specification.
 * Clears old meshes and draws places, actors, and objects.
 * @param {Object} spec - The world specification.
 * @returns {void}
 */
function rebuild3DWorld(spec) {
    // Clear old objects
    threeObjects.forEach((obj) => {
        scene.remove(obj);
    });
    threeObjects.clear();

    const placeMap = new Map();
    spec.places.forEach((place) => {
        placeMap.set(place.id, place);

        // Draw Place: Semi-transparent glass box
        const center = place.bounds.center;
        const extents = place.bounds.half_extents;

        const geometry = new THREE.BoxGeometry(extents.x * 2, extents.z * 2, extents.y * 2); // UE4 Z is up, Three.js Y is up
        const material = new THREE.MeshPhongMaterial({
            color: 0x7928ca,
            transparent: true,
            opacity: 0.15,
            wireframe: false,
            side: THREE.DoubleSide,
            depthWrite: false
        });

        const box = new THREE.Mesh(geometry, material);
        box.position.set(center.x, center.z, -center.y); // Translate UE coords to Three.js

        // Outline border
        const edges = new THREE.EdgesGeometry(geometry);
        const line = new THREE.LineSegments(edges, new THREE.LineBasicMaterial({ color: 0x00d2ff, linewidth: 2 }));
        box.add(line);

        // Add Label sprite
        const sprite = createTextSprite(place.name, '#00d2ff');
        sprite.position.set(0, extents.z + 15, 0);
        box.add(sprite);

        scene.add(box);
        threeObjects.set(place.id, box);
    });

    spec.actors.forEach((actor) => {
        const place = placeMap.get(actor.place_id);
        const base_x = place ? place.bounds.center.x : 0;
        const base_y = place ? place.bounds.center.y : 0;
        const base_z = place ? place.bounds.center.z : 0;

        const rel_pos = actor.placement.position;
        const abs_x = base_x + rel_pos.x;
        const abs_y = base_y + rel_pos.y;
        const abs_z = base_z + rel_pos.z;

        // Draw Actor: Cylinder
        const geometry = new THREE.CylinderGeometry(8, 8, 40, 16);
        const material = new THREE.MeshPhongMaterial({
            color: 0x00ffaa,
            emissive: 0x005522,
            flatShading: true
        });

        const cylinder = new THREE.Mesh(geometry, material);
        cylinder.position.set(abs_x, abs_z + 20, -abs_y); // Height offset so base sits on ground

        // Add Label sprite
        const sprite = createTextSprite(actor.name, '#00ffaa');
        sprite.position.set(0, 30, 0);
        cylinder.add(sprite);

        scene.add(cylinder);
        threeObjects.set(actor.id, cylinder);
    });

    spec.objects.forEach((obj) => {
        const place = placeMap.get(obj.place_id);
        const base_x = place ? place.bounds.center.x : 0;
        const base_y = place ? place.bounds.center.y : 0;
        const base_z = place ? place.bounds.center.z : 0;

        const rel_pos = obj.placement.position;
        const abs_x = base_x + rel_pos.x;
        const abs_y = base_y + rel_pos.y;
        const abs_z = base_z + rel_pos.z;

        // Draw Object: Sphere
        const geometry = new THREE.SphereGeometry(10, 16, 16);
        const material = new THREE.MeshPhongMaterial({
            color: 0xffaa00,
            emissive: 0x553300,
            flatShading: true
        });

        const sphere = new THREE.Mesh(geometry, material);
        sphere.position.set(abs_x, abs_z + 10, -abs_y);

        // Add Label sprite
        const sprite = createTextSprite(obj.name, '#ffaa00');
        sprite.position.set(0, 20, 0);
        sphere.add(sprite);

        scene.add(sphere);
        threeObjects.set(obj.id, sphere);
    });
}

/**
 * Selects an entity in the UI and focuses the camera on its 3D object.
 * @param {string} id - The entity ID to select.
 * @returns {void}
 */
function selectEntity(id) {
    selectedEntityId = id;

    // Highlight in lists
    document.querySelectorAll('.entity-item').forEach((item) => {
        if (item.dataset.id === id) {
            item.classList.add('selected');
        } else {
            item.classList.remove('selected');
        }
    });

    // Find entity details
    let entity = null;
    let type = '';

    if (currentSpec) {
        entity = currentSpec.places.find(p => p.id === id);
        if (entity) type = 'Place';
        
        if (!entity) {
            entity = currentSpec.actors.find(a => a.id === id);
            if (entity) type = 'Actor';
        }

        if (!entity) {
            entity = currentSpec.objects.find(o => o.id === id);
            if (entity) type = 'Object';
        }
    }

    const detailsCard = document.getElementById('entity-details');
    if (entity) {
        detailsCard.classList.remove('hidden');
        document.getElementById('details-name').innerText = entity.name;
        document.getElementById('details-id').innerText = entity.id;
        document.getElementById('details-class').innerText = type === 'Place' ? 'Physical Zone' : (entity.role || entity.class);
        
        const pos = type === 'Place' ? entity.bounds.center : entity.placement.position;
        document.getElementById('details-pos').innerText = `X: ${pos.x.toFixed(1)}, Y: ${pos.y.toFixed(1)}, Z: ${pos.z.toFixed(1)}`;
        
        const propsStr = JSON.stringify(entity.properties || {}, null, 2);
        document.getElementById('details-properties').innerText = propsStr;

        // Focus camera on the object
        const threeObj = threeObjects.get(id);
        if (threeObj) {
            const targetPos = new THREE.Vector3();
            threeObj.getWorldPosition(targetPos);
            
            // Animate controls target
            const startTarget = controls.target.clone();
            const startCam = camera.position.clone();
            
            const duration = 500; // ms
            const start = performance.now();
            
            function step(now) {
                const progress = Math.min((now - start) / duration, 1);
                // Ease out quad
                const t = progress * (2 - progress);
                
                controls.target.lerpVectors(startTarget, targetPos, t);
                renderer.render(scene, camera);
                
                if (progress < 1) {
                    requestAnimationFrame(step);
                }
            }
            requestAnimationFrame(step);
        }
    } else {
        detailsCard.classList.add('hidden');
    }
}

/**
 * Updates DOM sidebar lists and stats with the new world specification data.
 * @param {Object} spec - The world specification.
 * @returns {void}
 */
function updateDOM(spec) {
    currentSpec = spec;

    document.getElementById('place-count').innerText = spec.places.len || spec.places.length;
    document.getElementById('actor-count').innerText = spec.actors.len || spec.actors.length;
    document.getElementById('object-count').innerText = spec.objects.len || spec.objects.length;

    // Render lists
    const placesList = document.getElementById('places-list');
    placesList.innerHTML = '';
    spec.places.forEach(p => {
        const li = document.createElement('li');
        li.className = `entity-item ${p.id === selectedEntityId ? 'selected' : ''}`;
        li.dataset.id = p.id;
        li.innerHTML = `<span class="name">${p.name}</span><span class="sub">${p.id}</span><i class="fa-solid fa-cube icon"></i>`;
        li.addEventListener('click', () => selectEntity(p.id));
        placesList.appendChild(li);
    });

    const actorsList = document.getElementById('actors-list');
    actorsList.innerHTML = '';
    spec.actors.forEach(a => {
        const li = document.createElement('li');
        li.className = `entity-item ${a.id === selectedEntityId ? 'selected' : ''}`;
        li.dataset.id = a.id;
        li.innerHTML = `<span class="name">${a.name}</span><span class="sub">${a.role}</span><i class="fa-solid fa-robot icon"></i>`;
        li.addEventListener('click', () => selectEntity(a.id));
        actorsList.appendChild(li);
    });

    const objectsList = document.getElementById('objects-list');
    objectsList.innerHTML = '';
    spec.objects.forEach(o => {
        const li = document.createElement('li');
        li.className = `entity-item ${o.id === selectedEntityId ? 'selected' : ''}`;
        li.dataset.id = o.id;
        li.innerHTML = `<span class="name">${o.name}</span><span class="sub">${o.class}</span><i class="fa-solid fa-box icon"></i>`;
        li.addEventListener('click', () => selectEntity(o.id));
        objectsList.appendChild(li);
    });

    // Render receipt chain
    const receiptList = document.getElementById('receipt-list');
    receiptList.innerHTML = '';
    
    // Align receipt indexes with history events (receipts are in same order as sorted history events)
    let sortedHistory = [...spec.history];
    sortedHistory.sort((a, b) => a.timestamp_ms - b.timestamp_ms || a.id.localeCompare(b.id));

    sortedHistory.forEach((event, idx) => {
        const receipt = spec.receipts && spec.receipts[idx];
        if (!receipt) return;

        const date = new Date(event.timestamp_ms);
        const timeStr = date.toLocaleTimeString();

        const item = document.createElement('div');
        item.className = 'receipt-item';
        item.innerHTML = `
            <div class="receipt-item-header">
                <span class="receipt-item-title">${event.activity} (${event.id})</span>
                <span class="receipt-item-time">${timeStr}</span>
            </div>
            <div class="receipt-item-hash">
                <span>Hash: ${receipt.hash.substring(0, 24)}...</span>
                <i class="fa-solid fa-copy" onclick="navigator.clipboard.writeText('${receipt.hash}'); addConsoleLog('Hash copied to clipboard', 'info');"></i>
            </div>
        `;
        receiptList.appendChild(item);
    });
}

/**
 * Appends a log entry to the custom on-screen console element, formatted with
 * a timestamp and colored according to the log type.
 *
 * @param {string} message - The message text to display in the console.
 * @param {string} [type='system'] - The log type/level (e.g., 'system', 'info', 'success', 'error').
 */
function addConsoleLog(message, type = 'system') {
    const consoleEl = document.getElementById('log-console');
    const entry = document.createElement('div');
    entry.className = `log-entry ${type}`;
    entry.innerText = `[${new Date().toLocaleTimeString()}] ${message}`;
    consoleEl.appendChild(entry);
    consoleEl.scrollTop = consoleEl.scrollHeight;
}

/**
 * Asynchronously fetches the initial world specification from the `/api/spec` endpoint,
 * rebuilds the 3D world, updates the DOM UI controls, and logs the result.
 *
 * @returns {Promise<void>} A promise that resolves when the spec has been loaded and the world is rebuilt.
 */
async function loadSpec() {
    try {
        const res = await fetch('/api/spec');
        if (!res.ok) throw new Error('Failed to fetch initial spec');
        const spec = await res.json();
        
        rebuild3DWorld(spec);
        updateDOM(spec);
        addConsoleLog('World specification successfully loaded.', 'success');
    } catch (e) {
        addConsoleLog(`Failed to load world spec: ${e.message}`, 'error');
    }
}

/**
 * Asynchronously reads the natural language intent prompt from the input field,
 * disables the evolution button to show a loading spinner, triggers the world
 * evolution using the prompt, and resets/enables the inputs upon completion.
 *
 * @returns {Promise<void>} A promise that resolves when the world evolution is complete.
 */
async function evolveWorld() {
    const promptInput = document.getElementById('prompt-input');
    const prompt = promptInput.value.trim();
    if (!prompt) return;

    const evolveBtn = document.getElementById('evolve-btn');
    evolveBtn.disabled = true;
    evolveBtn.querySelector('i').className = 'fa-solid fa-spinner fa-spin';

    await evolveWorldWithPrompt(prompt);
    promptInput.value = '';

    evolveBtn.disabled = false;
    evolveBtn.querySelector('i').className = 'fa-solid fa-gears';
}

/**
 * Asynchronously sends the natural language prompt to the `/api/evolve` API endpoint to evolve the world specification.
 * Rebuilds the 3D world, updates the DOM with the new spec, and updates the BLAKE3 receipt.
 * If cooking completes and a `worldUrl` is returned, displays a success overlay and redirects the browser
 * to the Unreal Engine 4 HTML5 artifact.
 *
 * @param {string} promptText - The natural language intent/prompt describing the desired world changes.
 * @returns {Promise<void>} A promise that resolves when the world has been successfully evolved and updated.
 */
async function evolveWorldWithPrompt(promptText) {
    if (!promptText) return;
    
    addConsoleLog(`Sending intent: "${promptText.split('\n')[0]}..."`, 'info');
    
    try {
        const res = await fetch('/api/evolve', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ intent: promptText })
        });
        
        const result = await res.json();
        if (!res.ok) {
            if (result.error === "WORLD MANUFACTURED TO .t3d BUT ENGINE IS MISSING.") {
                addConsoleLog(`[UE4 FACTORY] The .t3d world artifact was successfully generated.`, 'success');
                addConsoleLog(`[UE4 FACTORY] ${result.details}`, 'error');
                return;
            }
            throw new Error(result.error || 'Server error during evolution');
        }
        
        rebuild3DWorld(result.spec);
        updateDOM(result.spec);
        addConsoleLog('World successfully updated. BLAKE3 Receipt updated.', 'success');
        
        if (result.worldUrl) {
            addConsoleLog(`[UE4 FACTORY] UNREAL ENGINE 4 HTML5 ARTIFACT COOKED!`, 'success');
            addConsoleLog(`[UE4 FACTORY] Launching playable browser world...`, 'system');
            
            // Show a massive success overlay
            const overlay = document.createElement('div');
            overlay.style.position = 'fixed';
            overlay.style.top = '0';
            overlay.style.left = '0';
            overlay.style.width = '100vw';
            overlay.style.height = '100vh';
            overlay.style.backgroundColor = 'rgba(0, 0, 0, 0.9)';
            overlay.style.color = '#00ffaa';
            overlay.style.display = 'flex';
            overlay.style.flexDirection = 'column';
            overlay.style.justifyContent = 'center';
            overlay.style.alignItems = 'center';
            overlay.style.zIndex = '9999';
            overlay.style.fontFamily = 'Outfit, sans-serif';
            overlay.innerHTML = `
                <i class="fa-solid fa-rocket" style="font-size: 5rem; margin-bottom: 2rem;"></i>
                <h1 style="font-size: 3rem; margin-bottom: 1rem;">WORLD MANUFACTURED</h1>
                <p style="font-size: 1.5rem; color: #a1a1aa;">Booting Unreal Engine 4 WebGL Artifact...</p>
            `;
            document.body.appendChild(overlay);

            setTimeout(() => {
                window.location.href = result.worldUrl;
            }, 3000);
        } else {
            if (selectedEntityId) {
                selectEntity(selectedEntityId); // Refocus details card
            }

            // If walking, ensure camera is inside a valid zone
            if (isWalkMode) {
                const currentPlace = findPlaceContaining(camera.position);
                if (!currentPlace && currentSpec && currentSpec.places.length > 0) {
                    // If out of bounds or place was deleted, teleport back to the first place
                    const p = currentSpec.places[0];
                    const center = p.bounds.center;
                    const extents = p.bounds.half_extents;
                    const floorY = center.z - extents.z;
                    camera.position.set(center.x, floorY + 18, -center.y);
                }
            }
        }
    } catch (e) {
        addConsoleLog(`Evolution failure: ${e.message}`, 'error');
    }
}

/**
 * Checks whether a given 3D position falls within the bounding box of any defined Place in the current world specification.
 * Converts between Unreal Engine coordinate conventions and Three.js coordinate conventions (Three.js -Z maps to UE Y).
 *
 * @param {THREE.Vector3} pos - The 3D position to check.
 * @returns {Object|null} The place object containing the position, or null if the position is outside all places.
 */
function findPlaceContaining(pos) {
    if (!currentSpec) return null;
    for (const place of currentSpec.places) {
        const center = place.bounds.center;
        const extents = place.bounds.half_extents;
        
        const minX = center.x - extents.x;
        const maxX = center.x + extents.x;
        const minZ = center.y - extents.y; // UE Y is Three -Z
        const maxZ = center.y + extents.y;
        
        const posValX = pos.x;
        const posValZ = -pos.z;
        
        if (posValX >= minX && posValX <= maxX && posValZ >= minZ && posValZ <= maxZ) {
            return place;
        }
    }
    return null;
}

/**
 * Activates walkthrough mode. Deselects any active selections, disables orbital camera controls,
 * teleports the camera to eye level of the first place, locks the pointer, attaches event listeners
 * for movement/look controls, and shows the walkthrough UI overlay.
 */
function enterWalkMode() {
    if (isWalkMode) {
        exitWalkMode();
        return;
    }
    
    if (!currentSpec || currentSpec.places.length === 0) {
        addConsoleLog("Cannot enter walkthrough: no places exist in the world yet.", "error");
        return;
    }
    
    // Deselect active selections
    document.getElementById('entity-details').classList.add('hidden');
    selectedEntityId = null;
    document.querySelectorAll('.entity-item').forEach(item => item.classList.remove('selected'));
    
    isWalkMode = true;
    controls.enabled = false;
    
    // Align camera with the first place floor (eye-level)
    const p = currentSpec.places[0];
    const center = p.bounds.center;
    const extents = p.bounds.half_extents;
    const floorY = center.z - extents.z;
    camera.position.set(center.x, floorY + 18, -center.y);
    
    yaw = 0;
    pitch = 0;
    const qYaw = new THREE.Quaternion().setFromAxisAngle(new THREE.Vector3(0, 1, 0), yaw);
    const qPitch = new THREE.Quaternion().setFromAxisAngle(new THREE.Vector3(1, 0, 0), pitch);
    camera.quaternion.copy(qYaw).multiply(qPitch);
    
    renderer.domElement.requestPointerLock();
    
    document.getElementById('walkthrough-overlay').classList.remove('hidden');
    document.getElementById('walk-mode-btn').classList.add('active');
    document.getElementById('walk-mode-btn').innerHTML = '<i class="fa-solid fa-person-walking"></i> Walk Mode';
    
    document.addEventListener('pointerlockchange', onPointerLockChange);
    document.addEventListener('mousemove', handleMouseMove);
    
    addConsoleLog("Entered Walkthrough Mode. Lock pointer, WASD to walk, Enter to type intent, Esc to exit.", "system");
}

/**
 * Deactivates walkthrough mode. Re-enables orbit controls, resets the camera to the default bird's eye view,
 * releases the pointer lock, removes mouse/keyboard listeners, and hides the walkthrough UI overlay.
 */
function exitWalkMode() {
    if (!isWalkMode) return;
    
    isWalkMode = false;
    controls.enabled = true;
    
    camera.position.set(150, 150, 250);
    controls.target.set(0, 0, 0);
    
    if (document.pointerLockElement === renderer.domElement) {
        document.exitPointerLock();
    }
    
    document.getElementById('walkthrough-overlay').classList.add('hidden');
    document.getElementById('walk-mode-btn').classList.remove('active');
    document.getElementById('walk-mode-btn').innerHTML = '<i class="fa-solid fa-person-walking"></i> Enter World';
    document.getElementById('hud-console-container').classList.add('hidden');
    
    document.removeEventListener('pointerlockchange', onPointerLockChange);
    document.removeEventListener('mousemove', handleMouseMove);
    
    keysPressed = { w: false, a: false, s: false, d: false };
    addConsoleLog("Exited Walkthrough Mode.", "system");
}

/**
 * Event handler for the `pointerlockchange` event. Detects if pointer lock was lost (e.g. by pressing Escape)
 * and, if the console HUD input is not focused, automatically exits walkthrough mode.
 */
function onPointerLockChange() {
    if (document.pointerLockElement !== renderer.domElement) {
        // If console HUD input is not focused, exit walk mode
        if (document.activeElement !== document.getElementById('hud-console-input')) {
            exitWalkMode();
        }
    }
}

/**
 * Event handler for the `mousemove` event when pointer lock is active.
 * Updates the camera's yaw and pitch rotation quaternions based on mouse movement inputs,
 * applying sensitivity scaling and pitch angle clamping.
 *
 * @param {MouseEvent} event - The mouse move event object containing movement deltas.
 */
function handleMouseMove(event) {
    if (document.pointerLockElement !== renderer.domElement) return;
    if (document.activeElement === document.getElementById('hud-console-input')) return;
    
    const sensitivity = 0.0025;
    yaw -= event.movementX * sensitivity;
    pitch -= event.movementY * sensitivity;
    
    const minPitch = -Math.PI / 2 + 0.05;
    const maxPitch = Math.PI / 2 - 0.05;
    pitch = Math.max(minPitch, Math.min(maxPitch, pitch));
    
    const qYaw = new THREE.Quaternion().setFromAxisAngle(new THREE.Vector3(0, 1, 0), yaw);
    const qPitch = new THREE.Quaternion().setFromAxisAngle(new THREE.Vector3(1, 0, 0), pitch);
    camera.quaternion.copy(qYaw).multiply(qPitch);
}

/**
 * Updates the player camera position each frame during walkthrough mode based on active movement keys (W, A, S, D).
 * Ensures collision detection against room bounds using `findPlaceContaining` and snaps the camera to floor height.
 */
function updateWalkthroughMovement() {
    if (!isWalkMode || !currentSpec) return;
    if (document.activeElement === document.getElementById('hud-console-input')) return;
    
    let dx = 0;
    let dz = 0;
    const walkSpeed = 2.0;
    
    if (keysPressed.w) dz -= 1;
    if (keysPressed.s) dz += 1;
    if (keysPressed.a) dx -= 1;
    if (keysPressed.d) dx += 1;
    
    if (dx !== 0 || dz !== 0) {
        const moveVector = new THREE.Vector3(dx, 0, dz);
        const qYaw = new THREE.Quaternion().setFromAxisAngle(new THREE.Vector3(0, 1, 0), yaw);
        moveVector.applyQuaternion(qYaw);
        moveVector.normalize().multiplyScalar(walkSpeed);
        
        const targetPos = camera.position.clone().add(moveVector);
        
        // Check collision against all rooms
        const place = findPlaceContaining(targetPos);
        if (place) {
            camera.position.copy(targetPos);
            // Snapping to floor height
            const floorY = place.bounds.center.z - place.bounds.half_extents.z;
            camera.position.y = floorY + 18;
            document.getElementById('current-room-name').innerText = place.name;
        } else {
            // Blocked
            const currentPlace = findPlaceContaining(camera.position);
            document.getElementById('current-room-name').innerText = `${currentPlace ? currentPlace.name : 'Outside'} (Blocked by wall)`;
        }
    } else {
        const place = findPlaceContaining(camera.position);
        document.getElementById('current-room-name').innerText = place ? place.name : "Outside";
    }
}

// Move selected actor via keyboard
let keyPressTimeout = null;
window.addEventListener('keydown', (event) => {
    // Console controls first
    if (isWalkMode) {
        // Handle HUD Console Input typing
        if (document.activeElement === document.getElementById('hud-console-input')) {
            if (event.key === 'Enter') {
                event.preventDefault();
                const val = document.getElementById('hud-console-input').value.trim();
                if (val) {
                    evolveWorldWithPrompt(val);
                }
                document.getElementById('hud-console-input').value = '';
                document.getElementById('hud-console-container').classList.add('hidden');
                // Refocus pointer lock
                renderer.domElement.requestPointerLock();
            }
            return;
        }
        
        // Active walkthrough keyboard controls
        const key = event.key.toLowerCase();
        if (key === 'w' || event.key === 'ArrowUp') keysPressed.w = true;
        if (key === 's' || event.key === 'ArrowDown') keysPressed.s = true;
        if (key === 'a' || event.key === 'ArrowLeft') keysPressed.a = true;
        if (key === 'd' || event.key === 'ArrowRight') keysPressed.d = true;
        
        // Enter console
        if (event.key === '`' || event.key === 'Enter') {
            event.preventDefault();
            // Stop movement
            keysPressed = { w: false, a: false, s: false, d: false };
            
            document.getElementById('hud-console-container').classList.remove('hidden');
            document.getElementById('hud-console-input').focus();
        }
        return;
    }
    
    // Existing selection movement
    if (!selectedEntityId || !currentSpec) return;
    if (document.activeElement === document.getElementById('prompt-input')) return;

    const actor = currentSpec.actors.find(a => a.id === selectedEntityId);
    if (!actor) return;

    let dx = 0, dy = 0;
    const speed = 5;

    if (event.key === 'w' || event.key === 'ArrowUp') dy = speed;
    if (event.key === 's' || event.key === 'ArrowDown') dy = -speed;
    if (event.key === 'a' || event.key === 'ArrowLeft') dx = -speed;
    if (event.key === 'd' || event.key === 'ArrowRight') dx = speed;

    if (dx !== 0 || dy !== 0) {
        event.preventDefault();
        
        const threeObj = threeObjects.get(actor.id);
        if (threeObj) {
            threeObj.position.x += dx;
            threeObj.position.z -= dy;
        }

        actor.placement.position.x += dx;
        actor.placement.position.y += dy;

        if (keyPressTimeout) clearTimeout(keyPressTimeout);
        keyPressTimeout = setTimeout(async () => {
            addConsoleLog(`Updating actor ${actor.id} position to (${actor.placement.position.x}, ${actor.placement.position.y}, 0)...`, 'info');
            try {
                const updatePrompt = `update actor ${actor.id} position (${actor.placement.position.x}, ${actor.placement.position.y}, ${actor.placement.position.z})`;
                const res = await fetch('/api/evolve', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ intent: updatePrompt })
                });
                const result = await res.json();
                if (res.ok) {
                    updateDOM(result.spec);
                    addConsoleLog(`Position synchronized. BLAKE3 Receipt updated.`, 'success');
                }
            } catch (e) {
                addConsoleLog(`Failed to sync position: ${e.message}`, 'error');
            }
        }, 300);
    }
});

window.addEventListener('keyup', (event) => {
    if (isWalkMode) {
        const key = event.key.toLowerCase();
        if (key === 'w' || event.key === 'ArrowUp') keysPressed.w = false;
        if (key === 's' || event.key === 'ArrowDown') keysPressed.s = false;
        if (key === 'a' || event.key === 'ArrowLeft') keysPressed.a = false;
        if (key === 'd' || event.key === 'ArrowRight') keysPressed.d = false;
    }
});

// Setup DOM Tabs & Listeners
document.addEventListener('DOMContentLoaded', () => {
    initThree();
    loadSpec();

    // Tab switcher
    document.querySelectorAll('.tab-btn').forEach((btn) => {
        btn.addEventListener('click', (e) => {
            document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
            document.querySelectorAll('.tab-pane').forEach(p => p.classList.remove('active'));
            
            btn.classList.add('active');
            const tabId = btn.dataset.tab;
            document.getElementById(tabId).classList.add('active');
        });
    });

    // Button actions
    document.getElementById('evolve-btn').addEventListener('click', evolveWorld);
    document.getElementById('walk-mode-btn').addEventListener('click', enterWalkMode);
    
    document.getElementById('reset-cam-btn').addEventListener('click', () => {
        camera.position.set(150, 150, 250);
        controls.target.set(0, 0, 0);
        addConsoleLog('Camera reset to origin.', 'system');
    });

    document.getElementById('toggle-grid-btn').addEventListener('click', () => {
        gridHelper.visible = !gridHelper.visible;
        addConsoleLog(`Grid visibility: ${gridHelper.visible ? 'on' : 'off'}`, 'system');
    });

    document.getElementById('close-details-btn').addEventListener('click', () => {
        selectedEntityId = null;
        document.getElementById('entity-details').classList.add('hidden');
        document.querySelectorAll('.entity-item').forEach(item => item.classList.remove('selected'));
    });
});
