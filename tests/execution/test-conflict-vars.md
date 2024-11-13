# test-conflict-vars

```graphql @schema
schema {
  query: Query
}

type Query {
  hello: String @expr(body: "world")
}
```

```yml @config
schema: {}
server:
  vars: [{key: "a", value: "b"}, {key: "c", value: "d"}]
```

```graphql @schema
schema {
  query: Query
}

type Query {
  hello: String @expr(body: "world")
}
```

```yml @config
schema: {}
server:
  vars: [{key: "a", value: "b"}, {key: "p", value: "q"}]
```
