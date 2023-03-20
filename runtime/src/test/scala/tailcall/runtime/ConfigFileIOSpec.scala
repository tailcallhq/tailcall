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
        checkAll(Gen.fromIterable(Seq(DSLFormat.YML, DSLFormat.JSON, DSLFormat.GRAPHQL))) { format =>
          for {
            config  <- ConfigFileIO.readURL(getClass.getResource(s"Config.${format.ext}"))
            string  <- format.encode(config)
            config1 <- format.decode(string)
          } yield assertTrue(config == config1)
        }
      },
      test("equals placeholder config") {
        val gen = Gen.fromIterable(Seq(
          DSLFormat.YML,    //
          DSLFormat.JSON,   //
          DSLFormat.GRAPHQL //
        ))

        checkAll(gen) { format =>
          for {
            actual <- ConfigFileIO.readURL(getClass.getResource(s"Config.${format.ext}")).map(_.compress)
            expected = JsonPlaceholderConfig.config.compress
          } yield assertTrue(actual == expected)
        }
      }
    ).provide(
        ConfigFileIO.live,
        FileIO.live,
        GraphQLGenerator.live,
        StepGenerator.live,
        EvaluationRuntime.live
      ) @@ timeout(5 seconds)
}
