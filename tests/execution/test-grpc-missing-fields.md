# test-grpc-missing-fields

###### sdl error

#### server:

```graphql
schema @link(id: "news", src: "tests/graphql/errors/proto/news.proto", type: Protobuf) {
  query: Query
}

type Query {
  news: NewsData!
    @grpc(service: "news.NewsService", method: "GetAllNews", baseURL: "http://localhost:4000", protoId: "news")
}

type NewsData {
  news: [News]!
}

type News {
  body: String
  postImage: String
  title: String
}
```
