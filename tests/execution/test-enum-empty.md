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
          "type": "Foo",
          "args": {
            "val": {
              "type": "String",
              "required": true
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
