const http = require('http');
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const PORT = 3000;
const WEB_DIR = path.join(__dirname, 'genie-web');
const SPEC_PATH = path.join(__dirname, 'spec.json');
const MAP_PATH = path.join(__dirname, 'map.t3d');
const LOG_PATH = path.join(__dirname, 'deploy.log');
const UNIFY_BIN = path.join(__dirname, 'unify-rs', 'target', 'debug', 'unify');

// Ensure a default spec exists
function initDefaultSpec() {
    if (!fs.existsSync(SPEC_PATH)) {
        console.log("Initializing default world specification...");
        const defaultIntent = [
            "create place room_1 name \"Control Room\" at (0.0, 0.0, 0.0) bounds (100.0, 100.0, 50.0)",
            "create actor bot_1 name \"Welder Bot\" role RoboticWelder in room_1",
            "create object cnc_1 name \"CNC Alpha\" class CNC_Machine in room_1"
        ].join("\n");
        
        const tempIntentPath = path.join(__dirname, 'default_intent.txt');
        fs.writeFileSync(tempIntentPath, defaultIntent);

        try {
            // Manufacture the initial spec using the CLI
            const cmd = `"${UNIFY_BIN}" genie manufacture --intent "${tempIntentPath}" --out-spec "${SPEC_PATH}" --out-t3d "${MAP_PATH}"`;
            console.log(`Running: ${cmd}`);
            execSync(cmd);
            
            // Also run deploy to log it
            const deployCmd = `"${UNIFY_BIN}" genie deploy --spec "${SPEC_PATH}" --log "${LOG_PATH}"`;
            console.log(`Running: ${deployCmd}`);
            execSync(deployCmd);
        } catch (e) {
            console.error("Failed to compile default spec via CLI:", e.message);
            // Fallback mock spec if compiler isn't built yet
            const fallbackSpec = {
                places: [{ id: "room_1", name: "Control Room", bounds: { center: { x: 0, y: 0, z: 0 }, half_extents: { x: 100, y: 100, z: 50 } }, properties: {} }],
                actors: [{ id: "bot_1", name: "Welder Bot", role: "RoboticWelder", place_id: "room_1", placement: { position: { x: 0, y: 0, z: 0 }, rotation: { x: 0, y: 0, z: 0 } }, properties: {} }],
                objects: [{ id: "cnc_1", name: "CNC Alpha", class: "CNC_Machine", place_id: "room_1", placement: { position: { x: 0, y: 0, z: 0 }, rotation: { x: 0, y: 0, z: 0 } }, properties: {} }],
                relationships: [],
                rules: [],
                history: [{ id: "evt_init", timestamp_ms: Date.now(), activity: "Boot", details: {} }],
                receipts: [{ key: "history_receipt_evt_init", hash: "0000000000000000000000000000000000000000000000000000000000000000", issued_at: Date.now() }]
            };
            fs.writeFileSync(SPEC_PATH, JSON.stringify(fallbackSpec, null, 2));
        } finally {
            if (fs.existsSync(tempIntentPath)) {
                fs.unlinkSync(tempIntentPath);
            }
        }
    }
}

// Serve static files
function serveStatic(req, res) {
    let filePath;
    if (req.url.startsWith('/manufactured/')) {
        filePath = path.join(__dirname, 'pwa-staff', req.url);
    } else {
        const pwaPath = path.join(__dirname, 'pwa-staff', req.url === '/' ? 'index.html' : req.url);
        if (fs.existsSync(pwaPath) && !fs.statSync(pwaPath).isDirectory()) {
            filePath = pwaPath;
        } else {
            filePath = path.join(WEB_DIR, req.url === '/' ? 'index.html' : req.url);
        }
    }
    
    const extname = path.extname(filePath);
    
    let contentType = 'text/html';
    switch (extname) {
        case '.js':
            contentType = 'text/javascript';
            break;
        case '.css':
            contentType = 'text/css';
            break;
        case '.json':
            contentType = 'application/json';
            break;
        case '.png':
            contentType = 'image/png';
            break;
        case '.wasm':
            contentType = 'application/wasm';
            break;
        case '.data':
            contentType = 'application/octet-stream';
            break;
    }

    fs.readFile(filePath, (err, content) => {
        if (err) {
            if (err.code === 'ENOENT') {
                res.writeHead(404, { 'Content-Type': 'text/html' });
                res.end('<h1>404 Not Found</h1>', 'utf-8');
            } else {
                res.writeHead(500);
                res.end(`Server Error: ${err.code}`);
            }
        } else {
            res.writeHead(200, { 'Content-Type': contentType });
            res.end(content);
        }
    });
}

