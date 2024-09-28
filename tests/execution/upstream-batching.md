# Sending requests to be batched by the upstream server

```json @config
{
  "$schema": "../generated/.tailcallrc.schema.json",
  "server": {},
  "upstream": {},
  "schema": {
    "query": "Query"
  },
  "types": {
    "Query": {
      "fields": {
        "user": {
          "type": {
            "name": "User"
          },
          "args": {
            "id": {
              "type": {
                "name": "Int"
              }
            }
          },
          "http": {
            "batch": {
              "delay": 1,
              "maxSize": 100
            },
            "path": "/users",
            "query": [
              {
                "key": "id",
                "value": "{{.args.id}}"
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
          "type": {
            "name": "Int"
          },
          "cache": null
        },
        "name": {
          "type": {
            "name": "String"
          },
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
  response:
    status: 200
    body:
      - id: 1
        name: foo
      - id: 2
        name: bar
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { u1: user(id: 1) { id } u2: user(id: 2) { id } }"
```
