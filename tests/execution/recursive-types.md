# Recursive Type

```graphql @schema
schema @server {
  query: Query
  mutation: Mutation
}

type User {
  name: String
  id: Int!
  connections: [Connection] @http(url: "http://jsonplaceholder.typicode.com/connections/{{.value.id}}")
}

type Connection {
  type: String
  user: User
}

type Query {
  user: User @http(url: "http://jsonplaceholder.typicode.com/users/1")
}

type Mutation {
  createUser(user: User): User
    @http(url: "http://jsonplaceholder.typicode.com/user", method: "POST", body: "{{.args.user}}")
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
      name: User1
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/connections/1
  response:
    status: 200
    body:
      - type: friend
        user:
          id: 2
          name: User2

- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/connections/2
  response:
    status: 200
    body:
      - type: friend
        user:
          id: 3
          name: User3
      - type: coworker
        user:
          id: 4
          name: User4

- request:
    method: POST
    url: http://jsonplaceholder.typicode.com/user
    body:
      id: 111
      name: NewUser
      connections:
        - type: friend
          user:
            id: 1
            name: User1
  response:
    status: 200
    body:
      id: 111
      name: NewUser

- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/connections/111
  response:
    status: 200
    body:
      - type: friend
        user:
          id: 1
          name: User1
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user {
          name
          id
          connections {
            type
            user {
              name
              id
              connections {
                user {
                  name
                  id
                }
              }
            }
          }
        }
      }

- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      mutation {
        createUser(
          user: {
            id: 111,
            name: "NewUser",
            connections: [
              {
                type: "friend"
                user: {
                  id: 1
                  name: "User1"
                }
              }
            ]
          }
        ) {
          name
          id
          connections {
            type
            user {
              name
              id
            }
          }
        }
      }
```
