package tailcall.gateway

import tailcall.gateway.internal.{Extension, TestGen}
import tailcall.gateway.reader.ConfigReader
import zio.test.TestAspect.timeout
import zio.test._
import zio.{Scope, durationInt}

object ConfigReaderSpec extends ZIOSpecDefault {
  private val reader = ConfigReader.custom

  override def spec: Spec[TestEnvironment with Scope, Any] =
    suite("ConfigReader")(
      test("Config.yml is valid") {
        for {
          _ <- reader.readURL(getClass.getResource("Config.yml"))
        } yield assertCompletes
      },
      test("YML Generator (debug)") {
        for {
          config <- TestGen.genConfig.runHead
          _      <- Extension.YML.encode(config.get)
        } yield assertCompletes
      },
    ) @@ timeout(10 seconds)
}
