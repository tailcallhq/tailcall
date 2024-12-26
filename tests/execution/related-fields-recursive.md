```yaml @config
server:
  port: 8000
  hostname: "0.0.0.0"
```

```graphql @schema
schema {
  query: Query
}

type Query {
  queryNodeA: [NodeA] @graphQL(url: "http://localhost:8083/graphql", name: "queryNodeA", batch: false)
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
