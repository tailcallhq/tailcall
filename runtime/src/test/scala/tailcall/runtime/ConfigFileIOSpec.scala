package tailcall.runtime

import tailcall.runtime.internal.JsonPlaceholderConfig
import tailcall.runtime.model.Config.Field
import tailcall.runtime.model.{Config, Path, Step, TSchema}
import tailcall.runtime.service._
import zio.test.TestAspect.timeout
import zio.test._
import zio.{Scope, durationInt}

import java.io.File
import java.net.URL

object ConfigFileIOSpec extends ZIOSpecDefault {
  override def spec: Spec[TestEnvironment with Scope, Any] =
    suite("ConfigFileIO")(
      test("Config.yml is valid Config")(ConfigFileIO.readURL(getClass.getResource("Config.yml")).as(assertCompletes)),
      test("Config.json is valid Config")(
        ConfigFileIO.readURL(getClass.getResource("Config.json")).as(assertCompletes)
      ),
      test("Config.graphql is valid Config")(
        ConfigFileIO.readURL(getClass.getResource("Config.graphql")).as(assertCompletes)
      ),
      test("read write identity") {
        checkAll(Gen.fromIterable(DSLFormat.all)) { format =>
          for {
            config  <- ConfigFileIO.readURL(getClass.getResource(s"Config.${format.ext}"))
            string  <- format.encode(config)
            config1 <- format.decode(string)
          } yield assertTrue(config == config1)
        }
      },
      test("equals placeholder config") {
        val sourceConfig = JsonPlaceholderConfig.config.compress
        checkAll(Gen.fromIterable(DSLFormat.all)) { format =>
          for {
            config   <- ConfigFileIO.readURL(getClass.getResource(s"Config.${format.ext}")).map(_.compress)
            actual   <- format.encode(config)
            expected <- format.encode(sourceConfig)
          } yield assertTrue(config == sourceConfig, actual == expected)
        }
      },

      // NOTE: This test just re-writes the configuration files
      test("write generated config") {
        val config = JsonPlaceholderConfig.config.compress
        checkAll(Gen.fromIterable(DSLFormat.all)) { format =>
          // TODO: find a better way to get the path instead of hardcoding
          val url = new File(s"src/test/resources/tailcall/runtime/Config.${format.ext}")
          ConfigFileIO.write(url, config).as(assertCompletes)
        }
      },
      suite("compression")(test("http with schema") {
        val step     = Step.Http(path = Path.unsafe.fromString("/foo"), output = Option(TSchema.string))
        val config   = Config.default.withTypes("Query" -> Config.Type("foo" -> Field.ofType("String").withSteps(step)))
        val actual   = config.compress
        val expected = config
        assertTrue(actual == expected)
      }),
      test("directive definition") {
        val config = Config.default.withBaseURL(new URL("http://localhost:8080/graphql"))

        val expected = """|directive @server(baseURL: String) on SCHEMA
                          |directive @modify(name: String, omit: Boolean) on FIELD_DEFINITION | INPUT_FIELD_DEFINITION
                          |""".stripMargin.trim
        config.asGraphQLConfig.map(actual => assertTrue(actual.trim == expected))
      },
    ).provide(
        ConfigFileIO.live,
        FileIO.default,
        GraphQLGenerator.live,
        StepGenerator.live,
        EvaluationRuntime.default,
      ) @@ timeout(5 seconds)
}
