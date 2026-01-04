# Troubleshooting Guide

Solutions for common issues with RustCamp.

## Docker Issues

### "Docker not found"

**Symptoms**: App shows "Docker not detected" error, code challenges fail.

**Solutions**:

1. **Verify Docker is installed**:
   ```bash
   docker --version
   ```
   If not found, see [Installation Guide](INSTALLATION.md#installing-docker).

2. **Start Docker service** (Linux):
   ```bash
   sudo systemctl start docker
   ```

3. **Start Docker Desktop** (Windows/macOS):
   - Open Docker Desktop from Start menu or Applications
   - Wait for it to fully start (whale icon stops animating)

4. **Add user to docker group** (Linux):
   ```bash
   sudo usermod -aG docker $USER
   # Log out and back in
   ```

### "Permission denied" when running Docker

**Symptoms**: Challenges fail with permission errors.

**Solutions**:

1. **Linux**: Add user to docker group:
   ```bash
   sudo usermod -aG docker $USER
   newgrp docker
   ```

2. **Windows**: Run Docker Desktop as Administrator once, then restart normally.

### Docker container timeouts

**Symptoms**: Challenges timeout even with correct code.

**Solutions**:

1. **Free up system resources**: Close other applications.

2. **Increase Docker resources** (Docker Desktop):
   - Settings → Resources → CPUs (at least 2)
   - Settings → Resources → Memory (at least 4 GB)

3. **Restart Docker**:
   ```bash
   # Linux
   sudo systemctl restart docker
   
   # Windows/macOS: Restart Docker Desktop
   ```

## Application Issues

### App won't start

**Symptoms**: Nothing happens when launching, or app crashes immediately.

**Solutions**:

1. **Check logs**:
   - Windows: `%APPDATA%\RustCamp\logs\`
   - macOS: `~/Library/Logs/RustCamp/`
   - Linux: `~/.local/share/RustCamp/logs/`

2. **Reset settings**:
   Delete the settings file and restart:
   - Windows: `%APPDATA%\RustCamp\settings.json`
   - macOS: `~/Library/Application Support/RustCamp/settings.json`
   - Linux: `~/.local/share/RustCamp/settings.json`

3. **Clear cache** (development builds):
   ```bash
   rm -rf node_modules/.cache
   rm -rf src-tauri/target
   npm install
   npm run tauri dev
   ```

### White/blank screen

**Symptoms**: App opens but shows blank white screen.

**Solutions**:

1. **Wait**: First load may take 10-20 seconds while content loads.

2. **Check for JavaScript errors**: Open DevTools (F12 or Cmd+Option+I).

3. **Clear browser cache** (development):
   ```bash
   # Delete and rebuild
   rm -rf dist
   npm run tauri dev
   ```

### Settings not saving

**Symptoms**: Changes revert after restart.

**Solutions**:

1. **Check write permissions** to the data directory.

2. **Close properly**: Use the X button, don't force quit.

3. **Check disk space**: Ensure sufficient free space.

## Progress & Data Issues

### Progress lost

**Symptoms**: XP, completed lessons, or badges disappeared.

**Solutions**:

1. **Check for backup**: Settings → Data Management → check for recent backups.

2. **Import backup**:
   - Settings → Data Management → Import
   - Select your `.json` backup file

3. **Database corruption**: Try resetting (warning: loses progress):
   - Settings → Data Management → Reset All Progress
   - Start fresh with a new database

### Can't import backup

**Symptoms**: Import fails or shows error.

**Solutions**:

1. **Verify file format**: Must be a valid JSON file exported from RustCamp.

2. **Check file size**: Very large files may timeout (try smaller incremental backups).

3. **Manual recovery**: Contact support with your backup file.

## Content Issues

### Lectures not loading

**Symptoms**: "Content not found" or blank lecture page.

**Solutions**:

1. **Check curriculum**: Ensure a curriculum is activated in Settings.

2. **Refresh content**:
   - Settings → Content → Refresh
   - Or restart the app

3. **Re-download curriculum**:
   - Settings → Content → Remove curriculum
   - Re-add from the curriculum manager

### Code syntax highlighting missing

**Symptoms**: Code blocks show as plain text.

**Solutions**:

1. **Check file extension**: Ensure lecture files use `.md` extension.

2. **Verify fence format**: Code blocks should use triple backticks with language:
   ~~~
   ```rust
   fn main() {}
   ```
   ~~~

## Performance Issues

### App running slowly

**Symptoms**: Laggy UI, slow navigation.

**Solutions**:

1. **Reduce animations**: Some themes have fewer animations.

2. **Close other apps**: Free up RAM and CPU.

3. **Database optimization** (advanced):
   ```bash
   sqlite3 ~/.local/share/RustCamp/rustcamp.db "VACUUM;"
   ```

### High memory usage

**Symptoms**: App uses several GB of RAM.

**Solutions**:

1. **Restart the app**: Clears accumulated memory.

2. **Check Docker**: Docker containers may be accumulating:
   ```bash
   docker system prune -f
   ```

## API Key Issues

### "Invalid API key"

**Symptoms**: Checkpoint grading fails with authentication error.

**Solutions**:

1. **Verify key**: Check that the key starts with `sk-`.

2. **Re-enter key**: Settings → API Configuration → enter key again.

3. **Check OpenAI account**: Ensure your account has API access and credits.

### Grading not working

**Symptoms**: Checkpoints don't get graded or always fail.

**Solutions**:

1. **Check API key**: Settings → API Configuration.

2. **Check internet connection**: API requires internet access.

3. **API rate limits**: Wait a few minutes if you've made many requests.

## Getting More Help

### Debug Information

When reporting issues, include:

1. **App version**: Settings → About
2. **Operating system**: Windows/macOS/Linux + version
3. **Docker version**: `docker --version`
4. **Error messages**: Copy exact text
5. **Steps to reproduce**: What you did before the error

### Log Files

Collect logs from:
- **Windows**: `%APPDATA%\RustCamp\logs\`
- **macOS**: `~/Library/Logs/RustCamp/`
- **Linux**: `~/.local/share/RustCamp/logs/`

### Contact Support

- GitHub Issues: [Repository Issues](../../issues)
- Email: support@rustcamp.dev
