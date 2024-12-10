```json @config
{
  "inputs": [
    {
      "curl": {
        "src": "{{.env.BASE_URL}}/track/3135556",
        "fieldName": "track"
      }
    },
    {
      "curl": {
        "src": "{{.env.BASE_URL}}/album/302127",
        "fieldName": "album"
      }
    },
    {
      "curl": {
        "src": "{{.env.BASE_URL}}/artist/27",
        "fieldName": "artist"
      }
    },
    {
      "curl": {
        "src": "{{.env.BASE_URL}}/playlist/908622995",
        "fieldName": "playlist"
      }
    },
    {
      "curl": {
        "src": "{{.env.BASE_URL}}/chart",
        "fieldName": "chart"
      }
    },
    {
      "curl": {
        "src": "{{.env.BASE_URL}}/editorial",
        "fieldName": "editorial"
      }
    },
    {
      "curl": {
        "src": "{{.env.BASE_URL}}/user/2529",
        "fieldName": "user"
      }
    },
    {
      "curl": {
        "src": "{{.env.BASE_URL}}/search?q=eminem",
        "fieldName": "search"
      }
    }
  ],
  "preset": {
    "mergeType": 1.0,
    "treeShake": true,
    "inferTypeNames": true
  },
  "output": {
    "path": "./output.graphql"
  },
  "schema": {
    "query": "Query"
  }
}
```

```json @env
{
  "BASE_URL": "https://api.deezer.com"
}
```
