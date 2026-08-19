// @ts-check
// Temporary headless override for environments without a usable X display.
// Reuses the original config but forces headless mode and a lighter reporter.
const base = require('./playwright.config.js');

module.exports = {
  ...base,
  reporter: 'list',
  use: {
    ...base.use,
    headless: true,
  },
};
