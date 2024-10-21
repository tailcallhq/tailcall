# Interfaces defined in json

```json @config
{
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
            "url": "http://jsonplaceholder.typicode.com/posts"
          }
        }
      }
    }
  }
}
```
