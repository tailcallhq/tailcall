# test-response-header-value

```graphql @schema
schema {
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

```yml @config
schema: {}
server:
  headers: {custom: [{key: "a", value: "a"}]}
```

```graphql @schema
schema {
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

```yml @config
schema: {}
server:
  headers: {custom: [{key: "a", value: "b"}]}
```
