# Batching group by default

#### server:

```graphql
schema
  @server
  @upstream(baseURL: "http://jsonplaceholder.typicode.com", httpCache: true, batch: {delay: 1, maxSize: 1000}) {
  query: Query
}

type Query {
  posts: [Post] @http(path: "/posts?id=11&id=3&foo=1")
}

type Post {
  id: Int
  title: String
  body: String
  userId: Int!
  user: User
    @http(groupBy: ["id"], path: "/users", query: [{key: "id", value: "{{value.userId}}"}, {key: "foo", value: "bar"}])
}

type User {
  id: Int
  name: String
}
```

#### mock:

```yml
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/posts?id=11&id=3&foo=1
    body: null
  response:
    status: 200
    body:
      - body: bar
        id: 11
        title: foo
        userId: 1
      - body: bar
        id: 3
        title: foo
        userId: 2
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users?foo=bar&id=1&foo=bar&id=2
    body: null
  response:
    status: 200
    body:
      - id: 1
        name: foo
      - id: 2
        name: bar
```

#### assert:

```yml
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { posts { user { id } userId } }
```
