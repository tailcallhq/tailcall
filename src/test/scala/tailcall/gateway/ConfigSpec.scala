package tailcall.gateway

import tailcall.gateway.internal.{Extension, JsonPlaceholderConfig}
import zio.test.{ZIOSpecDefault, assertTrue}

object ConfigSpec extends ZIOSpecDefault {
  override def spec =
    suite("ConfigSpec")(test("encoding") {
      val extension = Extension.YML
      val config    = JsonPlaceholderConfig.config
      for {
        encoded <- extension.encode(config)
        _ = pprint.pprintln(encoded)
        decoded <- extension.decode(encoded)
      } yield assertTrue(decoded == config)
    })
}
