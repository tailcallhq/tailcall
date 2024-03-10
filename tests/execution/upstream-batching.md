# Sending requests to be batched by the upstream server

#### server:

```json
{
  "server": {},
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
              "type": "Int"
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
            "batchKey": [
              "id"
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

#### mock:

```yml
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

#### assert:

```yml
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { u1: user(id: 1) { id } u2: user(id: 2) { id } }"
```
