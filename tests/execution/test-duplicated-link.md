---
error: true
---

# test-duplicated-link

```graphql @file:jsonplaceholder.graphql
schema {
  query: Query
}

type Query {
  posts: [Post] @http(url: "http://jsonplaceholder.typicode.com/posts")
  users: [User] @http(url: "http://jsonplaceholder.typicode.com/users")
  user(id: Int!): User @http(url: "http://jsonplaceholder.typicode.com/users/{{.args.id}}")
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
  id: Int!
  userId: Int!
  title: String!
  body: String!
  user: User @http(url: "http://jsonplaceholder.typicode.com/users/{{.value.userId}}")
}
```

```yml @config
schema: {}
upstream:
  batch: {httpCache: 42, delay: 100}
links:
  - type: Config
    src: "jsonplaceholder.graphql"
    id: "placeholder"
  - type: Config
    src: "jsonplaceholder.graphql"
    id: "placeholder1"
  - type: Config
    src: "jsonplaceholder.graphql"
    id: "placeholder1"
  - type: Config
    src: "jsonplaceholder.graphql"
    id: "placeholder2"
  - type: Config
    src: "jsonplaceholder.graphql"
    id: "placeholder2"
```

```graphql @schema
schema {
  query: Query
}

type Query {
  posts: [Post] @http(url: "http://jsonplaceholder.typicode.com/posts")
  user(id: Int!): User @http(url: "http://jsonplaceholder.typicode.com/users/{{.args.id}}")
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
  id: Int!
  userId: Int!
  title: String!
  body: String!
  user: User @http(url: "http://jsonplaceholder.typicode.com/users/{{.value.userId}}")
}
```
