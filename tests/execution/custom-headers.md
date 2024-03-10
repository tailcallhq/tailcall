# Custom Headers

####
```json @server
{
  "server": {
    "responseHeaders": [
      {
        "key": "x-id",
        "value": "1"
      },
      {
        "key": "x-name",
        "value": "John Doe"
      }
    ]
  },
  "upstream": {},
  "schema": {
    "query": "Query"
  },
  "types": {
    "Query": {
      "fields": {
        "greet": {
          "type": "String",
          "const": {
            "data": "Hello World!"
          },
          "cache": null
        }
      },
      "cache": null
    }
  }
}
```

####
```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { greet }
```
