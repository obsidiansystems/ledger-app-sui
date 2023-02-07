module.exports = {
  timeout: process.env.LEDGER_LIVE_HARDWARE ? 0 : 16 * 1000
};
console.log("Config file loaded.");
console.log(module.exports);
