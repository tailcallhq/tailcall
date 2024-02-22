import {describe, test, expect} from "vitest"
import {readFile} from "fs/promises"
import {mf} from "./mf"

describe("fetch", () => {
  test("loadfiles", async () => {
    let placeholder = (await readFile("../examples/jsonplaceholder.graphql")).toString()
    let placeholder_batch = (await readFile("../examples/jsonplaceholder_batch.graphql")).toString()
    let grpc = (await readFile("../examples/grpc.graphql")).toString()
    let news_proto = (await readFile("../src/grpc/tests/news.proto")).toString()

    let bucket = await mf.getR2Bucket("MY_R2")
    await bucket.put("examples/grpc.graphql", grpc)
    await bucket.put("examples/../src/grpc/tests/news.proto", news_proto)
    await bucket.put("src/grpc/tests/news.proto", grpc)
    await bucket.put("examples/jsonplaceholder.graphql", placeholder)
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

  test("test_grpc", async () => {
    let resp = await mf.dispatchFetch("https://fake.host/graphql?config=examples/grpc.graphql", {
      method: "POST",
      body: '{"query":"{ news { news { id } } }"}',
    })
    let body = await resp.text()
    let expected = {data: {news: {news: [{id: 1}, {id: 2}, {id: 3}, {id: 4}, {id: 5}]}}}
    expect(body).toEqual(expected)
    expect(resp.status).toBe(200)
  })
})
