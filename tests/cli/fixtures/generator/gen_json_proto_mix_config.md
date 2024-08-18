```json @config
{
  "inputs": [
    {
      "curl": {
        "src": "https://jsonplaceholder.typicode.com/users",
        "fieldName": "users"
      }
    },
    {
      "proto": {
        "src": "/Users/ssdd/RustroverProjects/tco/tailcall-fixtures/fixtures/protobuf/news_dto.proto"
      }
    }
  ],
  "preset": {
    "mergeType": 1,
    "consolidateURL": 0.5,
    "inferTypeNames": true,
    "treeShake": true
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
