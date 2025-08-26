```graphql @schema
schema @server(port: 8000, hostname: "0.0.0.0") {
  query: Query
}

type Query {
  queryNodeA: [NodeA!] @graphQL(url: "http://upstream/graphql", name: "nodeA")
  queryNodeB: [NodeB!] @graphQL(url: "http://upstream/graphql", name: "nodeB")
}

type NodeA {
  name: String
  nodeA_id: String
  nodeB(nodeB_id: String): NodeB
}

type NodeB {
  name: String
  nodeB_id: String
}
```

```yml @mock
- request:
    method: POST
    url: http://upstream/graphql
    textBody: {"query": "query { nodeA { name nodeA_id nodeB(nodeB_id:) { name nodeB_id } } }"}
  expectedHits: 1
  response:
    status: 200
    body:
      data:
        nodeA:
          - name: nodeA
            nodeA_id: nodeA_id
            nodeB:
              name: NodeB
              nodeB_id: nodeB_id
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query queryNodeA($nodeB: String)  {
        queryNodeA {
          name
          nodeA_id
          nodeB(nodeB_id: $nodeB) {
            name
            nodeB_id
          }
        }
      }
    variables:
      nodeB: "nodeB_id"
```
