package tailcall.runtime

import tailcall.runtime.model.Config.{Field, Type}
import tailcall.runtime.model.Operation
import tailcall.runtime.model.Operation.Http
import tailcall.runtime.model.{Config, Path, TSchema}
import tailcall.test.TailcallSpec
import zio.test.assertTrue

object ConfigSpec extends TailcallSpec {
  def spec =
    suite("ConfigSpec")(suite("compression")(
      test("http with schema") {
        val step     = Operation.Http(path = Path.unsafe.fromString("/foo"), output = Option(TSchema.str))
        val config   = Config.default.withTypes("Query" -> Config.Type("foo" -> Field.ofType("String").withSteps(step)))
        val actual   = config.compress
        val expected = config
        assertTrue(actual == expected)
      },
      suite("n + 1")(
        test("with resolvers") {
          val config   = Config.default(
            "Query" -> Type("f1" -> Field.ofType("F1").asList.resolveWith(0)),
            "F1"    -> Type("f2" -> Field.ofType("F2").asList.resolveWith(0)),
            "F2"    -> Type("f3" -> Field.str),
          )
          val actual   = config.nPlusOne
          val expected = List(List("Query" -> "f1", "F1" -> "f2"))
          assertTrue(actual == expected)
        },
        test("with batched resolvers") {
          val http     = Http.fromPath("/f2").withGroupBy("a").withBatchKey("b")
          val config   = Config.default(
            "Query" -> Type("f1" -> Field.ofType("F1").asList.resolveWith(0)),
            "F1"    -> Type("f2" -> Field.ofType("F2").asList.withHttp(http)),
            "F2"    -> Type("f3" -> Field.str),
          )
          val actual   = config.nPlusOne
          val expected = List()
          assertTrue(actual == expected)
        },
        test("with nested resolvers") {
          val config   = Config.default(
            "Query" -> Type("f1" -> Field.ofType("F1").asList.resolveWith(0)),
            "F1"    -> Type("f2" -> Field.ofType("F2").asList),
            "F2"    -> Type("f3" -> Field.ofType("F3").asList),
            "F3"    -> Type("f4" -> Field.str.resolveWith(0)),
          )
          val actual   = config.nPlusOne
          val expected = List(List("Query" -> "f1", "F1" -> "f2", "F2" -> "f3", "F3" -> "f4"))
          assertTrue(actual == expected)
        },
        test("with nested resolvers non list resolvers") {
          val config   = Config.default(
            "Query" -> Type("f1" -> Field.ofType("F1").resolveWith(0)),
            "F1"    -> Type("f2" -> Field.ofType("F2").asList),
            "F2"    -> Type("f3" -> Field.ofType("F3").asList),
            "F3"    -> Type("f4" -> Field.str.resolveWith(0)),
          )
          val actual   = config.nPlusOne
          val expected = List(List("Query" -> "f1", "F1" -> "f2", "F2" -> "f3", "F3" -> "f4"))
          assertTrue(actual == expected)
        },
        test("without resolvers") {
          val config   = Config.default(
            "Query" -> Type("f1" -> Field.ofType("F1").asList.resolveWith(0)),
            "F1"    -> Type("f2" -> Field.ofType("F2").asList),
            "F2"    -> Type("f3" -> Field.str),
          )
          val actual   = config.nPlusOne
          val expected = List()
          assertTrue(actual == expected)
        },
        test("cycles") {
          val config   = Config.default(
            "Query" -> Type("f1" -> Field.ofType("F1").asList.resolveWith(0)),
            "F1"    -> Type("f1" -> Field.ofType("F1"), "f2" -> Field.ofType("F2").asList),
            "F2"    -> Type("f3" -> Field.str),
          )
          val actual   = config.nPlusOne
          val expected = List()
          assertTrue(actual == expected)
        },
        test("cycles with resolvers") {
          val config   = Config.default(
            "Query" -> Type("f1" -> Field.ofType("F1").asList.resolveWith(0)),
            "F1"    -> Type("f1" -> Field.ofType("F1").asList, "f2" -> Field.str.resolveWith(0)),
          )
          val actual   = config.nPlusOne
          val expected = List(List("Query" -> "f1", "F1" -> "f1", "F1" -> "f2"), List("Query" -> "f1", "F1" -> "f2"))
          assertTrue(actual == expected)
        },
      ),
    ))
}
