# Sending a batched graphql request

#### server:

```json
{
  "server": {
    "batchRequests": true
  },
  "upstream": {},
  "schema": {
    "query": "Query"
  },
  "types": {
    "Query": {
      "fields": {
        "user": {
          "type": "User",
          "http": {
            "path": "/users/1",
            "baseURL": "http://jsonplaceholder.typicode.com"
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

#### assert:

```yml
mock:
  - request:
      method: GET
      url: http://jsonplaceholder.typicode.com/users/1
      headers:
        test: test
      body: null
    response:
      status: 200
      headers: {}
      body:
        id: 1
        name: foo
assert:
  - request:
      method: POST
      url: http://localhost:8080/graphql
      headers: {}
      body:
        - query: query { user { id } }
        - query: query { user { name } }
  - request:
      method: POST
      url: http://localhost:8080/graphql
      headers: {}
      body:
        query: query { user { id } }
  - request:
      method: POST
      url: http://localhost:8080/graphql
      headers: {}
      body:
        query: FOO
env: {}
```
