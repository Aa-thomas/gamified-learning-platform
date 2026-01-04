# Installation Guide

Complete guide for installing RustCamp on your system.

## System Requirements

### Minimum Requirements
- **OS**: Windows 10+, macOS 10.15+, or Linux (Ubuntu 20.04+, Fedora 34+)
- **RAM**: 4 GB
- **Storage**: 500 MB free space
- **Display**: 1280x720 resolution

### Recommended
- **RAM**: 8 GB
- **Storage**: 2 GB free space (for Docker images)
- **Display**: 1920x1080 resolution

## Installing Docker

Docker is required for running code challenges.

### Windows

1. Download [Docker Desktop for Windows](https://www.docker.com/products/docker-desktop)
2. Run the installer
3. Follow the setup wizard
4. Enable WSL 2 if prompted
5. Restart your computer
6. Start Docker Desktop from the Start menu

Verify installation:
```powershell
docker --version
docker run hello-world
```

### macOS

1. Download [Docker Desktop for Mac](https://www.docker.com/products/docker-desktop)
2. Drag Docker.app to Applications folder
3. Open Docker from Applications
4. Grant permissions when prompted
5. Wait for Docker to start (whale icon in menu bar)

Verify installation:
```bash
docker --version
docker run hello-world
```

### Linux (Ubuntu/Debian)

```bash
# Update packages
sudo apt-get update

# Install Docker
sudo apt-get install docker.io

# Start Docker service
sudo systemctl start docker
sudo systemctl enable docker

# Add user to docker group (avoids needing sudo)
sudo usermod -aG docker $USER

# Log out and back in for group changes to take effect
```

### Linux (Fedora)

```bash
# Install Docker
sudo dnf install docker

# Start Docker service
sudo systemctl start docker
sudo systemctl enable docker

# Add user to docker group
sudo usermod -aG docker $USER

# Log out and back in
```

### Linux (Arch)

```bash
# Install Docker
sudo pacman -S docker

# Start Docker service
sudo systemctl start docker
sudo systemctl enable docker

# Add user to docker group
sudo usermod -aG docker $USER

# Log out and back in
```

## Installing RustCamp

### Windows

1. Download `RustCamp_x.x.x_x64-setup.exe` from Releases
2. Double-click the installer
3. Follow the installation wizard
4. Launch RustCamp from Start menu or desktop shortcut

### macOS

1. Download `RustCamp_x.x.x_x64.dmg` from Releases
2. Open the DMG file
3. Drag RustCamp to Applications folder
4. First launch: Right-click → Open (to bypass Gatekeeper)
5. Grant permissions when prompted

### Linux (Debian/Ubuntu)

```bash
# Download and install
sudo dpkg -i rustcamp_x.x.x_amd64.deb

# If dependencies are missing
sudo apt-get install -f
```

### Linux (Fedora/RHEL)

```bash
sudo rpm -i rustcamp-x.x.x-1.x86_64.rpm
```

### Linux (AppImage)

```bash
# Make executable
chmod +x RustCamp_x.x.x_amd64.AppImage

# Run
./RustCamp_x.x.x_amd64.AppImage
```

## Building from Source

### Prerequisites

- Rust 1.75+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- Node.js 18+ (`nvm install 18` or download from nodejs.org)
- Tauri CLI prerequisites (see [Tauri docs](https://tauri.app/v1/guides/getting-started/prerequisites))

### Build Steps

```bash
# Clone repository
git clone https://github.com/your-org/gamified-learning-platform.git
cd gamified-learning-platform/apps/desktop

# Install Node dependencies
npm install

# Development mode
npm run tauri dev

# Build release
npm run tauri build
```

Build output will be in `src-tauri/target/release/bundle/`.

## Post-Installation Setup

1. **Launch RustCamp** - The onboarding wizard will guide you through setup
2. **Docker Check** - The app will verify Docker is running
3. **API Key** (optional) - Add your OpenAI API key for checkpoint grading
4. **Start Learning** - Begin with the first lecture!

## Updating

### Automatic Updates (Recommended)
RustCamp checks for updates on launch. When an update is available, click "Update Now" to download and install.

### Manual Updates
Download the latest release and install over the existing version. Your progress is preserved.

## Uninstalling

### Windows
Settings → Apps → RustCamp → Uninstall

### macOS
Drag RustCamp.app to Trash

### Linux
```bash
# Debian/Ubuntu
sudo apt-get remove rustcamp

# Fedora
sudo dnf remove rustcamp
```

## Data Location

User data is stored in platform-specific locations:

- **Windows**: `%APPDATA%\RustCamp\`
- **macOS**: `~/Library/Application Support/RustCamp/`
- **Linux**: `~/.local/share/RustCamp/`

This includes:
- `rustcamp.db` - SQLite database with progress
- `settings.json` - Application settings
- `content/` - Downloaded curriculum content
