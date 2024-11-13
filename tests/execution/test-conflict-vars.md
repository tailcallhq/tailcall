# test-conflict-vars

```graphql @config
schema {
  query: Query
}

type Query {
  hello: String @expr(body: "world")
}
```

```yml @file:config.yml
schema: {}
server:
  vars: [{key: "a", value: "b"}, {key: "c", value: "d"}]
```

```graphql @config
schema {
  query: Query
}

type Query {
  hello: String @expr(body: "world")
}
```

```yml @file:config.yml
schema: {}
server:
  vars: [{key: "a", value: "b"}, {key: "p", value: "q"}]
```
