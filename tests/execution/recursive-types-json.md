# Recursive Type JSON

```json @config
{
  "$schema": "./.tailcallrc.schema.json",
  "upstream": {
    "baseURL": "https://jsonplaceholder.typicode.com",
    "httpCache": 42
  },
  "schema": {
    "query": "Query",
    "mutation": "Mutation"
  },
  "types": {
    "Query": {
      "fields": {
        "user": {
          "type": {
            "name": "User"
          },
          "http": {
            "path": "/users/1"
          }
        }
      }
    },
    "Mutation": {
      "fields": {
        "createUser": {
          "args": {
            "user": {
              "type": {
                "name": "User"
              }
            }
          },
          "type": {
            "name": "User"
          },
          "http": {
            "path": "/user",
            "method": "POST",
            "body": "{{.args.user}}"
          }
        }
      }
    },
    "User": {
      "fields": {
        "id": {
          "type": {
            "name": "Int",
            "required": true
          }
        },
        "name": {
          "type": {
            "name": "String",
            "required": true
          }
        },
        "connections": {
          "type": {
            "list": {
              "name": "Connection"
            }
          },
          "http": {
            "path": "/connections/{{.value.id}}"
          }
        }
      }
    },
    "Connection": {
      "fields": {
        "type": {
          "type": {
            "name": "String"
          }
        },
        "user": {
          "type": {
            "name": "User"
          }
        }
      }
    }
  }
}
```

```yml @mock
- request:
    method: GET
    url: https://jsonplaceholder.typicode.com/users/1
  response:
    status: 200
    body:
      id: 1
      name: User1
- request:
    method: GET
    url: https://jsonplaceholder.typicode.com/connections/1
  response:
    status: 200
    body:
      - type: friend
        user:
          id: 2
          name: User2

- request:
    method: GET
    url: https://jsonplaceholder.typicode.com/connections/2
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
    url: https://jsonplaceholder.typicode.com/user
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
    url: https://jsonplaceholder.typicode.com/connections/111
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
