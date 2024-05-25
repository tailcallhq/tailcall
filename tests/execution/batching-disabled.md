# Batching disabled

```json @config
{
  "server": {},
  "upstream": {
    "baseURL": "http://jsonplaceholder.typicode.com",
    "httpCache": 42,
    "batch": {
      "maxSize": 100,
      "delay": 0,
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
            "path": "/users/{{.args.id}}"
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
        },
        "username": {
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
    url: http://jsonplaceholder.typicode.com/users/1
  response:
    status: 200
    body:
      id: 1
      name: Leanne Graham
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/2
  response:
    status: 200
    body:
      id: 2
      name: Leanne Graham
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { u1: user(id: 1) {id} u2: user(id: 2) {id} }"
```
