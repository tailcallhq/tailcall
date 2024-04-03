# With args URL

```json @server
{
  "server": {},
  "upstream": {
    "baseURL": "http://jsonplaceholder.typicode.com"
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
            "baseURL": "http://jsonplaceholder.typicode.com",
            "path": "/users/{{args.id}}"
          },
          "cache": null,
          "protected": null
        }
      },
      "cache": null,
      "protected": null
    },
    "User": {
      "fields": {
        "id": {
          "type": "Int",
          "cache": null,
          "protected": null
        },
        "name": {
          "type": "String",
          "cache": null,
          "protected": null
        }
      },
      "cache": null,
      "protected": null
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
    query: "query { user(id: 1) { name } }"
```
