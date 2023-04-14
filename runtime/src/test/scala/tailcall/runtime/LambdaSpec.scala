package tailcall.runtime

import tailcall.runtime.http.HttpClient
import tailcall.runtime.internal.DynamicValueUtil
import tailcall.runtime.model.{Context, Endpoint, TSchema}
import tailcall.runtime.remote.Remote.{logic, math}
import tailcall.runtime.remote.{Remote, ~>}
import tailcall.runtime.service.{DataLoader, EvaluationRuntime}
import zio.durationInt
import zio.http.Client
import zio.schema.DynamicValue
import zio.test.Assertion._
import zio.test.TestAspect.timeout
import zio.test._

object LambdaSpec extends ZIOSpecDefault {
  import tailcall.runtime.remote.Numeric._

  def spec =
    suite("Lambda")(
      suite("math")(
        test("add") {
          val program = math.add(Remote(1), Remote(2))
          assertZIO(program.evaluate)(equalTo(3))
        },
        test("subtract") {
          val program = math.sub(Remote(1), Remote(2))
          assertZIO(program.evaluate)(equalTo(-1))
        },
        test("multiply") {
          val program = math.mul(Remote(2), Remote(3))
          assertZIO(program.evaluate)(equalTo(6))
        },
        test("divide") {
          val program = math.div(Remote(6), Remote(3))
          assertZIO(program.evaluate)(equalTo(2))
        },
        test("modulo") {
          val program = math.mod(Remote(7), Remote(3))
          assertZIO(program.evaluate)(equalTo(1))
        },
        test("greater than") {
          val program = math.gt(Remote(2), Remote(1))
          assertZIO(program.evaluate)(isTrue)
        },
      ),
      suite("logical")(
        test("and") {
          val program = logic.and(Remote(true), Remote(true))
          assertZIO(program.evaluate)(isTrue)
        },
        test("or") {
          val program = logic.or(Remote(true), Remote(false))
          assertZIO(program.evaluate)(isTrue)
        },
        test("not") {
          val program = logic.not(Remote(true))
          assertZIO(program.evaluate)(isFalse)
        },
        test("equal") {
          val program = logic.eq(Remote(1), Remote(1))
          assertZIO(program.evaluate)(equalTo(true))
        },
        test("not equal") {
          val program = logic.eq(Remote(1), Remote(2))
          assertZIO(program.evaluate)(equalTo(false))
        },
      ),
      suite("diverge")(
        test("isTrue") {
          val program = logic.cond(Remote(true))(Remote("Yes"), Remote("No"))
          assertZIO(program.evaluate)(equalTo("Yes"))
        },
        test("isFalse") {
          val program = logic.cond(Remote(false))(Remote("Yes"), Remote("No"))
          assertZIO(program.evaluate)(equalTo("No"))
        },
      ),
      suite("fromFunction")(
        test("one level") {
          val program = Remote.fromLambdaFunction[Int, Int](i => math.add(i, Remote(1)))
          assertZIO(program.evaluateWith(1))(equalTo(2))
        },
        test("two level") {
          val program = Remote.fromLambdaFunction[Int, Int] { i =>
            val f1 = Remote.fromLambdaFunction[Int, Int](j => math.mul(i, j))
            math.add(i, Remote(1)) >>> f1
          }
          assertZIO(program.evaluateWith(10))(equalTo(110))
        },
        test("three level") {
          val program = Remote.fromLambdaFunction[Int, Int] { i =>
            val f1 = Remote.fromLambdaFunction[Int, Int] { j =>
              val f2 = Remote.fromLambdaFunction[Int, Int](k => math.mul(math.mul(i, j), k))
              math.add(j, Remote(1)) >>> f2
            }
            math.add(i, Remote(1)) >>> f1
          }
          assertZIO(program.evaluateWith(10))(equalTo(10 * 11 * 12))
        },
        test("nested siblings") {
          val program = Remote.fromLambdaFunction[Int, Int] { i =>
            val f1 = Remote.fromLambdaFunction[Int, Int](j => math.mul(i, j))
            val f2 = Remote.fromLambdaFunction[Int, Int](j => math.mul(i, j))
            math.add(math.add(i, Remote(1)) >>> f1, math.sub(i, Remote(1)) >>> f2)
          }
          assertZIO(program.evaluateWith(10))(equalTo(200))
        },
      ),
      suite("recursion")(
        test("sum") {
          val sum: Int ~> Int = Remote.recurse[Int, Int] { next =>
            logic.cond(logic.eq(Remote.identity[Int], Remote(0)))(
              isTrue = Remote(0),
              isFalse = math.add(Remote.identity[Int], math.dec(Remote.identity[Int]) >>> next),
            )
          }
          assertZIO(sum.evaluateWith(5))(equalTo(15))

        },
        test("factorial") {
          val factorial: Int ~> Int = Remote.recurse[Int, Int](next =>
            logic.cond(math.gte(Remote.identity[Int], Remote(1)))(
              math.mul(Remote.identity[Int], math.sub(Remote.identity[Int], Remote(1)) >>> next),
              Remote(1),
            )
          )
          assertZIO(factorial.evaluateWith(5))(equalTo(120))
        },
        test("fibonnaci") {
          val fib = Remote.recurse[Int, Int] { next =>
            logic.cond(math.gte(Remote.identity[Int], Remote(2)))(
              math.add(
                math.sub(Remote.identity[Int], Remote(1)) >>> next,
                math.sub(Remote.identity[Int], Remote(2)) >>> next,
              ),
              Remote.identity[Int],
            )
          }
          assertZIO(fib.evaluateWith(10))(equalTo(55))
        },
      ),
      suite("map")(
        test("get some") {
          val program = Remote.dict.get(Remote("key"), Remote.identity[Map[String, String]])
          assertZIO(program.evaluateWith(Map("key" -> "value")))(equalTo(Some("value")))
        },
        test("get none") {
          val program = Remote.dict.get(Remote("key"), Remote.identity[Map[String, String]])
          assertZIO(program.evaluateWith(Map("key0" -> "value")))(equalTo(None))
        },
        test("put") {
          val program = Remote.dict.put(Remote("key"), Remote("value"), Remote.identity[Map[String, String]])
          assertZIO(program.evaluateWith(Map("key0" -> "value")))(equalTo(Map("key" -> "value", "key0" -> "value")))
        },
        test("toPair") {
          val program = Remote(Map("a" -> 1, "b" -> 2)) >>> Remote.dict.toPair
          assertZIO(program.evaluate)(equalTo(Seq(("a", 1), ("b", 2))))
        },
      ),
      suite("DynamicValueOps")(
        suite("AsSeq")(
          test("some - int") {
            val p = Remote(DynamicValue(Seq(1, 2, 3))) >>> Remote.dynamic.toTyped[Seq[Int]]
            assertZIO(p.evaluate)(equalTo(Some(Seq(1, 2, 3))))
          },
          test("some - string") {
            val p = Remote(DynamicValue(Seq("1", "2", "3"))) >>> Remote.dynamic.toTyped[Seq[String]]
            assertZIO(p.evaluate)(equalTo(Some(Seq("1", "2", "3"))))
          },
          test("none - string") {
            val p = Remote(DynamicValue(Seq("1", "2", "3"))) >>> Remote.dynamic.toTyped[Seq[Int]]
            assertZIO(p.evaluate)(equalTo(None))
          },
          test("none - int") {
            val p = Remote(DynamicValue(Seq(1, 2, 3))) >>> Remote.dynamic.toTyped[Seq[String]]
            assertZIO(p.evaluate)(equalTo(None))
          },
        ),
        suite("asMap")(
          test("some - int") {
            val p = Remote(DynamicValue(Map("a" -> 1, "b" -> 2))) >>> Remote.dynamic.toTyped[Map[String, Int]]
            assertZIO(p.evaluate)(equalTo(Some(Map("a" -> 1, "b" -> 2))))
          },
          test("none -int") {
            val p = Remote(DynamicValue(Map("a" -> "1", "b" -> "2"))) >>> Remote.dynamic.toTyped[Map[String, Int]]
            assertZIO(p.evaluate)(equalTo(None))
          },
        ),
        suite("asInt")(
          test("some") {
            val p = Remote(DynamicValue(1)) >>> Remote.dynamic.toTyped[Int]
            assertZIO(p.evaluate)(equalTo(Some(1)))
          },
          test("none") {
            val p = Remote(DynamicValue("1")) >>> Remote.dynamic.toTyped[Int]
            assertZIO(p.evaluate)(equalTo(None))
          },
        ),
        suite("asBoolean")(
          test("some") {
            val p = Remote(DynamicValue(true)) >>> Remote.dynamic.toTyped[Boolean]
            assertZIO(p.evaluate)(equalTo(Some(true)))
          },
          test("none") {
            val p = Remote(DynamicValue(1)) >>> Remote.dynamic.toTyped[Boolean]
            assertZIO(p.evaluate)(equalTo(None))
          },
        ),
        suite("asString")(
          test("some") {
            val p = Remote(DynamicValue("1")) >>> Remote.dynamic.toTyped[String]
            assertZIO(p.evaluate)(equalTo(Some("1")))
          },
          test("none") {
            val p = Remote(DynamicValue(1)) >>> Remote.dynamic.toTyped[String]
            assertZIO(p.evaluate)(equalTo(None))
          },
        ),
        suite("toDynamic")(
          test("int") {
            val p = Remote(1) >>> Remote.dynamic.toDynamic
            assertZIO(p.evaluate)(equalTo(DynamicValue(1)))
          },
          test("string") {
            val p = Remote("1") >>> Remote.dynamic.toDynamic
            assertZIO(p.evaluate)(equalTo(DynamicValue("1")))
          },
          test("boolean") {
            val p = Remote(true) >>> Remote.dynamic.toDynamic
            assertZIO(p.evaluate)(equalTo(DynamicValue(true)))
          },
          test("map") {
            val p = Remote(Map("a" -> 1, "b" -> 2)) >>> Remote.dynamic.toDynamic
            assertZIO(p.evaluate)(equalTo(DynamicValue(Map("a" -> 1, "b" -> 2))))
          },
          test("seq") {
            val p = Remote(Seq(1, 2, 3)) >>> Remote.dynamic.toDynamic
            assertZIO(p.evaluate)(equalTo(DynamicValue(Seq(1, 2, 3))))
          },
          test("option") {
            val p = Remote(Option(100)) >>> Remote.dynamic.toDynamic
            assertZIO(p.evaluate)(equalTo(DynamicValue(Option(100))))
          },
        ),
        suite("path")(
          test("one level") {
            val context  = Context(DynamicValue("Tailcall"), Map("foo" -> DynamicValue(1)), None)
            val p        = Remote(DynamicValue(context)) >>> Remote.dynamic.path("value")
            val expected = DynamicValue("Tailcall")
            assertZIO(p.evaluate)(equalTo(Some(expected)))
          },
          test("with option") {
            val parent   = Context(value = DynamicValue("Parent"))
            val context  = Context(value = DynamicValue("Child"), parent = Option(parent))
            val p        = Remote(DynamicValue(context)) >>> Remote.dynamic.path("parent", "value")
            val expected = DynamicValue("Parent")
            assertZIO(p.evaluate)(equalTo(Some(expected)))
          },
          test("with map") {
            val input    = Map("a" -> 100)
            val p        = Remote(DynamicValue(input)) >>> Remote.dynamic.path("a")
            val expected = DynamicValue(100)
            assertZIO(p.evaluate)(equalTo(Some(expected)))
          },
        ),
      ),
      suite("option")(
        test("isSome") {
          val program = Remote(Option(1)) >>> Remote.option.isSome
          assertZIO(program.evaluate)(isTrue)
        },
        test("isNone") {
          val program = Remote(Option.empty[Int]) >>> Remote.option.isNone
          assertZIO(program.evaluate)(isTrue)
        },
        test("fold some") {
          val program = Remote.option.fold(
            Remote(Option(0)),
            ifNone = Remote.math.inc(Remote.identity[Int]),
            ifSome = Remote.math.inc(Remote.identity[Int]),
          )
          assertZIO(program.evaluateWith(100))(equalTo(1))
        },
        test("fold none") {
          val program = Remote.option.fold(
            Remote(Option.empty[Int]),
            ifNone = Remote.math.inc(Remote.identity[Int]),
            ifSome = Remote.math.inc(Remote.identity[Int]),
          )
          assertZIO(program.evaluateWith(100))(equalTo(101))
        },
        test("apply some") {
          val program = Remote.option(Option(Remote(0)))
          assertZIO(program.evaluate)(equalTo(Some(0)))
        },
        test("apply none") {
          val program = Remote.option(Option.empty[Int ~> Int])
          assertZIO(program.evaluateWith(0))(equalTo(None))
        },
      ),
      suite("unsafe")(
        test("endpoint /users/1") {
          val endpoint = Endpoint.make("jsonplaceholder.typicode.com").withPath("/users/{{id}}")
            .withOutput(Option(TSchema.obj("id" -> TSchema.num, "name" -> TSchema.string)))
          val program  = Remote.unsafe.fromEndpoint(endpoint)
          val expected = DynamicValueUtil.record("id" -> DynamicValue(1L), "name" -> DynamicValue("Leanne Graham"))
          assertZIO(program.evaluateWith(DynamicValue(Map("id" -> 1))))(equalTo(expected))
        },
        test("error") {
          val endpoint = Endpoint.make("jsonplaceholder.typicode.com").withPath("/users/{{id}}")
            .withOutput(Option(TSchema.obj("id" -> TSchema.num, "name" -> TSchema.string)))
          val program  = Remote.unsafe.fromEndpoint(endpoint).evaluateWith(DynamicValue(Map("id" -> 100))).flip
            .map(_.getMessage)

          assertZIO(program)(equalTo("HTTP Error: 404"))
        },
      ) @@ timeout(5 seconds),
    ).provide(EvaluationRuntime.default, HttpClient.live, Client.default, DataLoader.http)
}
