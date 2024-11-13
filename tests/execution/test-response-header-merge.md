# test-response-header-value

```graphql @config
schema @link(src: "config-a.yml", type: Config) {
  query: Query
}

type User {
  name: String
  age: Int
}

type Query {
  user: User @expr(body: {name: "John"})
}
```

```yml @file:config-a.yml
schema: {}
server:
  headers: {custom: [{key: "a", value: "a"}]}
```

```graphql @config
schema @link(src: "config-b.yml", type: Config) {
  query: Query
}

type User {
  name: String
  age: Int
}

type Query {
  user: User @expr(body: {name: "John"})
}
```

```yml @file:config-b.yml
schema: {}
server:
  headers: {custom: [{key: "a", value: "b"}]}
```
