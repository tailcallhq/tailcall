import {describe, test, expect} from "vitest"
import {readFile} from "fs/promises"
import {mf} from "./mf"

describe("fetch", () => {
  test("loadfiles", async () => {
    let placeholder = (await readFile("../examples/jsonplaceholder.graphql")).toString()
    let placeholder_batch = (await readFile("../examples/jsonplaceholder_batch.graphql")).toString()


    let bucket = await mf.getR2Bucket("MY_R2")
    await bucket.put("examples/jsonplaceholder.graphql", placeholder)
    await bucket.put("examples/jsonplaceholder_batch.graphql", placeholder_batch)
  })
  test("ide", async () => {
    let resp = await mf.dispatchFetch("https://fake.host/?config=examples/jsonplaceholder.graphql", {
      method: "GET",
    })
    let body = await resp.text()
    expect(body.includes("<title>Tailcall - GraphQL IDE</title>")).toBe(true)
    expect(resp.status).toBe(200)
  })

  test("sample_resp", async () => {
    let resp = await mf.dispatchFetch("https://fake.host/graphql?config=examples/jsonplaceholder.graphql", {
      method: "POST",
      body: '{"operationName":null,"variables":{},"query":"{\\n  user(id: 1) {\\n    id\\n  }\\n}\\n"}',
    })
    let body = await resp.text()
    expect(body).toBe('{"data":{"user":{"id":1}}}')
    expect(resp.status).toBe(200)
  })

  test("test_batching", async () => {
    let resp = await mf.dispatchFetch("https://fake.host/graphql?config=examples/jsonplaceholder_batch.graphql", {
      method: "POST",
      body: '{"operationName":null,"variables":{},"query":"{\\n  posts {\\n    id\\n  }\\n}\\n"}',
    })
    let body = await resp.text()
    expect(body).toBe('{"data":{"posts":[{"id":1}]}}')
    expect(resp.status).toBe(200)
  })
})
