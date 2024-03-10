# Ref other nested

####

```json @server
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

####

```yml @mock
- request:
    method: GET
    url: https://jsonplaceholder.typicode.com/users/1
    body: null
  expected_hits: 2
  response:
    status: 200
    body:
      id: 1
      name: Leanne Graham
```

####

```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { firstUser { user1 { user2 { name } } } }
```
