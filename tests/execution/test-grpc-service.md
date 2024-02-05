# test-grpc-service

###### sdl error

#### server:

```graphql
schema @link(id: "news", src: "../graphql/errors/proto/news.proto", type: Protobuf) {
  query: Query
}

type Query {
  news: NewsData
    @grpc(service: "YourServiceName", method: "GetAllNews", baseURL: "http://localhost:4000", protoId: "news")
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
