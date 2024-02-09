;((globalThis) => {
  const {
    core: {ops},
  } = Deno

  globalThis.timerPromises = {
    async setTimeout(ms) {
      return ops.op_sleep(ms)
    },
  }
})(globalThis)
