package tailcall.runtime

import tailcall.runtime.model.Config.Field
import tailcall.runtime.model.UnsafeSteps.Operation
import tailcall.runtime.model.{Config, Path, TSchema}
import zio.test.{ZIOSpecDefault, assertTrue}

object ConfigSpec extends ZIOSpecDefault {
  def spec =
    suite("ConfigSpec")(suite("compression")(test("http with schema") {
      val step     = Operation.Http(path = Path.unsafe.fromString("/foo"), output = Option(TSchema.str))
      val config   = Config.default.withTypes("Query" -> Config.Type("foo" -> Field.ofType("String").withSteps(step)))
      val actual   = config.compress
      val expected = config
      assertTrue(actual == expected)
    }))
}
