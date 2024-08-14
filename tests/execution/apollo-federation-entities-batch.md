# Apollo federation \_entities query

```graphql @config
schema
  @server(port: 8000)
  @upstream(baseURL: "http://jsonplaceholder.typicode.com", httpCache: 42, batch: {delay: 100}) {
  query: Query
}

type Query {
  user(id: Int!): User @http(path: "/users/{{.args.id}}")
}

type User @http(path: "/users", query: [{key: "id", value: "{{.value.id}}"}], batchKey: ["id"]) {
  id: Int!
  name: String!
}

type Post
  @graphQL(baseURL: "http://upstream/graphql", batch: true, name: "post", args: [{key: "id", value: "{{.value.id}}"}]) {
  id: Int!
  title: String!
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users?id=1&id=2
  assertHits: false
  response:
    status: 200
    body:
      - id: 1
        name: Leanne Graham
      - id: 2
        name: Ervin Howell

- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users?id=2&id=1
  assertHits: false
  response:
    status: 200
    body:
      - id: 2
        name: Ervin Howell
      - id: 1
        name: Leanne Graham

- request:
    method: POST
    url: http://upstream/graphql
    textBody: '[{ "query": "query { post(id: 3) { id id title } }" },{ "query": "query { post(id: 5) { id id title } }" }]'
  assertHits: false
  response:
    status: 200
    body:
      - data:
          post:
            id: 3
            title: ea molestias quasi exercitationem repellat qui ipsa sit aut
      - data:
          post:
            id: 5
            title: nesciunt quas odio

- request:
    method: POST
    url: http://upstream/graphql
    textBody: '[{ "query": "query { post(id: 5) { id id title } }" },{ "query": "query { post(id: 3) { id id title } }" }]'
  assertHits: false
  response:
    status: 200
    body:
      - data:
          post:
            id: 5
            title: nesciunt quas odio
      - data:
          post:
            id: 3
            title: ea molestias quasi exercitationem repellat qui ipsa sit aut
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
```
