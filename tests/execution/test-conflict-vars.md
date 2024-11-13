# test-conflict-vars

```graphql @config
schema @link(src: "config-a.yml", type: Config) {
  query: Query
}

type Query {
  hello: String @expr(body: "world")
}
```

```graphql @config
schema @link(src: "config-b.yml", type: Config) {
  query: Query
}

type Query {
  hello: String @expr(body: "world")
}
```

```yml @file:config-a.yml
schema: {}
server:
  vars: [{key: "a", value: "b"}, {key: "c", value: "d"}]
```

```yml @file:config-b.yml
schema: {}
server:
  vars: [{key: "a", value: "b"}, {key: "p", value: "q"}]
```
