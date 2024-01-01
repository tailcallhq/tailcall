---
title: Mustache template with tailcall
---
Mustache templates are like fill-in-the-blank in forms for code. They let you put placeholders in your configurations, allowing dynamic content to be inserted. The dynamic content inserted within the curly braces ( {{ }} )
Learn more about mustache template [here](https://mustache.github.io/)

### Leveraging mustache template
#### Dynamic url Path 
Suppose you want to only get the data of a specific user by it's Id.

> [!NOTE] 
> The base url used for this example is https://jsonplaceholder.typicode.com

```graphql 
type Query {
  user(id: ID!): User @http(path: "/users/{{args.id}}")
}
```

When you run the `user` query with an ID argument, say `1`, the Mustache template `{{args.id}}` dynamically incorporates this ID into the URL. For instance, invoking `user(id: 1)` results in an HTTP request to `/users/1`, fetching the user data associated with the provided ID.

To execute it in the playground:

```graphql
query {
  user(id: 1) {
    id
    name
  }
}
```

#### Contextual Transformation

To get a list of todos you will run the query `todo` defined below.

> [!NOTE] 
> The base url used for this example is https://jsonplaceholder.typicode.com

```graphql
type Query {
  todos : [Todo] @http("/todos")
}
type Todo {
  id: ID!
  title: String!
  completed: Boolean!
}
```

Suppose you now only want the completed todos to be shown.You can accomplish this by utilizing the `completedTodos` query. This modified query includes a `completed` argument, allowing you to specify whether you want to retrieve completed or ongoing todos.

```graphql
type Query {
  completedTodos(completed: Boolean!): [Todo] @http(
    path: "/todos",
    query: [{ key: "completed", value: "{{args.completed}}" }]
  )
}
```

In the changed `completedTodos` query, we have added a way to ask for either completed or ongoing tasks. When you run this `completedTodos` query in the playground, you can ask for only the completed tasks by typing `completed: true` in the box where you normally put details. This way, you'll get a list that includes only the tasks that are finished.

#### Customization in Queries

Imagine a scenario where you need to fetch paginated data from an API

```graphql
type Query {
  paginatedPosts(page: Int!): [Post]
    @http(path: "/posts", query: [{key: "page", value: "{{args.page}}"}])
}
```
 when you run the query `paginatedPosts` it accepts a crucial argument, page, enabling the selection of specific pages of posts.Utilizing a Mustache template `/posts?page={{args.page}}`, the query dynamically generates a URL structure. When calling `paginatedPosts(page: 2)`, for instance, this template dynamically forms the URL `/posts?page=2`. This crafted URL then instructs the API to provide the posts located on the second page.

#### Dynamic input