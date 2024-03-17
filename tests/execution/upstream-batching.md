# Sending requests to be batched by the upstream server

```json @server
{
  "server": {},
  "upstream": {
    "batch": {
      "delay": 1,
      "headers": [],
      "maxSize": 100
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
              "type": "Int"
            }
          },
          "http": {
            "baseURL": "http://jsonplaceholder.typicode.com",
            "batchKey": [
              "id"
            ],
            "path": "/users",
            "query": [
              {
                "key": "id",
                "value": "{{args.id}}"
              }
            ]
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
    query: "query { u1: user(id: 1) { id } u2: user(id: 2) { id } }"
```
