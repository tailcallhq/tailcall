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

#### mock:

```yml
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1
    headers:
      test: test
    body: null
  response:
    status: 200
    body:
      id: 1
      name: foo

```

#### assert:

```yml
- method: POST
  url: http://localhost:8080/graphql
  body:
  - query: query { user { id } }
  - query: query { user { name } }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { user { id } }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: FOO
```
