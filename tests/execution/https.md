# Against a server with HTTPS

####
```json @server
{
  "server": {},
  "upstream": {
    "baseURL": "https://jsonplaceholder.typicode.com"
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
            "path": "/users/1",
            "baseURL": "https://jsonplaceholder.typicode.com"
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

####
```yml @mock
- request:
    method: GET
    url: https://jsonplaceholder.typicode.com/users/1
    body: null
  response:
    status: 200
    body:
      id: 1
      name: Leanne Graham
```

####
```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { firstUser { name } }
```
