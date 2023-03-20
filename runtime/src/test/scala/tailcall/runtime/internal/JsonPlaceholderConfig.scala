package tailcall.runtime.internal

import tailcall.runtime.ast.Path
import tailcall.runtime.dsl.json.Config
import tailcall.runtime.dsl.json.Config.{Argument, Step}

object JsonPlaceholderConfig {
  val users           = Config.Step.Http(Path.unsafe.fromString("/users"))
  val userById        = Config.Step.Http(Path.unsafe.fromString("/users/{{userId}}"))
  val postsById       = Config.Step.Http(Path.unsafe.fromString("/posts/{{args.id}}"))
  val posts           = Config.Step.Http(Path.unsafe.fromString("/posts"))
  val userPosts: Step = Config.Step.Http(Path.unsafe.fromString("/users/{{value.id}}/posts"))

  val graphQL = Config.GraphQL(
    schema = Config.SchemaDefinition(query = Some("Query")),
    types = Map(
      "Query"   -> Map(
        "posts" -> Config.Field("Post", posts).asList,
        "users" -> Config.Field("User", users).asList,
        "post"  -> Config.Field("Post", postsById)("id" -> Argument.int.asRequired),
        "user"  -> Config
          .Field("User", Config.Step.ObjPath("userId" -> List("args", "id")), userById)("id" -> Argument.int.asRequired)
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
        "user"   -> Config.Field("User", Config.Step.ObjPath("userId" -> List("value", "userId")), userById)
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

  val server = Config.Server(host = Option("jsonplaceholder.typicode.com"), port = Option(443))
  val config = Config(server = server, graphQL = graphQL)
}
