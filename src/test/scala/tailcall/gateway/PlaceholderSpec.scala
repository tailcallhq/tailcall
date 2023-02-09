package tailcall.gateway

import tailcall.gateway.ast.Placeholder
import zio.Chunk
import zio.test.Assertion._
import zio.test._

object PlaceholderSpec extends ZIOSpecDefault {
  def spec = suite("PlaceholderSpec")(test("syntax") {
    val input = List(
      "a"     -> Placeholder(Chunk("a")),
      "a.b"   -> Placeholder(Chunk("a", "b")),
      "a.b.c" -> Placeholder(Chunk("a", "b", "c"))
    )

    checkAll(Gen.fromIterable(input)) { case (input, expected) =>
      val output = Placeholder.syntax.parseString(input)
      assert(output)(isRight(equalTo(expected)))
    }

  })
}
