# Interfaces defined in json

```json @config
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
          "type": {
            "name": "String"
          }
        }
      }
    },
    "B": {
      "implements": ["IA"],
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
    "Query": {
      "fields": {
        "bar": {
          "type": {
            "name": "B"
          },
          "http": {
            "path": "/posts"
          }
        }
      }
    }
  }
}
```
