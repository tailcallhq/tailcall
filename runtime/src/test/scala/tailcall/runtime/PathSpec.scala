package tailcall.runtime

import tailcall.runtime.model.Path
import tailcall.runtime.model.Path.Segment.{Literal, Param}
import tailcall.test.TailcallSpec
import zio.ZIO
import zio.schema.DynamicValue
import zio.test.Assertion.equalTo
import zio.test._

object PathSpec extends TailcallSpec {
  val syntax = Path.syntax.route

  override def spec =
    suite("path")(
      test("segments") {
        val input: Seq[(String, List[Path.Segment])] = Seq(
          "/a"                 -> (Literal("a") :: Nil),
          "/a/b"               -> (Literal("a") :: Literal("b") :: Nil),
          "/a/b/c"             -> (Literal("a") :: Literal("b") :: Literal("c") :: Nil),
          "/a-b"               -> (Literal("a-b") :: Nil),
          "/a/b/{{c}}"         -> (Literal("a") :: Literal("b") :: Param("c") :: Nil),
          "/a/{{b}}/{{c}}"     -> (Literal("a") :: Param("b") :: Param("c") :: Nil),
          "/{{a}}/{{b}}/{{c}}" -> (Param("a") :: Param("b") :: Param("c") :: Nil),
          "/{{a}}/{{b}}"       -> (Param("a") :: Param("b") :: Nil),
          "/{{a}}"             -> (Param("a") :: Nil),
          "/a_b"               -> (Literal("a_b") :: Nil),
        )
        checkAll(Gen.fromIterable(input)) { case (input, expected) =>
          val parsed = ZIO.fromEither(syntax.parseString(input)).map(_.segments)
          assertZIO(parsed)(equalTo(expected))
        }
      },
      test("unsafeEvaluate") {
        val inputs = List(
          "/{{a}}/{{b}}/{{c}}" -> DynamicValue(Map("a" -> "a", "b" -> "b", "c" -> "c")),
          "/{{a.b.c}}/b/c"     -> DynamicValue(Map("a" -> Map("b" -> Map("c" -> "a")))),
        )

        checkAll(Gen.fromIterable(inputs)) { case (path, input) =>
          val string = ZIO.fromEither(syntax.parseString(path)).map(_.unsafeEval(input))
          assertZIO(string)(equalTo("/a/b/c"))
        }
      },
      test("evaluate") {
        val inputs = List(
          "/{{a}}/{{b}}/{{c}}" -> DynamicValue(Map("a" -> "a", "b" -> "b"))             -> "/a/b/{{c}}",
          "/{{a}}/{{b}}/{{c}}" -> DynamicValue(Map("a" -> "a", "b" -> "b", "c" -> "c")) -> "/a/b/c",
          "/{{a}}/{{b}}/{{c}}" -> DynamicValue(Map.empty[String, String])               -> "/{{a}}/{{b}}/{{c}}",
        )

        checkAll(Gen.fromIterable(inputs)) { case path -> input -> output =>
          for {
            actual   <- ZIO.fromEither(syntax.parseString(path)).map(_.reduce(input))
            expected <- ZIO.fromEither(syntax.parseString(output))
          } yield assertTrue(actual == expected)
        }
      },
      test("unreserved characters in path segment") {
        val inputs = List("/v1.1", "/v2~2", "/some-name", "/some_name")

        checkAll(Gen.fromIterable(inputs)) { case str =>
          assertTrue(Path.unsafe.fromString(str).encode.getOrElse("") == str)
        }
      },
    )
}
