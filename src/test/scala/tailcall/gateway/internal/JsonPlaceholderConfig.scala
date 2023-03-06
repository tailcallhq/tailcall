package tailcall.gateway.internal

import tailcall.gateway.ast.Path
import tailcall.gateway.dsl.json.Config
import tailcall.gateway.dsl.json.Config.Step

object JsonPlaceholderConfig {

  val users           = Config.Step.Http(Path.unsafe.fromString("/users"))
  val posts           = Config.Step.Http(Path.unsafe.fromString("/posts"))
  val userPosts: Step = Config.Step.Http(Path.unsafe.fromString("/users/{{id}}/posts"))
  val postUser: Step  = Config.Step.Http(Path.unsafe.fromString("/posts/{{id}}/user"))

  val graphQL = Config.GraphQL(
    schema = Config.SchemaDefinition(query = Some("Query"), mutation = Some("Mutation")),
    types = Map(
      "Query"   -> Map(
        "posts" -> Config.Field("Post", posts).asList,
        "users" -> Config.Field("User", users).asList
        // "post"  -> Config.Field("Post", postsById)("id" -> Argument.int.asRequired),
        // "user"  -> Config.Field("User", userById)("id" -> Argument.int.asRequired)
      ),
      "User"    -> Map(
        "id"       -> Config.Field.int.asRequired,
        "name"     -> Config.Field.string.asRequired,
        "username" -> Config.Field.string.asRequired,
        "email"    -> Config.Field.string.asRequired,
        "address"  -> Config.Field("Address"),
        "phone"    -> Config.Field.string,
        "website"  -> Config.Field.string,
        "company"  -> Config.Field("Company"),
        "posts"    -> Config.Field("Post", userPosts).asList
      ),
      "Post"    -> Map(
        "id"     -> Config.Field.int.asRequired,
        "userId" -> Config.Field.int.asRequired,
        "title"  -> Config.Field.string,
        "body"   -> Config.Field.string,
        "user"   -> Config.Field("User", postUser)
      ),
      "Address" -> Map(
        "street"  -> Config.Field.string,
        "suite"   -> Config.Field.string,
        "city"    -> Config.Field.string,
        "zipcode" -> Config.Field.string,
        "geo"     -> Config.Field("Geo")
      ),
      "Company" -> Map(
        "name"        -> Config.Field.string,
        "catchPhrase" -> Config.Field.string,
        "bs"          -> Config.Field.string
      ),
      "Geo"     -> Map("lat" -> Config.Field.string, "lng" -> Config.Field.string)
    )
  )

  val server = Config.Server(host = "jsonplaceholder.typicode.com", port = Option(443))
  val config = Config(server = server, graphQL = graphQL)
}
