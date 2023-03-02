package tailcall.gateway.internal

import tailcall.gateway.ast.Endpoint
import tailcall.gateway.dsl.scala.Orc
import tailcall.gateway.dsl.scala.Orc.Field
import zio.schema.{Schema, DeriveSchema}

object JsonPlaceholder:
  final case class User(id: Int, name: String)
  object User:
    implicit val schema: Schema[User] = DeriveSchema.gen[User]
  final case class Post(id: Int, title: String, body: String, userId: Int)
  object Post:
    implicit val schema: Schema[Post] = DeriveSchema.gen[Post]
  final case class Comment(id: Int, name: String, email: String, body: String, postId: Int)
  object Comment:
    implicit val schema: Schema[Comment] = DeriveSchema.gen[Comment]
  final case class Album(id: Int, title: String, userId: Int)
  object Album:
    implicit val schema: Schema[Album] = DeriveSchema.gen[Album]
  final case class Photo(id: Int, title: String, url: String, thumbnailUrl: String, albumId: Int)
  object Photo:
    implicit val schema: Schema[Photo] = DeriveSchema.gen[Photo]
  final case class Todo(id: Int, title: String, completed: Boolean, userId: Int)
  object Todo:
    implicit val schema: Schema[Todo] = DeriveSchema.gen[Todo]
  final case class UserId(userId: Int)
  object UserId:
    implicit val schema: Schema[UserId] = DeriveSchema.gen[UserId]
  final case class PostId(postId: Int)
  object PostId:
    implicit val schema: Schema[PostId] = DeriveSchema.gen[PostId]
  final case class EmailId(email: String)
  object EmailId:
    implicit val schema: Schema[EmailId] = DeriveSchema.gen[EmailId]
  final case class AlbumId(albumId: Int)
  object AlbumId:
    implicit val schema: Schema[AlbumId] = DeriveSchema.gen[AlbumId]

  object endpoints:
    val typicode: Endpoint  = Endpoint.make("jsonplaceholder.typicode.com")
    val users: Endpoint     = typicode.withPath("/users").withOutput[List[User]]
    val posts: Endpoint     = typicode.withPath("/posts").withOutput[List[Post]]
    val userPosts: Endpoint = typicode.withPath("/posts").withQuery("userId" -> "${userId}").withInput[UserId]
      .withOutput[List[Post]]

    val postComments: Endpoint = typicode.withPath("/comments").withQuery("postId" -> "${postId}").withInput[PostId]
      .withOutput[List[Comment]]

    val userComments: Endpoint = typicode.withPath("/comments").withQuery("email" -> "${email}").withInput[EmailId]
      .withOutput[List[Comment]]

    val userAlbums: Endpoint = typicode.withPath("/albums").withQuery("userId" -> "${userId}").withInput[UserId]
      .withOutput[List[Album]]

    val UserTodos: Endpoint = typicode.withPath("/todos").withQuery("userId" -> "${userId}").withInput[UserId]
      .withOutput[List[Todo]]

    val AlbumPhotos: Endpoint = typicode.withPath("/photos").withQuery("albumId" -> "${albumId}").withInput[AlbumId]
      .withOutput[List[Photo]]

    val ManyUsersPosts: Endpoint = typicode.withPath("/posts").withQuery("userId" -> "${userId}").withInput[UserId]
      .withOutput[List[Post]]

  object mocks {}

  val orc =
    // scalafmt: {maxColumn: 80}
    Orc(
      "Query" -> List(
        "users" -> Field.output.asList("User"),
        "posts" -> Field.output.asList("Post")
      ),
      "User"  -> List(
        "posts"    -> Field.output.asList("Post"),
        "fullName" -> Field.output.asList("String"),
        "comments" -> Field.output.asList("Comment"),
        "albums"   -> Field.output.asList("Album"),
        "todos"    -> Field.output.asList("Todo")
      )
    )

//    Orc.obj(
//      "Query" -> List(
//        "users" -> Orc.fromRemote(endpoints.users(unit)),
//        "posts" -> Orc.fromRemote(endpoints.posts(unit))
//      ),
//      "User" -> List(
//        "posts" -> Orc.fromContext { context =>
//          endpoints.userPosts(Remote.record("userId" -> context.value))
//        },
//        "fullName" -> Orc.fromContext { context =>
//          val fn = context.value.path("firstName").flatMap(_.asString).getOrDie
//          val ln = context.value.path("lastName").flatMap(_.asString).getOrDie
//          Remote.dynamicValue(fn ++ Remote(" ") ++ ln)
//        },
//        "comments" -> Orc.fromContext { context =>
//          endpoints.userComments(
//            Remote.record("email" -> context.value.path("email").getOrDie)
//          )
//        },
//        "albums" -> Orc.fromContext { context =>
//          endpoints.userAlbums(
//            Remote.record("userId" -> context.value.path("id").getOrDie)
//          )
//        },
//        "todos" -> Orc.fromContext { context =>
//          endpoints.UserTodos(
//            Remote.record("userId" -> context.value.path("id").getOrDie)
//          )
//        }
//      ),
//      "Post" -> List("comments" -> Orc.fromContext { context =>
//        endpoints.postComments(
//          Remote.record("postId" -> context.value.path("id").getOrDie)
//        )
//      }),
//      "Album" -> List("photos" -> Orc.fromContext { context =>
//        endpoints.AlbumPhotos(
//          Remote.record("albumId" -> context.value.path("id").getOrDie)
//        )
//      })
//    )