// Semantic Natural Language Layout Compiler
function compilePromptToIntent(prompt) {
    const trimmed = prompt.trim();
    const lines = trimmed.split('\n').map(l => l.trim()).filter(l => l.length > 0 && !l.startsWith('#'));
    
    // Check if it's already raw DSL commands
    const isRaw = lines.every(line => 
        line.startsWith('create ') || 
        line.startsWith('update ') || 
        line.startsWith('delete ')
    );
    if (isRaw && lines.length > 0) {
        return { isRaw: true, dsl: trimmed, isNewWorld: false };
    }

    console.log("Natural language prompt received: " + trimmed);

    const lower = trimmed.toLowerCase();
    
    // Check if it is a whole new world request vs incremental update
    const isNewWorld = lower.includes("create a") || lower.includes("manufacture a") || lower.includes("operations center") || lower.includes("factory") || lower.includes("facility center") || (!lower.includes("add ") && !lower.includes("delete ") && !lower.includes("update "));

    if (isNewWorld) {
        let placesList = [];
        const withMatch = trimmed.match(/with\s+([^.]+)/i);
        if (withMatch) {
            const listStr = withMatch[1].replace(/\band\b/g, ',');
            placesList = listStr.split(',')
                .map(s => s.trim().replace(/(facilities|facility|rooms|room|areas|area|offices|office|bays|bay)$/i, '').trim())
                .filter(s => s.length > 0);
        }

        if (placesList.length === 0) {
            const words = trimmed.match(/\b[a-zA-Z]{3,15}\b/g) || [];
            const ignored = ['create', 'with', 'and', 'operations', 'center', 'facilities', 'facility', 'rooms', 'room', 'areas', 'area', 'offices', 'office', 'bays', 'bay', 'manufacture'];
            placesList = words.filter(w => !ignored.includes(w.toLowerCase()));
            placesList = [...new Set(placesList)].slice(0, 5);
        }

        if (placesList.length === 0) {
            placesList = ["Control Room", "Storage Area"];
        }

        let dsl = [];
        let rooms = [];
        const spacing = 400.0;
        const roomWidth = 150.0;
        const roomLength = 150.0;
        const roomHeight = 50.0;

        placesList.forEach((placeName, index) => {
            const cleanName = placeName.trim();
            const cleanId = cleanName.toLowerCase().replace(/[^a-z0-9_]/g, '_').replace(/^_+|_+$/g, '');
            const id = cleanId || `room_${index + 1}`;
            
            const gridX = (index % 2) * spacing;
            const gridY = Math.floor(index / 2) * spacing;
            
            dsl.push(`create place ${id} name "${cleanName}" at (${gridX.toFixed(1)}, ${gridY.toFixed(1)}, 0.0) bounds (${roomWidth.toFixed(1)}, ${roomLength.toFixed(1)}, ${roomHeight.toFixed(1)})`);
            rooms.push({ id, name: cleanName, x: gridX, y: gridY });
        });

        // Add connecting relationships
        for (let i = 0; i < rooms.length; i++) {
            const r1 = rooms[i];
            const r2 = rooms[(i + 1) % rooms.length];
            if (r1.id !== r2.id) {
                dsl.push(`create relationship rel_${r1.id}_to_${r2.id} connects from ${r1.id} to ${r2.id}`);
            }
        }

        // Add Actors and Objects to each room
        rooms.forEach((room) => {
            const id = room.id;
            const name = room.name.toLowerCase();
            
            let actorRole = "SupervisorBot";
            let actorName = `${room.name} Supervisor`;
            let objectClass = "ControlTerminal";
            let objectName = `${room.name} Terminal`;

            if (name.includes("dispatch")) {
                actorRole = "Dispatcher";
                actorName = "Dispatcher Bot";
                objectClass = "DispatchConsole";
                objectName = "Dispatch Console";
            } else if (name.includes("maintenance") || name.includes("repair") || name.includes("bay")) {
                actorRole = "Mechanic";
                actorName = "Mechanic Bot";
                objectClass = "TruckLift";
                objectName = "Hydraulic Truck Lift";
            } else if (name.includes("finance") || name.includes("billing") || name.includes("office")) {
                actorRole = "Accountant";
                actorName = "Financial Officer";
                objectClass = "SecureVault";
                objectName = "Cash Vault";
            } else if (name.includes("training") || name.includes("driver") || name.includes("school")) {
                actorRole = "Instructor";
                actorName = "Training Instructor";
                objectClass = "TruckSimulator";
                objectName = "Drive Sim Cabinet";
            } else if (name.includes("storage") || name.includes("warehouse")) {
                actorRole = "ForkliftDriver";
                actorName = "Logistics Welder";
                objectClass = "CargoPallet";
                objectName = "Heavy Pallet Stack";
            }

            const actorId = `bot_${id}`;
            const objectId = `prop_${id}`;

            dsl.push(`create actor ${actorId} name "${actorName}" role ${actorRole} in ${id}`);
            dsl.push(`create object ${objectId} name "${objectName}" class ${objectClass} in ${id}`);
            dsl.push(`create relationship rel_contains_actor_${id} contains from ${id} to ${actorId}`);
            dsl.push(`create relationship rel_contains_object_${id} contains from ${id} to ${objectId}`);
        });

        return { isRaw: false, dsl: dsl.join('\n'), isNewWorld: true };
    } else {
        // Incremental NL changes
        let dsl = [];
        
        if (lower.startsWith("add ") || lower.startsWith("create ")) {
            if (lower.includes("room") || lower.includes("bay") || lower.includes("office") || lower.includes("facility") || lower.includes("space") || lower.includes("place")) {
                const cleanName = trimmed.replace(/^(add a|add|create a|create)\s+/i, '').replace(/\s+(room|bay|office|facility|space|place)$/i, '').trim();
                const id = cleanName.toLowerCase().replace(/[^a-z0-9_]/g, '_');
                
                let posX = 200.0;
                let posY = 200.0;
                try {
                    if (fs.existsSync(SPEC_PATH)) {
                        const currentSpecData = JSON.parse(fs.readFileSync(SPEC_PATH, 'utf-8'));
                        if (currentSpecData.places && currentSpecData.places.length > 0) {
                            const lastPlace = currentSpecData.places[currentSpecData.places.length - 1];
                            posX = lastPlace.bounds.center.x + 400.0;
                            posY = lastPlace.bounds.center.y;
                        }
                    }
                } catch(err) {
                    console.error("Failed to read existing spec for placement:", err);
                }

                dsl.push(`create place ${id} name "${cleanName} Room" at (${posX.toFixed(1)}, ${posY.toFixed(1)}, 0.0) bounds (150.0, 150.0, 50.0)`);
            } else if (lower.includes("bot") || lower.includes("actor") || lower.includes("worker") || lower.includes("mechanic") || lower.includes("dispatcher")) {
                const actorRole = lower.includes("mechanic") ? "Mechanic" : (lower.includes("dispatcher") ? "Dispatcher" : "SupervisorBot");
                const actorName = lower.includes("mechanic") ? "Mechanic Bot" : (lower.includes("dispatcher") ? "Dispatcher Bot" : "Assistant Bot");
                
                let targetPlaceId = "room_1";
                const inMatch = lower.match(/(?:in|to)\s+(\S+)/);
                if (inMatch) {
                    targetPlaceId = inMatch[1].replace(/[^a-z0-9_]/g, '');
                } else {
                    try {
                        if (fs.existsSync(SPEC_PATH)) {
                            const specJson = JSON.parse(fs.readFileSync(SPEC_PATH, 'utf-8'));
                            if (specJson.places && specJson.places.length > 0) {
                                targetPlaceId = specJson.places[0].id;
                            }
                        }
                    } catch(e) {}
                }
                const randomId = "bot_" + Math.random().toString(36).substring(2, 6);
                dsl.push(`create actor ${randomId} name "${actorName}" role ${actorRole} in ${targetPlaceId}`);
            } else {
                const objClass = "ControlTerminal";
                const objName = "Utility Console";
                let targetPlaceId = "room_1";
                const inMatch = lower.match(/(?:in|to)\s+(\S+)/);
                if (inMatch) {
                    targetPlaceId = inMatch[1].replace(/[^a-z0-9_]/g, '');
                }
                const randomId = "prop_" + Math.random().toString(36).substring(2, 6);
                dsl.push(`create object ${randomId} name "${objName}" class ${objClass} in ${targetPlaceId}`);
            }
        } else if (lower.startsWith("delete ") || lower.startsWith("remove ")) {
            const idToDelete = trimmed.replace(/^(delete|remove)\s+/i, '').trim();
            dsl.push(`delete ${idToDelete}`);
        } else if (lower.startsWith("move ") || lower.startsWith("update ")) {
            const idToUpdate = trimmed.match(/^(?:move|update)\s+(\S+)/i)?.[1] || "";
            const coordsMatch = trimmed.match(/\(\s*([-\d.]+)\s*,\s*([-\d.]+)\s*\)/);
            if (idToUpdate && coordsMatch) {
                const x = parseFloat(coordsMatch[1]);
                const y = parseFloat(coordsMatch[2]);
                dsl.push(`update actor ${idToUpdate} position (${x.toFixed(1)}, ${y.toFixed(1)}, 0.0)`);
            } else {
                return { isRaw: true, dsl: trimmed, isNewWorld: false };
            }
        } else {
            return { isRaw: true, dsl: trimmed, isNewWorld: false };
        }

        return { isRaw: false, dsl: dsl.join('\n'), isNewWorld: false };
    }
}

