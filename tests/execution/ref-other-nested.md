# Ref other nested

```json @config
{
  "server": {},
  "schema": {
    "query": "Query"
  },
  "types": {
    "Query": {
      "fields": {
        "firstUser": {
          "type": {
            "name": "User1"
          },
          "http": {
            "url": "http://jsonplaceholder.typicode.com/users/1"
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
    },
    "User1": {
      "fields": {
        "user1": {
          "type": {
            "name": "User2"
          },
          "cache": null
        }
      },
      "cache": null
    },
    "User2": {
      "fields": {
        "user2": {
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
    }
  }
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1
  expectedHits: 2
  response:
    status: 200
    body:
      id: 1
      name: Leanne Graham
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { firstUser { user1 { user2 { name } } } }
```
