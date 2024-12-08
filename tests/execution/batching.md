# Sending a batched graphql request

```json @config
{
  "server": {
    "batchRequests": true
  },
  "upstream": {},
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
          "http": {
            "url": "http://jsonplaceholder.typicode.com/users/1"
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
    url: http://jsonplaceholder.typicode.com/users/1
    headers:
      test: test
  expectedHits: 3
  response:
    status: 200
    body:
      id: 1
      name: foo
```

```yml @test
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
