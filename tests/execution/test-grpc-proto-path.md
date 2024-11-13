---
error: true
---

# test-grpc-proto-path

```yml @file:config.yml
schema: {}
links:
  - id: "news"
    src: "news.proto"
    type: Protobuf
```

```graphql @config
schema {
  query: Query
}

type Query {
  news: NewsData @grpc(method: "GetAllNews", url: "http://localhost:4000")
}

type NewsData {
  news: [News]
}

type News {
  id: Int!
  title: String!
  body: String!
  postImage: String!
}
```
