function argsToMessage(...args) {
  return args.map((arg) => JSON.stringify(arg)).join(" ")
}

const console = {
  log(...args) {
    globalThis.__qjs_print(`${argsToMessage(...args)}\n`, false)
  },
  error(...args) {
    globalThis.__qjs_print(`${argsToMessage(...args)}\n`, true)
  },
}

globalThis.console = console
