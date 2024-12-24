---
error: true
---

# test-duplicated-link

```yaml @config
links:
  - id: placeholder
    src: jsonplaceholder.graphql
    type: Config
  - id: placeholder1
    src: jsonplaceholder.graphql
    type: Config
  - id: placeholder1
    src: jsonplaceholder.graphql
    type: Config
  - id: placeholder2
    src: jsonplaceholder.graphql
    type: Config
  - id: placeholder2
    src: jsonplaceholder.graphql
    type: Config
```

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
