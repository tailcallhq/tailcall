import {Miniflare, Response} from "miniflare"
import {MockAgent} from "undici"

const mockAgent = new MockAgent()

mockAgent.get("https://cloudflare.com").intercept({path: "/"}).reply(200, "cloudflare!")
mockAgent.get("http://localhost:50051").intercept({path: "/NewsService/GetAllNews"})
  .reply(200, [0, 0, 0, 0, 185, 10, 35, 8, 1, 18, 6, 78, 111, 116, 101, 32, 49, 26, 9, 67, 111, 110, 116, 101, 110, 116, 32, 49, 34, 12, 80, 111, 115, 116, 32, 105, 109, 97,
    103, 101, 32, 49, 10, 35, 8, 2, 18, 6, 78, 111, 116, 101, 32, 50, 26, 9, 67, 111, 110, 116, 101, 110, 116, 32, 50, 34, 12, 80, 111, 115, 116, 32, 105,
    109, 97, 103, 101, 32, 50, 10, 35, 8, 3, 18, 6, 78, 111, 116, 101, 32, 51, 26, 9, 67, 111, 110, 116, 101, 110, 116, 32, 51, 34, 12, 80, 111, 115, 116,
    32, 105, 109, 97, 103, 101, 32, 51, 10, 35, 8, 4, 18, 6, 78, 111
    , 116, 101, 32, 52, 26, 9, 67, 111, 110, 116, 101, 110, 116, 32, 52, 34, 12, 80, 111, 115, 116, 32, 105, 109, 97,
    103, 101, 32, 52, 10, 35, 8, 5, 18, 6, 78, 111, 116, 101, 32, 53, 26, 9, 67, 111, 110, 116, 101, 110, 116, 32, 53, 34, 12, 80,
    111, 115, 116, 32, 105, 109, 97, 103, 101, 32, 53])

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
