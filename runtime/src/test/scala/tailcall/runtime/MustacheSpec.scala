package tailcall.runtime

import tailcall.runtime.model.Mustache
import tailcall.runtime.model.Mustache.Template
import tailcall.runtime.model.Mustache.Template.{lit, prm}
import zio.ZIO
import zio.schema.DynamicValue
import zio.test.Assertion._
import zio.test._

object MustacheSpec extends ZIOSpecDefault {

  def spec =
    suite("MustacheSpec")(
      test("syntax") {
        val input = List(
          //
          "{{a}}"     -> Mustache("a"),
          "{{a.b}}"   -> Mustache("a", "b"),
          "{{a.b.c}}" -> Mustache("a", "b", "c"),
        )

        checkAll(Gen.fromIterable(input)) { case (input, expected) =>
          val output = Mustache.syntax.parseString(input)
          assert(output)(isRight(equalTo(expected)))
        }
      },
      test("encoding") {
        val input = List(
          //
          Mustache("a")           -> "{{a}}",
          Mustache("a", "b")      -> "{{a.b}}",
          Mustache("a", "b", "c") -> "{{a.b.c}}",
        )
        checkAll(Gen.fromIterable(input)) { case (input, expected) =>
          val output = Mustache.syntax.printString(input)
          assert(output)(isRight(equalTo(expected)))
        }
      },
      test("evaluate") {
        val input = List(
          "{{a}}"     -> DynamicValue(Map("a" -> 1)),
          "{{a.b}}"   -> DynamicValue(Map("a" -> Map("b" -> 1))),
          "{{a.b.c}}" -> DynamicValue(Map("a" -> Map("b" -> Map("c" -> 1)))),
        )

        checkAll(Gen.fromIterable(input)) { case (mustache, input) =>
          val output = Mustache.evaluate(mustache, input)
          assert(output)(equalTo("1"))
        }
      },
      suite("template")(
        test("syntax") {
          val input = List(
            "ab"          -> Template(lit("ab")),
            "ab{{c.d}}"   -> Template(lit("ab"), prm("c", "d")),
            "ab{{c.d}}ef" -> Template(lit("ab"), prm("c", "d"), lit("ef")),
          )

          checkAll(Gen.fromIterable(input)) { case (string, template) =>
            val output  = Template.syntax.parseString(string)
            val encoded = Template.syntax.printString(template)
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
              parsed <- ZIO.fromEither(Template.syntax.parseString(template)).map(_.evaluate(input))
              actual <- ZIO.fromEither(Template.syntax.printString(parsed))
            } yield assertTrue(actual == expected)
          }
        },
      ),
    )
}
