import {Miniflare, Response} from "miniflare"
import {MockAgent} from "undici"

const mockAgent = new MockAgent()

mockAgent.get("https://cloudflare.com").intercept({path: "/"}).reply(200, "cloudflare!")

mockAgent
  .get("http://jsonplaceholder.typicode.com")
  .intercept({path: "/posts"})
  .reply(
    200,
    [
      {
        id: 1,
        name: "Alo",
        username: "alo",
        email: "alo@alo.com",
      },
    ],
    {
      headers: {
        "content-type": "application/json",
      },
    },
  )

export const mf = new Miniflare({
  scriptPath: "./build/worker/shim.mjs",
  cache: true,
  modules: true,
  modulesRules: [{type: "CompiledWasm", include: ["**/*.wasm"], fallthrough: true}],
  bindings: {BUCKET: "MY_R2"},
  r2Buckets: ["MY_R2"],
  fetchMock: mockAgent,
})
