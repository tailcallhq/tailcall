# Tailcall Federation router

```graphql @config
schema
  @link(src: "http://localhost:4000", type: SubGraph, meta: {name: "Users"})
  @link(src: "http://localhost:5000", type: SubGraph, meta: {name: "Posts"})
{
  query: Query
}

type Query {
  version: String @expr(body: "test")
}
```

```yml @mock
- request:
    method: POST
    url: http://localhost:4000/graphql
    textBody: {"query": "{ config }"}
  response:
    status: 200
    body:
      config: |
        schema
          @server(port: 8000)
          @upstream(baseURL: "http://jsonplaceholder.typicode.com", httpCache: 42, batch: {delay: 100}) {
          query: Query
        }

        type Query {
          users: [User] @http(path: "/users")
          user(id: Int!): User @http(path: "/users/{{.args.id}}")
        }

        type User {
          id: Int!
          name: String!
          username: String!
          email: String!
          phone: String
          website: String
        }

        type Post {
          userId: Int!
          user: User @http(path: "/users/{{.value.userId}}")
        }

- request:
    method: POST
    url: http://localhost:5000/graphql
    textBody: {"query": "{ config }"}
  response:
    status: 200
    body:
      config: |
        schema
          @server(port: 8000)
          @upstream(baseURL: "http://jsonplaceholder.typicode.com", httpCache: 42, batch: {delay: 100}) {
          query: Query
        }

        type Query {
          posts: [Post] @http(path: "/posts")
        }

        type Post {
          id: Int!
          userId: Int!
          title: String!
          body: String!
        }
```
