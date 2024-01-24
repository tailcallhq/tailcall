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
  compatibilityDate: "2023-05-18",
  cache: true,
  cachePersist: false,
  d1Persist: false,
  kvPersist: false,
  r2Persist: false,
  modules: true,
  modulesRules: [{type: "CompiledWasm", include: ["**/*.wasm"], fallthrough: true}],
  bindings: {
    BUCKET: "MY_R2",
    SOME_SECRET: "secret!",
  },
  serviceBindings: {
    async remote() {
      return new Response("hello world")
    },
  },
  r2Buckets: ["MY_R2"],
  queueConsumers: {
    my_queue: {
      maxBatchTimeout: 1,
    },
  },
  queueProducers: ["my_queue", "my_queue"],
  fetchMock: mockAgent,
})
