package tailcall.gateway

import tailcall.gateway.dsl.json.Reader
import tailcall.gateway.internal.{FileExtension, TestGen}
import zio.test.TestAspect.{failing, timeout}
import zio.test._
import zio.{Scope, durationInt}

object ReaderSpec extends ZIOSpecDefault:
  override def spec: Spec[TestEnvironment with Scope, Any] =
    suite("Reader")(
      test("Config.yml is valid") {
        for _ <- Reader.config.readURL(getClass.getResource("Config.yml")) yield assertCompletes
      } @@ failing,
      test("Schema.graphql is valid")(
        for _ <- Reader.document.readURL(getClass.getResource("Schema.graphql")) yield assertCompletes
      ),
      test("YML Generator (debug)") {
        for
          config <- TestGen.genConfig.runHead
          _      <- FileExtension.YML.encode(config.get)
        yield assertCompletes
      } @@ failing
    ) @@ timeout(10.seconds)
