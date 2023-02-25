package tailcall.gateway

import zio.test.{ZIOSpecDefault, assertCompletes}

object DocumentStepGenerator extends ZIOSpecDefault {
  def spec = suite("DocumentStepGenerator")(test("one level")(assertCompletes))
}
