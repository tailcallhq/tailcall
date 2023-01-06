package tailcall.gateway

import tailcall.gateway.Loader.Extension
import tailcall.gateway.internal.TestGen
import zio.test.TestAspect.{failing, timeout}
import zio.test._
import zio.{Scope, durationInt}

object LoaderSpec extends ZIOSpecDefault {
  // TODO: fix failing tests
  override def spec: Spec[TestEnvironment with Scope, Any] =
    suite("LoaderSpec")(
      test("codec") {
        val ext = Extension.JSON
        for {
          json    <- Loader.readFile("Config.json")
          config  <- ext.decode(json)
          json0   <- ext.encode(config)
          config0 <- ext.decode(json0)
        } yield assertTrue(config0 == config)
      } @@ failing,
      test("json == yml") {
        val ext = Extension.JSON
        for {
          json    <- Loader.readFile("Config.json")
          config  <- Extension.JSON.decode(json)
          yml     <- Extension.YML.encode(config)
          config0 <- Extension.YML.decode(yml)
        } yield assertTrue(config0 == config)
      } @@ failing,
      test("Config.yml is valid") {
        val ext = Extension.YML
        for {
          file   <- Loader.readFile("Config.yml")
          config <- ext.decode(file)
          _ = pprint.pprintln(config)
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
