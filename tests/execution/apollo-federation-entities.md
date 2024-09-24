# Apollo federation query

```graphql @config
schema
  @server(port: 8000)
  @upstream(baseURL: "http://jsonplaceholder.typicode.com", httpCache: 42, batch: {delay: 100})
  @link(src: "./posts.graphql") {
  query: Query
}

type Query {
  user(id: Int!): User @http(path: "/users/{{.args.id}}")
}

type User @call(steps: [{query: "user", args: {id: "{{.value.user.id}}"}}]) @shareable {
  id: Int!
  name: String!
}
```

```graphql @file:posts.graphql
type Post @expr(body: {id: "{{.value.id}}", title: "post-title-{{.value.id}}"}) {
  id: Int!
  title: String! @override(from: "name")
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1
  response:
    status: 200
    body:
      id: 1
      name: Leanne Graham

- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/2
  response:
    status: 200
    body:
      id: 2
      name: Ervin Howell
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: >
      {
        _entities(representations: [
          {user: { id: 1 }, __typename: "User"}
          {user: { id: 2 }, __typename: "User"}
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
