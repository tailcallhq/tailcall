# Sending a batched graphql request


```json @server
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


```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1
    headers:
      test: test
    body: null
  expected_hits: 3
  response:
    status: 200
    body:
      id: 1
      name: foo
```


```yml @assert
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
