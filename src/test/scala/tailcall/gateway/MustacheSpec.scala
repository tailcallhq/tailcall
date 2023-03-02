package tailcall.gateway

import tailcall.gateway.ast.Mustache
import zio.schema.DynamicValue
import zio.test.Assertion._
import zio.test._

object MustacheSpec extends ZIOSpecDefault:
  def spec =
    suite("MustacheSpec")(
      test("syntax") {
        val input =
          List("{{a}}" -> Mustache("a"), "{{a.b}}" -> Mustache("a", "b"), "{{a.b.c}}" -> Mustache("a", "b", "c"))

        checkAll(Gen.fromIterable(input)) { case (input, expected) =>
          val output = Mustache.syntax.parseString(input)
          assert(output)(isRight(equalTo(expected)))
        }
      },
      test("evaluate") {
        val input = List(
          "{{a}}"     -> DynamicValue(Map("a" -> 1)),
          "{{a.b}}"   -> DynamicValue(Map("a" -> Map("b" -> 1))),
          "{{a.b.c}}" -> DynamicValue(Map("a" -> Map("b" -> Map("c" -> 1))))
        )

        checkAll(Gen.fromIterable(input)) { case (mustache, input) =>
          val output = Mustache.evaluate(mustache, input)
          assert(output)(equalTo("1"))
        }
      }
    )
