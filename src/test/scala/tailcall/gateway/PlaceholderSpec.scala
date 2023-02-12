package tailcall.gateway

import tailcall.gateway.ast.Placeholder
import zio.test.Assertion._
import zio.test._

object PlaceholderSpec extends ZIOSpecDefault {
  def spec =
    suite("PlaceholderSpec")(test("syntax") {
      val input = List(
        "${a}"     -> Placeholder("a"),
        "${a.b}"   -> Placeholder("a", "b"),
        "${a.b.c}" -> Placeholder("a", "b", "c"),
        "a"        -> Placeholder.literal("a")
      )

      checkAll(Gen.fromIterable(input)) { case (input, expected) =>
        val output = Placeholder.syntax.parseString(input)
        assert(output)(isRight(equalTo(expected)))
      }
    })
}
