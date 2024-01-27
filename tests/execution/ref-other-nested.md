# Ref other nested

#### server:
```json
{
  "server": {},
  "upstream": {
    "baseURL": "https://jsonplaceholder.typicode.com"
  },
  "schema": {
    "query": "Query"
  },
  "types": {
    "Query": {
      "fields": {
        "firstUser": {
          "type": "User1",
          "http": {
            "path": "/users/1",
            "baseURL": "https://jsonplaceholder.typicode.com"
          },
          "cache": null
        }
      },
      "cache": null
    },
    "User": {
      "fields": {
        "id": {
          "type": "Int",
          "cache": null
        },
        "name": {
          "type": "String",
          "cache": null
        }
      },
      "cache": null
    },
    "User1": {
      "fields": {
        "user1": {
          "type": "User2",
          "cache": null
        }
      },
      "cache": null
    },
    "User2": {
      "fields": {
        "user2": {
          "type": "User",
          "http": {
            "path": "/users/1",
            "baseURL": "https://jsonplaceholder.typicode.com"
          },
          "cache": null
        }
      },
      "cache": null
    }
  }
}
```

#### assert:
```yml
mock:
- request:
    method: GET
    url: https://jsonplaceholder.typicode.com/users/1
    headers: {}
    body: null
  response:
    status: 200
    headers: {}
    body:
      id: 1
      name: Leanne Graham
assert:
- request:
    method: POST
    url: http://localhost:8080/graphql
    headers: {}
    body:
      query: query { firstUser { user1 { user2 { name } } } }
env: {}
```
