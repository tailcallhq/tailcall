# Graphql datasource

```graphql @server
schema {
  query: Query
  mutation: Mutation
}

input UserInput {
  email: String!
  name: String!
  phone: String
}

type Mutation {
  createUser(user: UserInput!): User
    @graphQL(args: [{key: "user", value: "{{.args.user}}"}], baseURL: "http://upstream/graphql", name: "createUser")
}

type Query {
  users: [User] @graphQL(baseURL: "http://upstream/graphql", name: "users")
}

type User {
  id: Int
  name: String
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
