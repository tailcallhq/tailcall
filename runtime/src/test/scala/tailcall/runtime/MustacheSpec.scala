package tailcall.runtime

import tailcall.runtime.model.Mustache
import tailcall.runtime.model.Mustache.{prm, txt}
import zio.ZIO
import zio.schema.DynamicValue
import zio.test.Assertion._
import zio.test._

object MustacheSpec extends ZIOSpecDefault {
  def spec =
    suite("Mustache")(
      test("syntax") {
        val input = List(
          "ab"          -> Mustache(txt("ab")),
          "ab{{c.d}}"   -> Mustache(txt("ab"), prm("c", "d")),
          "ab{{c.d}}ef" -> Mustache(txt("ab"), prm("c", "d"), txt("ef")),
        )

        checkAll(Gen.fromIterable(input)) { case (string, template) =>
          val output  = Mustache.syntax.parseString(string)
          val encoded = Mustache.syntax.printString(template)
          assert(output)(isRight(equalTo(template))) && assert(encoded)(isRight(equalTo(string)))
        }
      },
      test("evaluate") {
        val input = List(
          "x{{a}}"             -> DynamicValue(Map("a" -> 1))                         -> "x1",
          "{{a.b}}y"           -> DynamicValue(Map("a" -> Map("b" -> 1)))             -> "1y",
          "x{{a.b.c}}y"        -> DynamicValue(Map("a" -> Map("b" -> Map("c" -> 1)))) -> "x1y",
          "x{{a}}y{{b}}z{{c}}" -> DynamicValue(Map("a" -> 1, "b" -> 2))               -> s"x1y2z{{c}}",
        )

        checkAll(Gen.fromIterable(input)) { case template -> input -> expected =>
          for {
            parsed <- ZIO.fromEither(Mustache.syntax.parseString(template)).map(_.evaluate(input))
            actual <- ZIO.fromEither(Mustache.syntax.printString(parsed))
          } yield assertTrue(actual == expected)
        }
      },
    )
}
