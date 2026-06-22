// @ts-check
const { spawn } = require('child_process');
const { join } = require('path');
const http = require('http');
const fs = require('fs');

const SERVER_BIN = join(__dirname, '..', 'target', 'release', 'stockrate');
const CREDENTIALS_FILE = join(__dirname, '..', 'resources', 'credentials.txt');
const CREDENTIALS_BACKUP = join(__dirname, 'credentials_backup.txt');

module.exports = async () => {
  // Backup credentials and disable Finnhub for tests
  if (fs.existsSync(CREDENTIALS_FILE)) {
    fs.copyFileSync(CREDENTIALS_FILE, CREDENTIALS_BACKUP);
    fs.writeFileSync(CREDENTIALS_FILE, '# FINNHUB_API_KEY=disabled\n');
  }
  
  const server = spawn(SERVER_BIN, [], { stdio: ['ignore', 'ignore', 'pipe'] });
  
  server.stderr.on('data', (data) => {
    // Suppress stderr
  });
  
  server.on('error', (err) => {
    // Restore credentials on error
    restoreCredentials();
    throw new Error(`Failed to start server: ${err.message}`);
  });

  server.on('close', (code) => {
    // Suppress - teardown kills the server
  });
  
  // Write PID to file for teardown
  fs.writeFileSync(join(__dirname, 'server.pid'), server.pid.toString());
  
  // Wait for server to be ready
  await new Promise((resolve, reject) => {
    let started = false;
    const timeout = setTimeout(() => {
      if (!started) {
        server.kill('SIGTERM');
        restoreCredentials();
        reject(new Error('Timed out waiting for server'));
      }
    }, 30000);
    
    function tryConnect() {
      if (started) return;
      http.get('http://127.0.0.1:3000/', (res) => {
        started = true;
        clearTimeout(timeout);
        res.resume();
        res.on('end', () => resolve());
      }).on('error', () => {
        setTimeout(tryConnect, 500);
      });
    }
    
    setTimeout(tryConnect, 1000);
  });
};

function restoreCredentials() {
  if (fs.existsSync(CREDENTIALS_BACKUP)) {
    fs.copyFileSync(CREDENTIALS_BACKUP, CREDENTIALS_FILE);
    fs.unlinkSync(CREDENTIALS_BACKUP);
  }
}
