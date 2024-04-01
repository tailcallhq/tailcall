---
check_identity: true
---

# test-tag

```graphql @server
schema @server @upstream(baseURL: "http://jsonplacheholder.typicode.com") {
  query: Query
}

type NEWS {
  getAllNews: News__NewsList!
}

type News__News @tag(id: "news.News") {
  body: String @const(data: "This is a news body")
  id: Int @const(data: 1)
  postImage: String @const(data: "http://example.com/image.jpg")
  title: String @const(data: "This is a news title")
}

type News__NewsList @tag(id: "news.NewsList") {
  news: [News__News]
}

type Query {
  news: NEWS
}
```
