# Simple query

#### server:

```json
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
        "firstUser": {
          "type": "User",
          "http": {
            "path": "/users/1"
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
      headers: {}
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
        query: query { firstUser { name } }
env: {}
```
