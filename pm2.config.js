module.exports = {
  name: "app", // Name of your application
  script: "server.bun.js", // Entry point of your application
  interpreter: "bun", // Path to the Bun interpreter
  instances: 14,
  exec_mode: "fork"
};