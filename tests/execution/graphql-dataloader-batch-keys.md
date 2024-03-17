# GraphQL datasource

##### skip

[//]: # "nested @graphql directives currently not supported"
[//]: # "This test had an assertion with a fail annotation that testconv cannot convert losslessly. If you need the original responses, you can find it in git history. For example, at commit https://github.com/tailcallhq/tailcall/tree/1c32ca9e8080ae3b17e9cf41078d028d3e0289da"

```graphql @server
schema @server(hostname: "0.0.0.0", port: 8001, queryValidation: false) @upstream(baseURL: "http://upstream/graphql", batch: {delay: 1, headers: [], maxSize: 100}, httpCache: true) {
  query: Query
}

type A {
  b: B @graphQL(args: [{key: "id", value: "{{value.bid}}"}], batch: true, name: "b")
  bid: Int!
  c: C @graphQL(args: [{key: "id", value: "{{value.cid}}"}], batch: true, name: "c")
  cid: Int!
  id: Int!
}

type B {
  id: Int!
  y: String!
}

type C {
  id: Int!
  x: String!
}

type Query {
  a: [A] @graphQL(name: "posts")
}
```

```yml @mock
- request:
    method: POST
    url: http://upstream/graphql
    body: '[{"query": "a {id, bid, cid}"}]'
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
    body: '[{"query": "b {y}"},{"query": "c {x}"}]'
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
    body: '[{"query": "c {x}"},{"query": "b {y}"}]'
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

```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { a { id bid cid b { y } c { x } } }
```
