# Federation subgraph with no entities in the config and enableFederation=true

```yaml @config
server:
  port: 8000
  enableFederation: true
upstream:
  httpCache: 42
  batch:
    delay: 100
```

```graphql @schema
schema {
  query: Query
}

type Query {
  user(id: Int!): User @http(url: "http://jsonplaceholder.typicode.com/users/{{.args.id}}")
}

type User @tag(name: "team-accounts") {
  id: Int!
  name: String! @shareable
}

type Post {
  id: Int!
  title: String! @external
}
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: >
      {
        _entities(representations: [
          {id: 1, __typename: "User"}
          {id: 2, __typename: "User"}
          {id: 3, __typename: "Post"}
          {id: 5, __typename: "Post"}
        ]) {
          __typename
          ...on User {
            id
            name
          }
          ...on Post {
            id
            title
          }
        }
      }

- method: POST
  url: http://localhost:8080/graphql
  body:
    query: >
      { _service { sdl } }
```
