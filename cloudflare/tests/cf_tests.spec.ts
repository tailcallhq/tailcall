import {describe, test, expect} from "vitest"
import {readFile} from "fs/promises"
import {mf} from "./mf"

describe("fetch", () => {
  test("loadfiles", async () => {
    let placeholder = (await readFile("../examples/jsonplaceholder.graphql")).toString()
    let placeholder_batch = (await readFile("../examples/jsonplaceholder_batch.graphql")).toString()

    let bucket = await mf.getR2Bucket("MY_R2")
    // @ts-ignore
    await bucket.put("examples/jsonplaceholder.graphql", placeholder)
    // @ts-ignore
    await bucket.put("examples/jsonplaceholder_batch.graphql", placeholder_batch)
  })
  test("ide", async () => {
    let resp = await mf.dispatchFetch("https://fake.host/", {
      method: "GET",
    })
    let body = await resp.text()
    expect(body.includes("<title>Tailcall - GraphQL IDE</title>")).toBe(true)
    expect(resp.status).toBe(200)
  })

  test("sample_resp", async () => {
    let resp = await mf.dispatchFetch("https://fake.host/graphql?config=examples/jsonplaceholder.graphql", {
      method: "POST",
      body: '{"query":"{user(id: 1) {id}}"}',
    })
    let body = await resp.json()
    let expected = {data: {user: {id: 1}}}
    expect(body).toEqual(expected)
    expect(resp.status).toBe(200)
  })

  test("test_batching", async () => {
    let resp = await mf.dispatchFetch("https://fake.host/graphql?config=examples/jsonplaceholder_batch.graphql", {
      method: "POST",
      body: '{"query":"{ posts { id } }"}',
    })
    let body = await resp.json()
    let expected = {data: {posts: [{id: 1}]}}
    expect(body).toEqual(expected)
    expect(resp.status).toBe(200)
  })
})
