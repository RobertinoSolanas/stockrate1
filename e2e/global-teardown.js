// @ts-check
const fs = require('fs');
const { join } = require('path');
const { execSync } = require('child_process');

const CREDENTIALS_BACKUP = join(__dirname, 'credentials_backup.txt');
const CREDENTIALS_FILE = join(__dirname, '..', 'resources', 'credentials.txt');

module.exports = () => {
  // Kill server if running
  const pidFile = join(__dirname, 'server.pid');
  if (fs.existsSync(pidFile)) {
    const pid = fs.readFileSync(pidFile, 'utf-8').trim();
    try {
      execSync(`kill ${pid}`, { stdio: 'ignore' });
    } catch (e) {
      // Process may already be dead
    }
    fs.unlinkSync(pidFile);
  }
  
  // Restore credentials
  if (fs.existsSync(CREDENTIALS_BACKUP)) {
    fs.copyFileSync(CREDENTIALS_BACKUP, CREDENTIALS_FILE);
    fs.unlinkSync(CREDENTIALS_BACKUP);
  }
};
