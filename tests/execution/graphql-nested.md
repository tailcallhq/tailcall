# Complicated queries

```graphql @schema
schema @server(port: 8000, hostname: "0.0.0.0") {
  query: Query
}

type Query {
  queryNodeA: NodeA @graphQL(url: "http://upstream/graphql", name: "nodeA")
}

type NodeA {
  name: String
  nodeB: NodeB
  nodeC: NodeC
  nodeA: NodeA @modify(name: "child")
}

type NodeB {
  name: String
  nodeA: NodeA
  nodeC: NodeC
}

type NodeC {
  name: String
  nodeA: NodeA
  nodeB: NodeB
}
```

```yml @mock
- request:
    method: POST
    url: http://upstream/graphql
    textBody: {"query": "query { nodeA { name nodeB { name } nodeC { name } } }"}
  expectedHits: 1
  response:
    status: 200
    body:
      data:
        nodeA:
          name: nodeA
          nodeB:
            name: nodeB
          nodeC:
            name: nodeC
          nodeA:
            name: nodeA
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query queryNodeA {
        queryNodeA {
          name
          nodeA {
            name
          }
          nodeB {
            name
          }
          nodeC {
           name
         }
        }
      }
```
