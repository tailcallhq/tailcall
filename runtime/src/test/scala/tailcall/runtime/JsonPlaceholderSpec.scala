package tailcall.runtime

import caliban.InputValue
import tailcall.runtime.internal.JsonPlaceholderConfig
import tailcall.runtime.model.Config.{Arg, Field, Type}
import tailcall.runtime.model.UnsafeSteps.Operation.Http
import tailcall.runtime.model.{Config, ConfigFormat}
import tailcall.runtime.service._
import tailcall.runtime.transcoder.Transcoder
import zio.test.Assertion.equalTo
import zio.test._
import zio.{Scope, ZIO}

import java.io.{File, FileNotFoundException}

object JsonPlaceholderSpec extends ZIOSpecDefault {
  private val typicode = Config.default.withBaseURL("https://jsonplaceholder.typicode.com")

  override def spec: Spec[TestEnvironment with Scope, Any] =
    suite("JsonPlaceholder")(
      test("Config.yml is valid Config")(ConfigFileIO.readURL(getClass.getResource("Config.yml")).as(assertCompletes)),
      test("Config.json is valid Config")(
        ConfigFileIO.readURL(getClass.getResource("Config.json")).as(assertCompletes)
      ),
      test("Config.graphql is valid Config")(
        ConfigFileIO.readURL(getClass.getResource("Config.graphql")).as(assertCompletes)
      ),
      test("read write identity") {
        checkAll(Gen.fromIterable(ConfigFormat.all)) { format =>
          for {
            config  <- ConfigFileIO.readURL(getClass.getResource(s"Config.${format.ext}"))
            string  <- format.encode(config)
            config1 <- format.decode(string)
          } yield assertTrue(config == config1)
        }
      },
      test("equals placeholder config") {
        val sourceConfig = JsonPlaceholderConfig.config.compress
        checkAll(Gen.fromIterable(ConfigFormat.all)) { format =>
          for {
            config <- ConfigFileIO.readURL(getClass.getResource(s"Config.${format.ext}")).map(_.compress)
          } yield assertTrue(config == sourceConfig)
        }
      },
      test("equals placeholder schema") {
        val sourceConfig = JsonPlaceholderConfig.config.compress
        checkAll(Gen.fromIterable(ConfigFormat.all)) { format =>
          for {
            config   <- ConfigFileIO.readURL(getClass.getResource(s"Config.${format.ext}"))
            actual   <- format.encode(config)
            expected <- format.encode(sourceConfig)
          } yield assertTrue(actual == expected)
        }
      },

      // NOTE: This test just re-writes the configuration files
      test("write generated config") {
        val config = JsonPlaceholderConfig.config
        checkAll(Gen.fromIterable(ConfigFormat.all)) { format =>
          val url = new File(s"src/test/resources/tailcall/runtime/Config.${format.ext}")
          ConfigFileIO.write(url, config).as(assertCompletes)
        }
      },
      test("output schema") {
        val config   = JsonPlaceholderConfig.config
        val expected = """|schema {
                          |  query: Query
                          |  mutation: Mutation
                          |}
                          |
                          |input NewAddress {
                          |  city: String
                          |  geo: NewGeo
                          |  street: String
                          |  suite: String
                          |  zipcode: String
                          |}
                          |
                          |input NewCompany {
                          |  bs: String
                          |  catchPhrase: String
                          |  name: String
                          |}
                          |
                          |input NewGeo {
                          |  lat: String
                          |  lng: String
                          |}
                          |
                          |"A new user."
                          |input NewUser {
                          |  address: NewAddress
                          |  company: NewCompany
                          |  email: String!
                          |  name: String!
                          |  phone: String
                          |  username: String!
                          |  website: String
                          |}
                          |
                          |type Address {
                          |  city: String
                          |  geo: Geo
                          |  street: String
                          |  suite: String
                          |  zip: String
                          |}
                          |
                          |type Company {
                          |  bs: String
                          |  catchPhrase: String
                          |  name: String
                          |}
                          |
                          |type Geo {
                          |  lat: String
                          |  lng: String
                          |}
                          |
                          |"An Id container."
                          |type Id {
                          |  id: Int!
                          |}
                          |
                          |type Mutation {
                          |  createUser("User as an argument." user: NewUser!): Id
                          |}
                          |
                          |type Post {
                          |  body: String
                          |  id: Int!
                          |  title: String
                          |  user: User
                          |  userId: Int!
                          |}
                          |
                          |type Query {
                          |  "A single post by id."
                          |  post(id: Int!): Post
                          |  "A list of all posts."
                          |  posts: [Post]
                          |  "A single user by id."
                          |  user(id: Int!): User
                          |  "A list of all users."
                          |  users: [User]
                          |}
                          |
                          |type User {
                          |  address: Address
                          |  company: Company
                          |  email: String!
                          |  id: Int!
                          |  name: String!
                          |  phone: String
                          |  posts: [Post]
                          |  username: String!
                          |  website: String
                          |}
                          |""".stripMargin.trim

        for { actual <- Transcoder.toSDL(config, false).toTask } yield assertTrue(actual == expected)
      },
      test("users name") {
        val program = resolve(JsonPlaceholderConfig.config)(""" query { users {name} } """)

        val expected = """{"users":[
                         |{"name":"Leanne Graham"},
                         |{"name":"Ervin Howell"},
                         |{"name":"Clementine Bauch"},
                         |{"name":"Patricia Lebsack"},
                         |{"name":"Chelsey Dietrich"},
                         |{"name":"Mrs. Dennis Schulist"},
                         |{"name":"Kurtis Weissnat"},
                         |{"name":"Nicholas Runolfsdottir V"},
                         |{"name":"Glenna Reichert"},
                         |{"name":"Clementina DuBuque"}
                         |]}""".stripMargin.replace("\n", "").trim
        assertZIO(program)(equalTo(expected))
      },
      test("user name") {
        val program = resolve(JsonPlaceholderConfig.config)(""" query { user(id: 1) {name} } """)
        assertZIO(program)(equalTo("""{"user":{"name":"Leanne Graham"}}"""))
      },
      test("post body") {
        val program  = resolve(JsonPlaceholderConfig.config)(""" query { post(id: 1) { title } } """)
        val expected =
          """{"post":{"title":"sunt aut facere repellat provident occaecati excepturi optio reprehenderit"}}"""
        assertZIO(program)(equalTo(expected))
      },
      test("user company") {
        val program  = resolve(JsonPlaceholderConfig.config)(""" query {user(id: 1) { company { name } } }""")
        val expected = """{"user":{"company":{"name":"Romaguera-Crona"}}}"""
        assertZIO(program)(equalTo(expected))
      },
      test("user posts") {
        val program  = resolve(JsonPlaceholderConfig.config)("""query {user(id: 1) { posts { title } } }""")
        val expected =
          """{"user":{"posts":[{"title":"sunt aut facere repellat provident occaecati excepturi optio reprehenderit"},
            |{"title":"qui est esse"},
            |{"title":"ea molestias quasi exercitationem repellat qui ipsa sit aut"},
            |{"title":"eum et est occaecati"},
            |{"title":"nesciunt quas odio"},
            |{"title":"dolorem eum magni eos aperiam quia"},
            |{"title":"magnam facilis autem"},
            |{"title":"dolorem dolore est ipsam"},
            |{"title":"nesciunt iure omnis dolorem tempora et accusantium"},
            |{"title":"optio molestias id quia eum"}]}}""".stripMargin.replace("\n", "").trim
        assertZIO(program)(equalTo(expected))
      },
      test("post user") {
        val program  = resolve(JsonPlaceholderConfig.config)(""" query {post(id: 1) { title user { name } } }""")
        val expected =
          """{"post":{"title":"sunt aut facere repellat provident occaecati excepturi optio reprehenderit","user":{"name":"Leanne Graham"}}}"""
        assertZIO(program)(equalTo(expected))
      },
      test("create user") {
        val program = resolve(JsonPlaceholderConfig.config)(
          """ mutation { createUser(user: {name: "test", email: "test@abc.com", username: "test"}) { id } } """
        )
        assertZIO(program)(equalTo("""{"createUser":{"id":11}}"""))
      },
      test("create user with zip code") {
        val program = resolve(JsonPlaceholderConfig.config)(
          """ mutation { createUser(user: {name: "test", email: "test@abc.com", username: "test", address: {zipcode: "1234-4321"}}) { id } } """
        )
        assertZIO(program)(equalTo("""{"createUser":{"id":11}}"""))
      },
      test("user zipcode") {
        val program  = resolve(JsonPlaceholderConfig.config)("""query { user(id: 1) { address { zip } } }""")
        val expected = """{"user":{"address":{"zip":"92998-3874"}}}"""
        assertZIO(program)(equalTo(expected))
      },
      suite("batching")(
        test("many users to posts") {
          val users           = Http.fromPath("/users")
          val userPostBatched = Http.fromPath("/posts").withQuery("userId" -> "{{parent.value.id}}").withBatchKey("id")
            .withGroupBy("userId")

          val config = typicode.withTypes(
            "Query" -> Type("users" -> Field.ofType("User").asList.withHttp(users)),
            "User"  -> Type("id" -> Field.int, "posts" -> Field.ofType("Post").asList.withHttp(userPostBatched)),
            "Post"  -> Type("userId" -> Field.int, "title" -> Field.str),
          )

          for {
            actual   <- resolve(config)("""query {users { id posts { userId title } }}""")
            expected <- readJson("user-posts-batched.json")
          } yield assertTrue(actual == expected)
        },
        test("single user to posts") {
          val user            = Http.fromPath("/users/{{args.id}}")
          val userPostBatched = Http.fromPath("/posts").withQuery("userId" -> "{{parent.value.id}}").withBatchKey("id")
            .withGroupBy("userId")

          val config = typicode.withTypes(
            "Query" -> Type("user" -> Field.ofType("User").withHttp(user).withArguments("id" -> Arg.int.asRequired)),
            "User"  -> Type("id" -> Field.int, "posts" -> Field.ofType("Post").asList.withHttp(userPostBatched)),
            "Post"  -> Type("userId" -> Field.int, "title" -> Field.str),
          )

          for {
            actual   <- resolve(config)("""query {user(id: 1) { id posts {userId title} }}""")
            expected <- readJson("user-posts-single.json")
          } yield assertTrue(actual == expected)
        },
        test("switching between single resolver and batched resolver") {
          val user            = Http.fromPath("/users/{{args.id}}")
          val users           = Http.fromPath("/users")
          val userPostBatched = Http.fromPath("/posts").withQuery("userId" -> "{{parent.value.id}}").withBatchKey("id")
            .withGroupBy("userId")

          val config = typicode.withTypes(
            "Query" -> Type(
              "user"  -> Field.ofType("User").withHttp(user).withArguments("id" -> Arg.int.asRequired),
              "users" -> Field.ofType("User").asList.withHttp(users),
            ),
            "User"  -> Type("id" -> Field.int, "posts" -> Field.ofType("Post").asList.withHttp(userPostBatched)),
            "Post"  -> Type("userId" -> Field.int, "title" -> Field.str),
          )

          for {
            actual   <- resolve(config)("""query { users { posts { title } } user (id: 1) { posts { title } } }""")
            expected <- readJson("user-posts-single-vs-batched.json")
          } yield assertTrue(actual == expected)
        },
        test("multiple posts to user") {
          val postUser = Http.fromPath("/users").withQuery("id" -> "{{parent.value.userId}}").withBatchKey("userId")
            .withGroupBy("id")
          val posts    = Http.fromPath("/posts")
          val config   = typicode.withTypes(
            "Query" -> Type("posts" -> Field.ofType("Post").asList.withHttp(posts)),
            "User"  -> Type("id" -> Field.int, "name" -> Field.str),
            "Post"  -> Type(
              "userId" -> Field.int,
              "title"  -> Field.str,
              "user"   -> Field.ofType("User").withHttp(postUser),
            ),
          )

          for {
            actual   <- resolve(config)("""query {posts { userId user { id name } } }""")
            expected <- readJson("posts-user-batched.json")
          } yield assertTrue(actual == expected)
        },
      ),
    ).provide(ConfigFileIO.default, GraphQLGenerator.default, HttpContext.default, FileIO.default)

  private def readJson(name: String): ZIO[FileIO, Throwable, String] = {
    for {
      path    <- ZIO.attempt(getClass.getResource(s"assertions/${name}").toURI.getPath)
        .refineOrDie { case _: NullPointerException => new FileNotFoundException(s"File $name not found") }
      file    <- ZIO.attempt(new File(path))
      content <- FileIO.read(file)
    } yield content
  }

  private def resolve(config: Config, variables: Map[String, InputValue] = Map.empty)(
    query: String
  ): ZIO[HttpContext with GraphQLGenerator, Throwable, String] = {
    for {
      blueprint   <- Transcoder.toBlueprint(config).toTask
      graphQL     <- blueprint.toGraphQL
      interpreter <- graphQL.interpreter
      result      <- interpreter.execute(query, variables = variables)

      _ <- result.errors.headOption match {
        case Some(error) => ZIO.fail(error)
        case None        => ZIO.unit
      }
    } yield result.data.toString
  }
}
