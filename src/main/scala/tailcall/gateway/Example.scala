package tailcall.gateway

import tailcall.gateway.ast._
import zio.schema.Schema

object Example extends App {
  val User = TSchema.structured(
    "id"    -> TSchema.int,
    "name"  -> TSchema.string,
    "phone" -> TSchema.string,
    "email" -> TSchema.string
  )

  val Post = TSchema
    .structured("id" -> TSchema.int, "title" -> TSchema.string, "body" -> TSchema.string)

  val typicode = Endpoint.inet("jsonplaceholder.typicode.com")

  val usersEndpoint: Endpoint = Endpoint
    .http(path = Path.unsafe.fromString("/users"), address = typicode, output = TSchema.arr(User))

  val postsEndpoint: Endpoint = Endpoint
    .http(path = Path.unsafe.fromString("/posts"), address = typicode, output = TSchema.arr(Post))

  val userPostsEndpoint = Endpoint.http(
    path = Path.unsafe.fromString("/users/{userId}/posts"),
    address = typicode,
    input = TSchema.structured("userId" -> TSchema.int),
    output = TSchema.arr(Post)
  )

  val manyUserPostsEndpoint = Endpoint.http(
    path = Path.unsafe.fromString("/posts"),
    query = Map("userId" -> "userId"),
    address = typicode,
    input = TSchema.structured("userId" -> TSchema.arr(TSchema.int)),
    output = TSchema.arr(Post)
  )

  val orch = Orch.obj(
    "Query" -> Orch
      .obj("users" -> Orch.endpoint(usersEndpoint), "posts" -> Orch.endpoint(postsEndpoint)),
    "User"  -> Orch.obj(
      "posts" -> Orch.spec("userId" -> Orch.context("id")).pipe(Orch.endpoint(userPostsEndpoint)),
      "fullName" -> (Orch.context("firstName") ++ Orch.context("lastName"))
    ),
    "Post"  -> Orch.obj(
      "user" -> Orch.batch(
        endpoint = Orch.spec("userId" -> Orch.context("userId"))
          .pipe(Orch.endpoint(manyUserPostsEndpoint)),
        groupBy = Orch.path("userId")
      )
    )
  )

  pprint.pprintln(Schema[Orch].toDynamic(orch))
}
