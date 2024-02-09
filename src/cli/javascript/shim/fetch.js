;((globalThis) => {
  const {
    core: {ops},
  } = Deno

  class Response {
    constructor(response) {
      this.response = response
    }

    async text() {
      // TODO: use TextDecoder to optimize conversion
      return String.fromCharCode.apply(null, new Uint8Array(this.response.body))
    }

    async json() {
      return JSON.parse(await this.text())
    }
  }

  globalThis.fetch = async function (url) {
    let response = await ops.op_fetch(url)

    return new Response(response)
  }
})(globalThis)
