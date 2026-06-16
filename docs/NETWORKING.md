# Networking Architecture

This document outlines the networking topology and communication patterns used across the Rocket Craft ecosystem, spanning from the game engine clients to the centralized Supabase backend and the Rust-based developer tooling.

## 1. Overview

Rocket Craft's networking architecture is divided into three primary domains:
1.  **Game Engine Networking:** UE4 HTML5 clients communicating with dedicated servers via WebSockets.
2.  **Backend Services & Real-time:** Supabase providing database access, authentication, real-time broadcasts, and Edge Functions.
3.  **Tooling & Orchestration:** Rust-based CLI tools (`rocket-cmd`) utilizing asynchronous HTTP clients to interact with remote services and infrastructure.

---

## 2. UE4 Game Networking (HTML5 / WebSockets)

Due to browser restrictions preventing raw UDP/TCP socket connections, HTML5 builds of the UE4 client must use WebSockets for multiplayer connectivity.

### Port Configuration
All HTML5 projects in this repository are standardized to communicate over port **8889**.

This is configured via `DefaultEngine.ini` using the `WebSocketNetworking` plugin:

```ini
[/Script/HTML5Networking.WebSocketNetDriver]
WebSocketPort=8889
NetConnectionClassName="/Script/HTML5Networking.WebSocketConnection"
AllowPeerConnections=False

[/Script/Engine.Engine]
NetDriverDefinitions=(DefName="GameNetDriver",DriverClassName="/Script/WebSocketNetworking.WebSocketNetDriver",DriverClassNameFallback="/Script/WebSocketNetworking.WebSocketNetDriver")
```

### Flow and WSS
1.  **Dedicated Server:** Listens on port `8889` for incoming WebSocket handshake requests.
2.  **Client:** The PWA connects to `ws://<server-ip>:8889`. Standard UE4 replication packets are wrapped in WebSocket frames.
3.  **Production WSS:** When hosted via HTTPS, browsers require secure WebSockets (`wss://`). A reverse proxy (e.g., Nginx) is utilized on the server to handle SSL termination on port 443 before proxying the traffic as standard `ws://` to the internal engine port 8889.

---

## 3. Supabase Real-time & Edge Functions

Supabase acts as the central backend, providing an API gateway, authentication, and database services.

### Real-time Interactions
The Progressive Web App (PWA) client and future microservices leverage **Supabase Realtime**.
- **Postgres Changes:** Clients can subscribe to database changes (INSERT, UPDATE, DELETE) on tables like `leaderboard` or `game_sessions` to instantly update the UI without polling.
- **Broadcasts / Presence:** Supabase Realtime channels can be used to synchronize online player presence or send ephemeral, low-latency messages directly between connected clients without persisting to the database.

### Edge Functions
For logic that requires secure, server-side execution (e.g., verifying a game match and committing a score), Rocket Craft uses **Supabase Edge Functions**.
- **Execution:** Powered by Deno, these functions run globally at the edge.
- **Location:** Managed within the repository under `supabase/functions/` (e.g., `submit-score`).
- **Invocation:** Triggered via HTTP POST requests to the Supabase API Gateway (`/functions/v1/function-name`). They validate the user's JWT Bearer token before applying business logic or database mutations using a service role key.

---

## 4. Rust Tooling (`reqwest` & `tokio`)

The `rocket-cmd` CLI and associated `rocket-sdk` libraries require network access to orchestrate deployments, fetch telemetry, or interact with Supabase APIs.

### The Stack
The Rust networking stack is built on:
- **`tokio`**: The asynchronous runtime providing thread pools, timers, and non-blocking I/O.
- **`reqwest`**: A high-level, asynchronous HTTP client used to interact with REST APIs and Edge Functions.

### Usage in the CLI
- **Async Execution:** Network bounds are strictly executed asynchronously to prevent blocking the main CLI thread, providing a responsive developer experience.
- **API Interactions:** The `reqwest` client is used to construct requests, inject required headers (like Supabase Anon Keys or Bearer tokens), handle JSON serialization/deserialization via `serde`, and manage connection pooling.
- **Robustness:** Network operations in the CLI are wrapped in comprehensive error handling and retry mechanisms to account for transient network failures during automation pipelines.

---

## 5. Dedicated CentOS Server Environment Configuration

For the dedicated server deployed on a CentOS environment, the following configuration steps must be performed to conform to WebSocket port 8889 requirements:

### 5.1 CentOS Firewall (firewalld) Configuration
Since CentOS utilizes `firewalld` by default, port `8889` must be explicitly opened for incoming TCP traffic (as WebSocket handshakes upgrade over TCP).
```bash
# Allow inbound TCP connections on port 8889 permanently
sudo firewall-cmd --zone=public --add-port=8889/tcp --permanent

# Reload firewalld to apply the changes
sudo firewall-cmd --reload

# Verify that port 8889/tcp is active
sudo firewall-cmd --list-ports
```

### 5.2 SELinux Network Policy Configuration
If a local reverse proxy (like Nginx) is configured on the same host to manage secure WebSocket (WSS) SSL termination, SELinux may block Nginx from initiating outbound network connections to port 8889. The following boolean must be set:
```bash
# Enable HTTP daemon to make network connections to backend processes
sudo setsebool -P httpd_can_network_connect 1
```

### 5.3 Systemd Daemon Service Configuration
To ensure the dedicated server runs as a persistent service that restarts automatically on failures, configure a systemd unit file at `/etc/systemd/system/rocket-server.service`:
```ini
[Unit]
Description=Rocket Craft Dedicated Server (WebSocket Port 8889)
After=network.target

[Service]
Type=simple
User=rocket
WorkingDirectory=/home/rocket/server/ShooterGame/Binaries/Linux
ExecStart=/home/rocket/server/ShooterGame/Binaries/Linux/ShooterServer -port=8889 -log
Restart=on-failure
RestartSec=5
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
```
Manage the service lifecycle using the following systemctl commands:
```bash
# Reload systemd configuration
sudo systemctl daemon-reload

# Enable service to start on boot
sudo systemctl enable rocket-server

# Start the dedicated server service
sudo systemctl start rocket-server

# Check service status
sudo systemctl status rocket-server
```

### 5.4 Nginx SSL/WSS Reverse Proxy Configuration
To handle secure browser client requests over HTTPS (`wss://`), proxy standard WebSocket connections through Nginx:
```nginx
server {
    listen 443 ssl;
    server_name game.rocketcraft.com;

    ssl_certificate /etc/letsencrypt/live/game.rocketcraft.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/game.rocketcraft.com/privkey.pem;

    location /ws {
        proxy_pass http://127.0.0.1:8889;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "Upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_read_timeout 86400s;
        proxy_send_timeout 86400s;
    }
}
```
