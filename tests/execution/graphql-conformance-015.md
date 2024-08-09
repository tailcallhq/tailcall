# Optional input fields

```graphql @config
schema
  @server(port: 8001, queryValidation: false, hostname: "0.0.0.0")
  @upstream(baseURL: "http://upstream/graphql", httpCache: 42) {
  query: Query
}

type Query {
  user(id: ID!): User! @graphQL(name: "user", args: [{key: "id", value: "{{.args.id}}"}])
}

type User {
  id: ID!
  name: String!
  profilePic(size: Int! = 100, width: Int, height: Int = 100): String!
    @expr(body: "{{.value.id}}_{{.args.size}}_{{.args.width}}_{{.args.height}}")
  featuredVideo(video: VideoSize! = {width: 1600, height: 900}): String!
    @expr(body: "video_{{.value.id}}_{{.args.video.width}}_{{.args.video.height}}_{{.args.video.hdr}}")
  featuredVideoPreview(video: VideoSize!): String!
    @expr(body: "video_{{.value.id}}_{{.args.video.width}}_{{.args.video.height}}_{{.args.video.hdr}}")
  searchComments(query: [[String!]!]! = [["today"]]): String! @expr(body: "video_{{.value.id}}_{{.args.query}}")
}

input VideoSize {
  width: Int
  height: Int
  hdr: Boolean = true
}
```

```yml @mock
- request:
    method: POST
    url: http://upstream/graphql
    textBody: '{ "query": "query { user(id: 4) { id name } }" }'
  expectedHits: 9
  response:
    status: 200
    body:
      data:
        user:
          id: 4
          name: User 4
```

```yml @test
# Positve: no optional
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4) {
          id
          name
          profilePic
        }
      }
# Positve: different size
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4) {
          id
          name
          profilePic(size: 200)
        }
      }
# Positve: width only
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4) {
          id
          name
          profilePic(width: 200)
        }
      }
# Positve: width only, unset height
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4) {
          id
          name
          profilePic(width: 200, height: null)
        }
      }
# Positve: width and height
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4) {
          id
          name
          profilePic(width: 200, height: 50)
        }
      }
# Positve: video default
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4) {
          id
          name
          featuredVideo
        }
      }
# Positve: video overwrite
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4) {
          id
          name
          featuredVideo(video: {width: 1920, height: 1080, hdr: true})
        }
      }
# Positve: comments default
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4) {
          id
          name
          searchComments
        }
      }
# Positve: comments overwrite
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4) {
          id
          name
          searchComments(query: [["test", "tost"], ["foo"], ["bar"], ["bizz", "buzz"]])
        }
      }

# # Positve: defaults from input
# Disabled because Tailcall does not uses default values from Input Object
# - method: POST
#   url: http://localhost:8080/graphql
#   body:
#     query: |
#       query {
#         user(id: 4) {
#           id
#           name
#           featuredVideoPreview
#         }
#       }

# Negative: invalid size
# Disabled because async_graphql::dynamic does not perform validation
# - method: POST
#   url: http://localhost:8080/graphql
#   body:
#     query: |
#       query {
#         user(id: 4) {
#           id
#           name
#           profilePic(size: null)
#         }
#       }
```
