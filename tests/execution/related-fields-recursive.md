```graphql @config
scalar DateTime

schema
@server(
    port: 8000
    hostname: "0.0.0.0"
)
@upstream( baseURL: "http://localhost:8083/graphql")
{
    query: Query
}

type Query {
    queryNodeA: [NodeA]  @graphQL (name:"queryNodeA" batch: false)

}

type NodeA {
    name : String
    nodeB : NodeB
}

type NodeB {
    name : String
    nodeA : NodeA
}
```
