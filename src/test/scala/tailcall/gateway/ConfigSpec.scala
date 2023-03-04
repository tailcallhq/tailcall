package tailcall.gateway

import tailcall.gateway.ast.Path
import tailcall.gateway.dsl.json.Config
import tailcall.gateway.dsl.json.Config.Operation
import tailcall.gateway.internal.Extension
import zio.test.{ZIOSpecDefault, assertTrue}

object ConfigSpec extends ZIOSpecDefault {
  override def spec =
    suite("ConfigSpec")(test("encoding") {

      val getUsers    = Config.Operation.Http(Path.unsafe.fromString("/users"))
      val getPosts    = Config.Operation.Http(Path.unsafe.fromString("/posts"))
      val getComments = Config.Operation.Http(Path.unsafe.fromString("/comments"))
      val getAlbums   = Config.Operation.Http(Path.unsafe.fromString("/albums"))
      val getPhotos   = Config.Operation.Http(Path.unsafe.fromString("/photos"))

      val userPosts: Operation    = Config.Operation.Http(Path.unsafe.fromString("/users/{id}/posts"))
      val userComments: Operation = Config.Operation.Http(Path.unsafe.fromString("/users/{id}/comments"))
      val userAlbums: Operation   = Config.Operation.Http(Path.unsafe.fromString("/users/{id}/albums"))

      val postComments: Operation = Config.Operation.Http(Path.unsafe.fromString("/posts/{id}/comments"))
      val postUser: Operation     = Config.Operation.Http(Path.unsafe.fromString("/posts/{id}/user"))

      val commentPost = Config.Operation.Http(Path.unsafe.fromString("/comments/{id}/post"))
      val photoAlbum  = Config.Operation.Http(Path.unsafe.fromString("/photos/{id}/album"))

      val graphQL = Config.GraphQL(
        schema = Config.SchemaDefinition(query = Some("Query"), mutation = Some("Mutation")),
        types = Map(
          "Query"   -> Map(
            "users"    -> Config.FieldDefinition("User", getUsers),
            "posts"    -> Config.FieldDefinition("Post", getPosts),
            "comments" -> Config.FieldDefinition("Comment", getComments),
            "albums"   -> Config.FieldDefinition("Album", getAlbums),
            "photos"   -> Config.FieldDefinition("Photo", getPhotos)
          ),
          "User"    ->
            Map(
              "id"       -> Config.FieldDefinition("String"),
              "name"     -> Config.FieldDefinition("String"),
              "email"    -> Config.FieldDefinition("String"),
              "address"  -> Config.FieldDefinition("Address"),
              "phone"    -> Config.FieldDefinition("String"),
              "website"  -> Config.FieldDefinition("String"),
              "company"  -> Config.FieldDefinition("Company"),
              "posts"    -> Config.FieldDefinition("Post", userPosts),
              "comments" -> Config.FieldDefinition("Comment", userComments),
              "albums"   -> Config.FieldDefinition("Album", userAlbums)
            ),
          "Post"    -> Map(
            "id"       -> Config.FieldDefinition("String"),
            "userId"   -> Config.FieldDefinition("Id"),
            "title"    -> Config.FieldDefinition("String"),
            "body"     -> Config.FieldDefinition("String"),
            "user"     -> Config.FieldDefinition("User", postUser),
            "comments" -> Config.FieldDefinition("Comment", postComments)
          ),
          "Address" -> Map(
            "street"  -> Config.FieldDefinition("String"),
            "suite"   -> Config.FieldDefinition("String"),
            "city"    -> Config.FieldDefinition("String"),
            "zipcode" -> Config.FieldDefinition("String"),
            "geo"     -> Config.FieldDefinition("Geo")
          ),
          "Company" -> Map(
            "name"        -> Config.FieldDefinition("String"),
            "catchPhrase" -> Config.FieldDefinition("String"),
            "bs"          -> Config.FieldDefinition("String")
          ),
          "Geo"     -> Map("lat" -> Config.FieldDefinition("String"), "lng" -> Config.FieldDefinition("String")),
          "Comment" -> Map(
            "id"     -> Config.FieldDefinition("String"),
            "postId" -> Config.FieldDefinition("Id"),
            "name"   -> Config.FieldDefinition("String"),
            "email"  -> Config.FieldDefinition("String"),
            "body"   -> Config.FieldDefinition("String"),
            "post"   -> Config.FieldDefinition("Post", commentPost)
          ),
          "Photo"   -> Map(
            "id"           -> Config.FieldDefinition("String"),
            "albumId"      -> Config.FieldDefinition("Id"),
            "title"        -> Config.FieldDefinition("String"),
            "url"          -> Config.FieldDefinition("String"),
            "thumbnailUrl" -> Config.FieldDefinition("String"),
            "album"        -> Config.FieldDefinition("Album", photoAlbum)
          )
        )
      )

      val server = Config.Server("https://jsonplaceholder.typicode.com/")
      val config = Config(server = server, graphQL = graphQL)

      val extension = Extension.YML

      for {
        encoded <- extension.encode(config)
        _ = pprint.pprintln(encoded)
        decoded <- extension.decode(encoded)
      } yield assertTrue(decoded == config)
    })
}
