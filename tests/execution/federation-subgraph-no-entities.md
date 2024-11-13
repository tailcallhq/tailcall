# Federation subgraph with no entities in the config

```graphql @config
schema @link(src: "config.yml", type: Config) {
  query: Query
}

type Query {
  user(id: Int!): User @http(url: "http://jsonplaceholder.typicode.com/users/{{.args.id}}")
}

type User {
  id: Int!
  name: String!
}

type Post {
  id: Int!
  title: String!
}
```

```yml @file:config.yml
upstream:
  httpCache: 42
  batch: {delay: 100}
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
