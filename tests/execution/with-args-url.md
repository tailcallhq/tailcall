# With args URL

```json @config
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
            "path": "/users/{{.args.id}}",
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
    url: http://jsonplaceholder.typicode.com/users/1
    headers:
      test: test
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
    query: "query { user(id: 1) { name } }"
```
