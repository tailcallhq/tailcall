# Batched graphql request to batched upstream query

```json @server
{
  "server": {
    "batchRequests": true
  },
  "upstream": {
    "batch": {
      "maxSize": 100,
      "delay": 1,
      "headers": []
    }
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
            "path": "/users",
            "query": [
              {
                "key": "id",
                "value": "{{args.id}}"
              }
            ],
            "baseURL": "http://jsonplaceholder.typicode.com",
            "batchKey": ["id"]
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
    }
  }
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users?id=1&id=2
    headers:
      test: test
    body: null
  response:
    status: 200
    body:
      - id: 1
        name: foo
      - id: 2
        name: bar
```

```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  body:
    - query: "query { user(id: 1) { id name } }"
    - query: "query { user(id: 2) { id name } }"
```
