package tailcall.gateway

import tailcall.gateway.ast.{Endpoint, Orc, TSchema}
import zio.test._

object OrcSpec extends ZIOSpecDefault {

  // STRUCTURAL TYPES
  val User = TSchema.obj("id" -> TSchema.int, "name" -> TSchema.str)
  val Post = TSchema.obj(
    "id"     -> TSchema.int,
    "title"  -> TSchema.str,
    "body"   -> TSchema.str,
    "userId" -> TSchema.int
  )

  // ENDPOINTS

  // Unit -> Unit
  val typicode = Endpoint.make("jsonplaceholder.typicode.com")

  // Unit -> Array[User]
  val users = typicode.withPath("/users").withOutput(TSchema.arr(User))

  // Unit -> Array[Post]
  val posts = typicode.withPath("/posts").withOutput(TSchema.arr(Post))

  // { userId: Int } -> Array[Post
  val userPosts = typicode
    .withPath("/posts")
    .withQuery("userId" -> "${userId}")
    .withInput(TSchema.obj("userId" -> TSchema.int))
    .withOutput(TSchema.arr(Post))

  // ORCHESTRATIONS
  val query = Orc.obj("Query", "users" -> Orc.endpoint(users), "posts" -> Orc.endpoint(posts))
  val user  = Orc.obj("User", "posts" -> Orc.endpoint(userPosts))

  pprint.pprintln(user)

  def spec = suite("OrchSpec")(test("test")(assertCompletes))
}
