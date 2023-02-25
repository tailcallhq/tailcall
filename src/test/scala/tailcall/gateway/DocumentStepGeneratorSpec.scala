package tailcall.gateway

import zio.test.{ZIOSpecDefault, assertCompletes}

object DocumentStepGeneratorSpec extends ZIOSpecDefault {
  def spec = suite("DocumentStepGenerator")(test("one level")(assertCompletes))
}
