const {core} = Deno

function argsToMessage(...args) {
  return args.map((arg) => JSON.stringify(arg)).join(" ")
}

const console = {
  log: (...args) => {
    core.print(`[out]: ${argsToMessage(...args)}\n`, false)
  },
  error: (...args) => {
    core.print(`[err]: ${argsToMessage(...args)}\n`, true)
  },
}

globalThis.console = console
