# Graphql datasource

#### server:

```graphql
schema {
  query: Query
  mutation: Mutation
}

type User {
  id: Int
  name: String
}

type Query {
  users: [User] @graphQL(baseURL: "http://upstream/graphql", name: "users")
}

type Mutation {
  createUser(user: UserInput!): User
    @graphQL(baseURL: "http://upstream/graphql", name: "createUser", args: [{key: "user", value: "{{args.user}}"}])
}

type UserInput {
  name: String!
  email: String!
  phone: String
}
```

#### assert:

```yml
mock:
  - request:
      method: POST
      url: http://upstream/graphql
      headers: {}
      body: '{ "query": "mutation { createUser(user: {name: \"Test Name\",email: \"test@email\"}) { name } }" }'
    response:
      status: 200
      headers: {}
      body:
        data:
          createUser:
            name: Test Name
assert:
  - request:
      method: POST
      url: http://localhost:8080/graphql
      headers: {}
      body:
        query: 'mutation { createUser(user: {name: "Test Name", email: "test@email"}) { name } }'
env: {}
```
