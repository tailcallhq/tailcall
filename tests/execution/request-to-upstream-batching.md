# Batched graphql request to batched upstream query

```json @config
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
  "types": [
    {
      "name": "Query",
      "fields": {
        "user": {
          "type": {
            "name": "User"
          },
          "args": {
            "id": {
              "type": {
                "name": "Int",
                "required": true
              }
            }
          },
          "http": {
            "url": "http://jsonplaceholder.typicode.com/users",
            "query": [
              {
                "key": "id",
                "value": "{{.args.id}}"
              }
            ],
            "batchKey": [
              "id"
            ]
          },
          "cache": null
        }
      },
      "cache": null
    },
    {
      "name": "User",
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
  ]
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
    - query: "query { user(id: 1) { id name } }"
    - query: "query { user(id: 2) { id name } }"
```
