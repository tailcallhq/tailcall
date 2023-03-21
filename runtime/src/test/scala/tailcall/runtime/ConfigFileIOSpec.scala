package tailcall.runtime

import tailcall.runtime.internal.JsonPlaceholderConfig
import tailcall.runtime.service._
import zio.test.TestAspect.timeout
import zio.test._
import zio.{Scope, durationInt}

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
        checkAll(Gen.fromIterable(DSLFormat.all)) { format =>
          for {
            actual <- ConfigFileIO.readURL(getClass.getResource(s"Config.${format.ext}")).map(_.compress)
            expected = JsonPlaceholderConfig.config.compress
          } yield assertTrue(actual == expected)
        }
      }
    ).provide(
        ConfigFileIO.live,
        FileIO.default,
        GraphQLGenerator.live,
        StepGenerator.live,
        EvaluationRuntime.default
      ) @@ timeout(5 seconds)
}
