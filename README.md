# Sidafa Cache

A high-performance caching server for YouTube API built with Rust, Axum, and Redis.

## Features

- YouTube API caching with Redis
- Rate limiting (100 requests/minute per IP)
- Request tracing and logging
- Systemd service integration

## Prerequisites

- Redis server
- Rust toolchain (for local development)
- Linux VPS with systemd

## Local Development

```bash
# Clone repository
git clone <repository-url>
cd sidafa-cache-rust

# Copy environment file
cp .env.example .env

# Edit .env with your configuration
nano .env

# Run the server
cargo run
```

## Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `PORT` | Server port | `3000` |
| `SERVER_HOST` | Server host | `127.0.0.1` |
| `REDIS_HOST` | Redis server host | `127.0.0.1` |
| `REDIS_PORT` | Redis server port | `6379` |
| `YOUTUBE_API_KEY` | YouTube Data API key | `AIza...` |
| `CHANNEL_ID` | YouTube channel ID | `UC...` |
| `RUST_LOG` | Log level | `info` |

---

## VPS Deployment Setup

### 1. Prepare VPS

```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install Redis
sudo apt install redis-server -y
sudo systemctl enable redis-server
sudo systemctl start redis-server

# Create application directory
sudo mkdir -p /opt/sidafa-cache
sudo chown $USER:$USER /opt/sidafa-cache
```

### 2. Create Environment File on VPS

```bash
sudo nano /opt/sidafa-cache/.env
```

Add your configuration:

```env
PORT=3000
SERVER_HOST=127.0.0.1
REDIS_HOST=127.0.0.1
REDIS_PORT=6379
YOUTUBE_API_KEY=your_youtube_api_key
CHANNEL_ID=your_channel_id
RUST_LOG=info
```

### 3. Create Systemd Service

```bash
sudo nano /etc/systemd/system/sidafa-cache.service
```

Paste the following:

```ini
[Unit]
Description=Sidafa Cache Server
After=network.target redis-server.service
Wants=redis-server.service

[Service]
Type=simple
User=root
Group=root
WorkingDirectory=/opt/sidafa-cache
ExecStart=/opt/sidafa-cache/sidafa-cache
Restart=always
RestartSec=5
Environment=RUST_LOG=info
EnvironmentFile=/opt/sidafa-cache/.env

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/sidafa-cache

[Install]
WantedBy=multi-user.target
```

Enable the service:

```bash
sudo systemctl daemon-reload
sudo systemctl enable sidafa-cache
```

### 4. Setup SSH Key for GitHub Actions

On your local machine, generate a deploy key:

```bash
ssh-keygen -t ed25519 -C "github-actions-deploy" -f ~/.ssh/sidafa_deploy_key
```

Copy public key to VPS:

```bash
ssh-copy-id -i ~/.ssh/sidafa_deploy_key.pub user@your-vps-ip
```

### 5. Configure GitHub Secrets

Go to your repository: **Settings > Secrets and variables > Actions**

Add the following secrets:

| Secret Name | Description | Example |
|-------------|-------------|---------|
| `VPS_HOST` | VPS IP address or hostname | `123.45.67.89` |
| `VPS_USER` | SSH username | `root` |
| `VPS_SSH_KEY` | Private SSH key content | `-----BEGIN OPENSSH PRIVATE KEY-----...` |
| `VPS_DEPLOY_PATH` | Deployment directory | `/opt/sidafa-cache` |

To get the SSH key content:

```bash
cat ~/.ssh/sidafa_deploy_key
```

### 6. Setup Nginx Reverse Proxy (Optional)

```bash
sudo apt install nginx -y
sudo nano /etc/nginx/sites-available/sidafa-cache
```

```nginx
server {
    listen 80;
    server_name your-domain.com;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
    }
}
```

Enable the site:

```bash
sudo ln -s /etc/nginx/sites-available/sidafa-cache /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

### 7. Setup SSL with Certbot (Optional)

```bash
sudo apt install certbot python3-certbot-nginx -y
sudo certbot --nginx -d your-domain.com
```

---

## Deployment

Deployment is automatic when you push to the `main` branch. You can also trigger it manually from the Actions tab.

### Manual Deployment

1. Go to repository **Actions** tab
2. Select **Build and Deploy to VPS** workflow
3. Click **Run workflow**

### Monitor Deployment

```bash
# Check service status
sudo systemctl status sidafa-cache

# View logs
sudo journalctl -u sidafa-cache -f

# Restart service
sudo systemctl restart sidafa-cache
```

---

## Troubleshooting

### Service fails to start

```bash
# Check logs for errors
sudo journalctl -u sidafa-cache -n 50 --no-pager

# Verify binary permissions
ls -la /opt/sidafa-cache/sidafa-cache

# Test binary manually
cd /opt/sidafa-cache && ./sidafa-cache
```

### Redis connection issues

```bash
# Check Redis status
sudo systemctl status redis-server

# Test Redis connection
redis-cli ping
```

### SSH connection issues

```bash
# Test SSH from GitHub Actions runner
ssh -i ~/.ssh/sidafa_deploy_key -o StrictHostKeyChecking=no user@vps-ip "echo 'Connection successful'"
```

## License

MIT
