# Federation subgraph with no entities in the config

```yaml @config
server:
  port: 8000
  enableFederation: false
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

type User @call(steps: [{query: "user", args: {id: "{{.value.id}}"}}]) {
  id: Int!
  name: String!
}

type Post @expr(body: {id: "{{.value.id}}", title: "post-title-{{.value.id}}"}) {
  id: Int!
  title: String!
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
