package tailcall.gateway

import tailcall.gateway.ast.{Endpoint, TGraph, TSchema}
import tailcall.gateway.remote.Remote
import zio.test._

object OrcSpec extends ZIOSpecDefault {

  object schema {
    val User = TSchema.obj("id" -> TSchema.int, "name" -> TSchema.str)

    val Post = TSchema.obj(
      "id"     -> TSchema.int,
      "title"  -> TSchema.str,
      "body"   -> TSchema.str,
      "userId" -> TSchema.int
    )
  }

  object endpoints {
    val typicode  = Endpoint.make("jsonplaceholder.typicode.com")
    val users     = typicode.withPath("/users").withOutput(TSchema.arr(schema.User))
    val posts     = typicode.withPath("/posts").withOutput(TSchema.arr(schema.Post))
    val userPosts = typicode
      .withPath("/posts")
      .withQuery("userId" -> "${userId}")
      .withInput(TSchema.obj("userId" -> TSchema.int))
      .withOutput(TSchema.arr(schema.Post))
  }

  val unit = Remote.dynamicValue(())

  val query = TGraph.query(
    "Query" -> List(
      "users" -> (_ => endpoints.users(unit)),
      "posts" -> (_ => endpoints.posts(unit))
    ),
    "User"  -> List(
      "posts"    -> { context => endpoints.userPosts(Remote.record("userId" -> context.value)) },
      "comments" -> { context =>
        endpoints.userPosts(Remote.record("userId" -> context.value.path("id")))
      }
    )
  )

  pprint.pprintln(query)

  def spec = suite("OrchSpec")(test("test")(assertCompletes))
}
