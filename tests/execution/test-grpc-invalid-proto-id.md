# test-grpc-invalid-proto-id

###### sdl error

#### server:

```graphql
schema {
  query: Query
}

type Query {
  news: NewsData
    @grpc(service: "news.NewsService", method: "GetAllNews", baseURL: "http://localhost:4000", protoId: "abc")
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
