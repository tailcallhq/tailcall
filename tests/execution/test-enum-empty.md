---
error: true
---

# test-enum-empty

```json @config
{
  "server": {},
  "upstream": {
    "baseURL": "http://localhost:8080"
  },
  "schema": {
    "query": "Query"
  },
  "types": {
    "Query": {
      "fields": {
        "foo": {
          "type": {
            "name": "Foo"
          },
          "args": {
            "val": {
              "type": {
                "name": "String",
                "required": true
              }
            }
          },
          "expr": {
            "body": "{{.args.val}}"
          },
          "cache": null,
          "protected": null
        }
      },
      "protected": null
    }
  },
  "enums": {
    "Foo": {
      "variants": [],
      "doc": null
    }
  }
}
```
