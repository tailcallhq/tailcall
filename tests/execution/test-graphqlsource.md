# test-graphqlsource

###### check identity

#### server:

```graphql
schema @server @upstream(baseURL: "http://localhost:8000/graphql") {
  query: Query
}

type Post {
  id: Int!
  user: User @graphQL(args: [{key: "id", value: "{{value.userId}}"}], name: "user")
  userId: Int!
}

type Query {
  post(id: Int!): Post @http(baseURL: "http://jsonplacheholder.typicode.com", path: "/posts/{{args.id}}")
}

type User {
  id: Int
  name: String
}
```
