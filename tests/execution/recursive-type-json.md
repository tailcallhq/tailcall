# Recursive Type JSON

```json @config
{
  "$schema": "./.tailcallrc.schema.json",
  "upstream": {
    "baseURL": "https://jsonplaceholder.typicode.com",
    "httpCache": 42
  },
  "schema": {
    "query": "Query"
  },
  "types": {
    "Query": {
      "fields": {
        "user": {
          "type": "User",
          "args": {
            "id": {
              "type": "Int",
              "required": true
            }
          },
          "http": {
            "path": "/users/1"
          }
        }
      }
    },
    "User": {
      "fields": {
        "id": {
          "type": "Int",
          "required": true
        },
        "name": {
          "type": "String",
          "required": true
        },
        "friend": {
          "type": "User",
          "http": {
            "path": "/friends/1"
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
    url: https://jsonplaceholder.typicode.com/friends/1
  expectedHits: 2
  response:
    status: 200
    body:
      id: 2
      name: User2
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { user { name id friend { name id friend { name id } } } }
```
