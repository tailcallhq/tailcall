# test-grpc-missing-fields

###### sdl error

#### server:

```graphql
schema @link(id: "news", src: "../graphql/errors/proto/news.proto", type: Protobuf) {
  query: Query
}

type Query {
  news: NewsData! @grpc(method: "news.NewsService.GetAllNews", baseURL: "http://localhost:4000")
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
