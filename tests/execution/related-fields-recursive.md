```graphql @config
schema @server(port: 8000, hostname: "0.0.0.0") {
  query: Query
}

type Query {
  queryNodeA: [NodeA] @graphQL(url: "http://localhost:8083/graphql",name: "queryNodeA", batch: false)
}

type NodeA {
  name: String
  nodeB: NodeB
}

type NodeB {
  name: String
  nodeA: NodeA
}
```
