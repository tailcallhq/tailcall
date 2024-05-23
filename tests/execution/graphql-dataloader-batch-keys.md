---
skip: true
---

# GraphQL datasource

[//]: # "nested @graphql directives currently not supported"
[//]: # "This test had an assertion with a fail annotation that testconv cannot convert losslessly. If you need the original responses, you can find it in git history. For example, at commit https://github.com/tailcallhq/tailcall/tree/1c32ca9e8080ae3b17e9cf41078d028d3e0289da"

```graphql @config
schema
  @server(port: 8001, queryValidation: false, hostname: "0.0.0.0")
  @upstream(baseURL: "http://upstream/graphql", httpCache: 42, batch: {delay: 1}) {
  query: Query
}

type Query {
  a: [A] @graphQL(name: "posts")
}

type A {
  id: Int!
  bid: Int!
  cid: Int!
  b: B @graphQL(name: "b", args: [{key: "id", value: "{{.value.bid}}"}], batch: true)
  c: C @graphQL(name: "c", args: [{key: "id", value: "{{.value.cid}}"}], batch: true)
}

type C {
  id: Int!
  x: String!
}

type B {
  id: Int!
  y: String!
}
```

```yml @mock
- request:
    method: POST
    url: http://upstream/graphql
    textBody: '[{"query": "a {id, bid, cid}"}]'
  response:
    status: 200
    body:
      data:
        a:
          - bid: 1
            cid: 1
            id: 1
          - bid: 1
            cid: 1
            id: 2
          - bid: 2
            cid: 2
            id: 3
          - bid: 2
            cid: 2
            id: 4
- request:
    method: POST
    url: http://upstream/graphql
    textBody: '[{"query": "b {y}"},{"query": "c {x}"}]'
  response:
    status: 200
    body:
      - data:
          b:
            y: 1
      - data:
          c:
            x: 1
- request:
    method: POST
    url: http://upstream/graphql
    textBody: '[{"query": "c {x}"},{"query": "b {y}"}]'
  response:
    status: 200
    body:
      - data:
          c:
            x: 1
      - data:
          b:
            y: 1
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { a { id bid cid b { y } c { x } } }
```
