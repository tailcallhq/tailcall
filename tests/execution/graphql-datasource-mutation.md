# Graphql datasource

```graphql @schema
schema {
  query: Query
  mutation: Mutation
}

type User {
  id: Int
  name: String
}

type Query {
  users: [User] @graphQL(url: "http://upstream/graphql", name: "users")
}

type Mutation {
  createUser(user: UserInput!): User
    @graphQL(url: "http://upstream/graphql", name: "createUser", args: [{key: "user", value: "{{.args.user}}"}])
}

type UserInput {
  name: String!
  email: String!
  phone: String
}
```

```yml @mock
- request:
    method: POST
    url: http://upstream/graphql
    textBody: {"query": 'mutation { createUser(user: {name: "Test Name", email: "test@email"}) { name } }'}
  response:
    status: 200
    body:
      data:
        createUser:
          name: Test Name
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: 'mutation { createUser(user: {name: "Test Name", email: "test@email"}) { name } }'
```
