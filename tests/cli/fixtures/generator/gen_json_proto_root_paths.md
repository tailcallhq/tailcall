```json @config
{
  "inputs": [
    {
      "proto": {
        "src": "tailcall-fixtures/fixtures/protobuf/news_root.proto"
      }
    }
  ],
  "preset": {
    "mergeType": 1.0
  },
  "output": {
    "path": "./output.graphql",
    "format": "graphQL"
  },
  "schema": {
    "query": "Query"
  }
}
```
