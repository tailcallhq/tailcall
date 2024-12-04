# Interfaces defined in json

```json @config
{
  "schema": {
    "query": "Query"
  },
  "types": [
    {
      "name": "IA",
      "fields": {
        "a": {
          "type": {
            "name": "String"
          }
        }
      }
    },
    {
      "name": "B",
      "implements": [
        "IA"
      ],
      "fields": {
        "a": {
          "type": {
            "name": "String"
          }
        },
        "b": {
          "type": {
            "name": "String"
          }
        }
      }
    },
    {
      "name": "Query",
      "fields": {
        "bar": {
          "type": {
            "name": "B"
          },
          "http": {
            "url": "http://jsonplaceholder.typicode.com/posts"
          }
        }
      }
    }
  ]
}
```
