# Batching disabled

#### server:
```json
{
  "server": {},
  "upstream": {
    "baseURL": "http://jsonplaceholder.typicode.com",
    "httpCache": true,
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
            "path": "/users/{{args.id}}"
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

#### assert:
```yml
mock:
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1
    headers: {}
    body: null
  response:
    status: 200
    headers: {}
    body:
      id: 1
      name: Leanne Graham
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/2
    headers: {}
    body: null
  response:
    status: 200
    headers: {}
    body:
      id: 2
      name: Leanne Graham
assert:
- request:
    method: POST
    url: http://localhost:8080/graphql
    headers: {}
    body:
      query: 'query { u1: user(id: 1) {id} u2: user(id: 2) {id} }'
env: {}
```
