import tailcall.gateway.Loader
import tailcall.gateway.Loader.Extension
import zio.Scope
import zio.test._

object LoaderSpec extends ZIOSpecDefault {
  override def spec: Spec[TestEnvironment with Scope, Any] =
    suite("encode / decode")(
      test("codec") {
        val ext = Extension.JSON
        for {
          json    <- Loader.read("Config.json")
          config  <- ext.decode(json)
          json0   <- ext.encode(config)
          config0 <- ext.decode(json0)
        } yield assertTrue(config0 == config)
      },
      test("json == yml") {
        val ext = Extension.JSON
        for {
          json    <- Loader.read("Config.json")
          config  <- Extension.JSON.decode(json)
          yml     <- Extension.YML.encode(config)
          config0 <- Extension.YML.decode(yml)
        } yield assertTrue(config0 == config)
      },
    )
}
