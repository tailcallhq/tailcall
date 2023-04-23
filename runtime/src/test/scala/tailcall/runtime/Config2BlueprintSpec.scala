package tailcall.runtime

import tailcall.runtime.model.{Config, Server}
import zio.test._

import java.net.URL

object Config2BlueprintSpec extends ZIOSpecDefault {
  def spec =
    suite("Config2BlueprintSpec")(test("timeout") {
      val timeout = Config(server = Server(baseURL = Some(new URL("http://localhost:8080")), timeout = Some(1000)))
        .toBlueprint.toOption.flatMap(_.server.globalResponseTimeout)

      assertTrue(timeout == Option(1000))
    })
}
