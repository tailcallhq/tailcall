# Interfaces defined in json

```json @server
{
  "upstream": {
    "baseURL": "http://jsonplaceholder.typicode.com"
  },
  "schema": {
    "query": "Query"
  },
  "types": {
    "IA": {
      "fields": {
        "a": {
          "type": "String"
        }
      }
    },
    "B": {
      "implements": ["IA"],
      "fields": {
        "a": {
          "type": "String"
        },
        "b": {
          "type": "String"
        }
      }
    },
    "Query": {
      "fields": {
        "bar": {
          "type": "B",
          "http": {
            "path": "/posts"
          }
        }
      }
    }
  }
}
```
