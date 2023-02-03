package tailcall.gateway

import zio.test._

object OrchSpec extends ZIOSpecDefault {
  def spec = suite("OrchSpec")(test("test")(assertCompletes))
}
