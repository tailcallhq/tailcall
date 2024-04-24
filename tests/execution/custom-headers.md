# Custom Headers

```json @server
{
  "server": {
    "headers": {
      "custom": [
        {
          "key": "x-id",
          "value": "1"
        },
        {
          "key": "x-name",
          "value": "John Doe"
        }
      ]
    }
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
          "expr": {
            "body": "Hello World!"
          },
          "cache": null
        }
      },
      "cache": null
    }
  }
}
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Accept: application/graphql-response+json
  body:
    query: query { greet }
```