// Start Server
const server = http.createServer((req, res) => {
    // API GET /api/spec
    if (req.method === 'GET' && req.url === '/api/spec') {
        fs.readFile(SPEC_PATH, 'utf-8', (err, data) => {
            if (err) {
                res.writeHead(500, { 'Content-Type': 'application/json' });
                res.end(JSON.stringify({ error: "Failed to read spec file" }));
            } else {
                res.writeHead(200, { 'Content-Type': 'application/json' });
                res.end(data);
            }
        });
    }
    // API POST /api/evolve
    else if (req.method === 'POST' && req.url === '/api/evolve') {
        let body = '';
        req.on('data', chunk => {
            body += chunk.toString();
        });
        req.on('end', () => {
            try {
                const { intent } = JSON.parse(body);
                if (!intent) {
                    res.writeHead(400, { 'Content-Type': 'application/json' });
                    return res.end(JSON.stringify({ error: "Missing 'intent' in request body" }));
                }

                // Compile natural language to Genie DSL
                const compiled = compilePromptToIntent(intent);
                console.log(`Compiled DSL output:\n${compiled.dsl}`);

                // Write intent to a temporary file
                const tempIntentPath = path.join(__dirname, 'evolve_intent.txt');
                fs.writeFileSync(tempIntentPath, compiled.dsl);

                // Determine whether to run fresh manufacture or incremental evolve
                let cmd;
                if (compiled.isNewWorld) {
                    cmd = `"${UNIFY_BIN}" genie manufacture --intent "${tempIntentPath}" --out-spec "${SPEC_PATH}" --out-t3d "${MAP_PATH}"`;
                } else {
                    cmd = `"${UNIFY_BIN}" genie evolve --spec "${SPEC_PATH}" --intent "${tempIntentPath}" --out-spec "${SPEC_PATH}" --out-t3d "${MAP_PATH}"`;
                }
                
                console.log(`Executing layout command: ${cmd}`);
                execSync(cmd);
                
                // Deploy to update log and run UE4 pipeline
                try {
                    const deployCmd = `"${UNIFY_BIN}" genie deploy --spec "${SPEC_PATH}" --log "${LOG_PATH}"`;
                    execSync(deployCmd, { stdio: 'pipe' });
                } catch (deployErr) {
                    // Check if it's the UE4_ROOT missing error
                    const output = deployErr.stdout ? deployErr.stdout.toString() : '';
                    if (output.includes("UE4_ROOT environment variable is not set") || (deployErr.stderr && deployErr.stderr.toString().includes("UE4_ROOT"))) {
                        console.error("UE4 Pipeline requires engine installation.");
                        // Clean up temp file
                        if (fs.existsSync(tempIntentPath)) {
                            fs.unlinkSync(tempIntentPath);
                        }
                        res.writeHead(500, { 'Content-Type': 'application/json' });
                        return res.end(JSON.stringify({ 
                            error: "WORLD MANUFACTURED TO .t3d BUT ENGINE IS MISSING.",
                            details: "The .t3d artifact was generated, but the UE4 HTML5 Cook Pipeline requires 'UE4_ROOT' environment variable to be set to a valid Unreal Engine 4.24 installation path."
                        }));
                    } else {
                        throw deployErr; // Rethrow to be caught by outer catch
                    }
                }

                // Clean up temp file
                if (fs.existsSync(tempIntentPath)) {
                    fs.unlinkSync(tempIntentPath);
                }

                // Read and return the updated spec
                const updatedSpec = fs.readFileSync(SPEC_PATH, 'utf-8');
                res.writeHead(200, { 'Content-Type': 'application/json' });
                res.end(JSON.stringify({
                    message: compiled.isNewWorld ? "Manufacturing successful" : "Evolution successful",
                    spec: JSON.parse(updatedSpec),
                    worldUrl: "/manufactured/Brm-HTML5-Shipping.html"
                }));
            } catch (e) {
                console.error("Layout generation failed:", e);
                const stderr = e.stderr ? e.stderr.toString() : e.message;
                res.writeHead(500, { 'Content-Type': 'application/json' });
                res.end(JSON.stringify({ error: stderr || "Execution error during layout generation" }));
            }
        });
    }
    // Static Files
    else {
        serveStatic(req, res);
    }
});

initDefaultSpec();

server.listen(PORT, () => {
    console.log(`=======================================================`);
    console.log(`  Genie 26 World Operating Center online!`);
    console.log(`  URL: http://localhost:${PORT}`);
    console.log(`=======================================================`);
});
