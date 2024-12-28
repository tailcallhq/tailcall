---
skip: true
---

# Test complex nested query.

TODO: Skipped because Tailcall does not send the whole query with the **fragments** to the remote server.

```yaml @config
server:
  port: 8001
  hostname: "0.0.0.0"
  queryValidation: false
upstream:
  httpCache: 42
```

```graphql @schema
schema {
  query: Query
}

type Query {
  user(id: ID!): User!
    @graphQL(url: "http://upstream/graphql", name: "user", args: [{key: "id", value: "{{.args.id}}"}])
}

type User {
  id: ID!
  name: String!
  profilePic(size: Int, width: Int, height: Int): String!
  friends(first: Int): [User!]!
  mutualFriends(first: Int): [User!]!
}
```

```yml @mock
- request:
    method: POST
    url: http://upstream/graphql
    textBody: '{ "query": "query { user(id: 4) { friends(first: 10) { id name profilePic(size: 50) } mutualFriends(first: 10) { id name profilePic(size: 50) } } }" }'
  expectedHits: 1
  response:
    status: 200
    body:
      data:
        user:
          friends:
            - id: 1
              name: friend_1
              profilePic: friend_1_pic
            - id: 2
              name: friend_2
              profilePic: friend_2_pic
            - id: 3
              name: friend_3
              profilePic: friend_3_pic
          mutualFriends:
            - id: 1
              name: mutual_friend_1
              profilePic: mutual_friend_1_pic
            - id: 2
              name: mutual_friend_2
              profilePic: mutual_friend_2_pic
            - id: 3
              name: mutual_friend_3
              profilePic: mutual_friend_3_pic
- request:
    method: POST
    url: http://upstream/graphql
    textBody: '{ "query": "query { user(id: 4) { friends(first: 10) { ...friendFields } mutualFriends(first: 10) { ...friendFields } } } fragment friendFields on User { id name profilePic(size: 50) }'
  expectedHits: 1
  response:
    status: 200
    body:
      data:
        user:
          friends:
            - id: 1
              name: friend_1
              profilePic: friend_1_pic
            - id: 2
              name: friend_2
              profilePic: friend_2_pic
            - id: 3
              name: friend_3
              profilePic: friend_3_pic
          mutualFriends:
            - id: 1
              name: mutual_friend_1
              profilePic: mutual_friend_1_pic
            - id: 2
              name: mutual_friend_2
              profilePic: mutual_friend_2_pic
            - id: 3
              name: mutual_friend_3
              profilePic: mutual_friend_3_pic
- request:
    method: POST
    url: http://upstream/graphql
    textBody: '{ "query": "query { user(id: 4) { friends(first: 10) { ...friendFields } mutualFriends(first: 10) { ...friendFields } } } fragment friendFields on User { id name ...standardProfilePic } fragment standardProfilePic on User { profilePic(size: 50) }'
  expectedHits: 1
  response:
    status: 200
    body:
      data:
        user:
          friends:
            - id: 1
              name: friend_1
              profilePic: friend_1_pic
            - id: 2
              name: friend_2
              profilePic: friend_2_pic
            - id: 3
              name: friend_3
              profilePic: friend_3_pic
          mutualFriends:
            - id: 1
              name: mutual_friend_1
              profilePic: mutual_friend_1_pic
            - id: 2
              name: mutual_friend_2
              profilePic: mutual_friend_2_pic
            - id: 3
              name: mutual_friend_3
              profilePic: mutual_friend_3_pic
```

```yml @test
# Positve: query
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4) {
          friends(first: 10) {
            id
            name
            profilePic(size: 50)
          }
          mutualFriends(first: 10) {
            id
            name
            profilePic(size: 50)
          }
        }
      }
# Positve: fragment simple
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4) {
          friends(first: 10) {
            ...friendFields
          }
          mutualFriends(first: 10) {
            ...friendFields
          }
        }
      }

      fragment friendFields on User {
        id
        name
        profilePic(size: 50)
      }
# Positve: fragment nested
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4) {
          friends(first: 10) {
            ...friendFields
          }
          mutualFriends(first: 10) {
            ...friendFields
          }
        }
      }

      fragment friendFields on User {
        id
        name
        ...standardProfilePic
      }

      fragment standardProfilePic on User {
        profilePic(size: 50)
      }
# Negative: missing fragment
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4) {
          friends(first: 10) {
            ...friendFields
          }
          mutualFriends(first: 10) {
            ...friendFields
          }
        }
      }

      fragment friendFields on User {
        id
        name
        ...standardProfilePic
      }
```
