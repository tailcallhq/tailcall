package tailcall.runtime

import tailcall.runtime.http.HttpClient
import tailcall.runtime.internal.DynamicValueUtil
import tailcall.runtime.lambda.Lambda.{logic, math}
import tailcall.runtime.lambda._
import tailcall.runtime.model.{Context, Endpoint, TSchema}
import tailcall.runtime.service.{DataLoader, EvaluationRuntime}
import zio.durationInt
import zio.http.Client
import zio.schema.DynamicValue
import zio.test.Assertion._
import zio.test.TestAspect.timeout
import zio.test._

object LambdaSpec extends ZIOSpecDefault {
  import tailcall.runtime.lambda.Numeric._

  def spec =
    suite("Lambda")(
      suite("math")(
        test("add") {
          val program = Lambda(1) + Lambda(2)
          assertZIO(program.evaluate)(equalTo(3))
        },
        test("subtract") {
          val program = Lambda(1) - Lambda(2)
          assertZIO(program.evaluate)(equalTo(-1))
        },
        test("multiply") {
          val program = Lambda(2) * Lambda(3)
          assertZIO(program.evaluate)(equalTo(6))
        },
        test("divide") {
          val program = Lambda(6) / Lambda(3)
          assertZIO(program.evaluate)(equalTo(2))
        },
        test("modulo") {
          val program = Lambda(7) % Lambda(3)
          assertZIO(program.evaluate)(equalTo(1))
        },
        test("greater than") {
          val program = Lambda(2) > Lambda(1)
          assertZIO(program.evaluate)(isTrue)
        },
      ),
      suite("logical")(
        test("and") {
          val program = Lambda(true) && Lambda(true)
          assertZIO(program.evaluate)(isTrue)
        },
        test("or") {
          val program = Lambda(true) || Lambda(false)
          assertZIO(program.evaluate)(isTrue)
        },
        test("not") {
          val program = !Lambda(true)
          assertZIO(program.evaluate)(isFalse)
        },
      ),
//      suite("equals")(
//        test("equal") {
//          val program = Lambda(1) =:= Lambda(1)
//          assertZIO(program.evaluate)(isTrue)
//        },
//        test("not equal") {
//          val program = Lambda(1) =:= Lambda(2)
//          assertZIO(program.evaluate)(isFalse)
//        }
//      ),
      suite("diverge")(
        test("isTrue") {
          val program = Lambda(true).diverge(Lambda("Yes"), Lambda("No"))
          assertZIO(program.evaluate)(equalTo("Yes"))
        },
        test("isFalse") {
          val program = Lambda(false).diverge(Lambda("Yes"), Lambda("No"))
          assertZIO(program.evaluate)(equalTo("No"))
        },
      ),
      suite("fromFunction")(
        test("one level") {
          val program = Lambda.fromFunction[Int, Int](i => i + Lambda(1))
          assertZIO(program.evaluateWith(1))(equalTo(2))
        },
        test("two level") {
          val program = Lambda.fromFunction[Int, Int] { i =>
            val f1 = Lambda.fromFunction[Int, Int](j => i * j)
            f1(i + Lambda(1))
          }(Lambda(10))
          assertZIO(program.evaluate)(equalTo(110))
        },
        test("three level") {
          val program = Lambda.fromFunction[Int, Int] { i =>
            val f1 = Lambda.fromFunction[Int, Int] { j =>
              val f2 = Lambda.fromFunction[Int, Int](k => i * j * k)
              f2(j + Lambda(1))
            }
            f1(i + Lambda(1))
          }(Lambda(10))
          assertZIO(program.evaluate)(equalTo(10 * 11 * 12))
        },
        test("three level") {
          val program = Lambda.fromFunction[Int, Int] { i =>
            val f1 = Lambda.fromFunction[Int, Int](j => j * i)
            val f2 = Lambda.fromFunction[Int, Int](j => i * j)
            f1(i + Lambda(1)) + f2(i - Lambda(1))
          }(Lambda(10))
          assertZIO(program.evaluate)(equalTo(200))
        },
      ),
      suite("option")(
        test("some") {
          val program = Lambda.option(Some(Lambda(1)))
          assertZIO(program.evaluate)(equalTo(Some(1)))
        },
        test("none") {
          val program = Lambda.option(None)
          assertZIO(program.evaluate)(equalTo(None))
        },
        test("isSome") {
          val program = Lambda.option(Some(Lambda(1))).isSome
          assertZIO(program.evaluate)(isTrue)
        },
        test("isNone") {
          val program = Lambda.option(None).isNone
          assertZIO(program.evaluate)(isTrue)
        },
        test("fold some") {
          val program = Lambda.option(Some(Lambda(1))).fold(Lambda(0), _ * Lambda(2))
          assertZIO(program.evaluate)(equalTo(2))
        },
        test("fold none") {
          val program = Lambda.option(Option.empty[Any ~> Int]).fold(Lambda(0), _ * Lambda(2))
          assertZIO(program.evaluate)(equalTo(0))
        },
      ),
      suite("dynamicValue")(
        test("int") {
          val program = Lambda(1).toDynamic
          assertZIO(program.evaluate)(equalTo(DynamicValue(1)))
        },
        test("some") {
          val program = Lambda(Option(1)).toDynamic
          assertZIO(program.evaluate)(equalTo(DynamicValue(Option(1))))
        },
        test("none") {
          val program = Lambda(Option.empty[Int]).toDynamic
          assertZIO(program.evaluate)(equalTo(DynamicValue(Option.empty[Int])))
        },
      ),
      suite("math")(
        test("add") {
          val program = math.add(Lambda(1), Lambda(2))
          assertZIO(program.evaluate)(equalTo(3))
        },
        test("subtract") {
          val program = math.sub(Lambda(1), Lambda(2))
          assertZIO(program.evaluate)(equalTo(-1))
        },
        test("multiply") {
          val program = math.mul(Lambda(2), Lambda(3))
          assertZIO(program.evaluate)(equalTo(6))
        },
        test("divide") {
          val program = math.div(Lambda(6), Lambda(3))
          assertZIO(program.evaluate)(equalTo(2))
        },
        test("modulo") {
          val program = math.mod(Lambda(7), Lambda(3))
          assertZIO(program.evaluate)(equalTo(1))
        },
        test("greater than") {
          val program = math.gt(Lambda(2), Lambda(1))
          assertZIO(program.evaluate)(isTrue)
        },
      ),
      suite("logical")(
        test("and") {
          val program = logic.and(Lambda(true), Lambda(true))
          assertZIO(program.evaluate)(isTrue)
        },
        test("or") {
          val program = logic.or(Lambda(true), Lambda(false))
          assertZIO(program.evaluate)(isTrue)
        },
        test("not") {
          val program = logic.not(Lambda(true))
          assertZIO(program.evaluate)(isFalse)
        },
        test("equal") {
          val program = logic.eq(Lambda(1), Lambda(1))
          assertZIO(program.evaluate)(equalTo(true))
        },
        test("not equal") {
          val program = logic.eq(Lambda(1), Lambda(2))
          assertZIO(program.evaluate)(equalTo(false))
        },
      ),
      suite("diverge")(
        test("isTrue") {
          val program = logic.cond(Lambda(true))(Lambda("Yes"), Lambda("No"))
          assertZIO(program.evaluate)(equalTo("Yes"))
        },
        test("isFalse") {
          val program = logic.cond(Lambda(false))(Lambda("Yes"), Lambda("No"))
          assertZIO(program.evaluate)(equalTo("No"))
        },
      ),
      suite("fromFunction")(
        test("one level") {
          val program = Lambda.fromFunction[Int, Int](i => math.add(i, Lambda(1)))
          assertZIO(program.evaluateWith(1))(equalTo(2))
        },
        test("two level") {
          val program = Lambda.fromFunction[Int, Int] { i =>
            val f1 = Lambda.fromFunction[Int, Int](j => math.mul(i, j))
            math.add(i, Lambda(1)) >>> f1
          }
          assertZIO(program.evaluateWith(10))(equalTo(110))
        },
        test("three level") {
          val program = Lambda.fromFunction[Int, Int] { i =>
            val f1 = Lambda.fromFunction[Int, Int] { j =>
              val f2 = Lambda.fromFunction[Int, Int](k => math.mul(math.mul(i, j), k))
              math.add(j, Lambda(1)) >>> f2
            }
            math.add(i, Lambda(1)) >>> f1
          }
          assertZIO(program.evaluateWith(10))(equalTo(10 * 11 * 12))
        },
        test("nested siblings") {
          val program = Lambda.fromFunction[Int, Int] { i =>
            val f1 = Lambda.fromFunction[Int, Int](j => math.mul(i, j))
            val f2 = Lambda.fromFunction[Int, Int](j => math.mul(i, j))
            math.add(math.add(i, Lambda(1)) >>> f1, math.sub(i, Lambda(1)) >>> f2)
          }
          assertZIO(program.evaluateWith(10))(equalTo(200))
        },
      ),
      suite("recursion")(
        test("sum") {
          val sum: Int ~> Int = Lambda.recurse[Int, Int] { next =>
            logic.cond(logic.eq(Lambda.identity[Int], Lambda(0)))(
              isTrue = Lambda(0),
              isFalse = math.add(Lambda.identity[Int], math.dec(Lambda.identity[Int]) >>> next),
            )
          }
          assertZIO(sum.evaluateWith(5))(equalTo(15))

        },
        test("factorial") {
          val factorial: Int ~> Int = Lambda.recurse[Int, Int](next =>
            logic.cond(math.gte(Lambda.identity[Int], Lambda(1)))(
              math.mul(Lambda.identity[Int], math.sub(Lambda.identity[Int], Lambda(1)) >>> next),
              Lambda(1),
            )
          )
          assertZIO(factorial.evaluateWith(5))(equalTo(120))
        },
        test("fibonnaci") {
          val fib = Lambda.recurse[Int, Int] { next =>
            logic.cond(math.gte(Lambda.identity[Int], Lambda(2)))(
              math.add(
                math.sub(Lambda.identity[Int], Lambda(1)) >>> next,
                math.sub(Lambda.identity[Int], Lambda(2)) >>> next,
              ),
              Lambda.identity[Int],
            )
          }
          assertZIO(fib.evaluateWith(10))(equalTo(55))
        },
      ),
      suite("map")(
        test("get some") {
          val program = Lambda.dict.get(Lambda("key"), Lambda.identity[Map[String, String]])
          assertZIO(program.evaluateWith(Map("key" -> "value")))(equalTo(Some("value")))
        },
        test("get none") {
          val program = Lambda.dict.get(Lambda("key"), Lambda.identity[Map[String, String]])
          assertZIO(program.evaluateWith(Map("key0" -> "value")))(equalTo(None))
        },
        test("put") {
          val program = Lambda.dict.put(Lambda("key"), Lambda("value"), Lambda.identity[Map[String, String]])
          assertZIO(program.evaluateWith(Map("key0" -> "value")))(equalTo(Map("key" -> "value", "key0" -> "value")))
        },
        test("toPair") {
          val program = Lambda(Map("a" -> 1, "b" -> 2)) >>> Lambda.dict.toPair
          assertZIO(program.evaluate)(equalTo(Seq(("a", 1), ("b", 2))))
        },
      ),
      suite("DynamicValueOps")(
        suite("AsSeq")(
          test("some - int") {
            val p = Lambda(DynamicValue(Seq(1, 2, 3))) >>> Lambda.dynamic.toTyped[Seq[Int]]
            assertZIO(p.evaluate)(equalTo(Some(Seq(1, 2, 3))))
          },
          test("some - string") {
            val p = Lambda(DynamicValue(Seq("1", "2", "3"))) >>> Lambda.dynamic.toTyped[Seq[String]]
            assertZIO(p.evaluate)(equalTo(Some(Seq("1", "2", "3"))))
          },
          test("none - string") {
            val p = Lambda(DynamicValue(Seq("1", "2", "3"))) >>> Lambda.dynamic.toTyped[Seq[Int]]
            assertZIO(p.evaluate)(equalTo(None))
          },
          test("none - int") {
            val p = Lambda(DynamicValue(Seq(1, 2, 3))) >>> Lambda.dynamic.toTyped[Seq[String]]
            assertZIO(p.evaluate)(equalTo(None))
          },
        ),
        suite("asMap")(
          test("some - int") {
            val p = Lambda(DynamicValue(Map("a" -> 1, "b" -> 2))) >>> Lambda.dynamic.toTyped[Map[String, Int]]
            assertZIO(p.evaluate)(equalTo(Some(Map("a" -> 1, "b" -> 2))))
          },
          test("none -int") {
            val p = Lambda(DynamicValue(Map("a" -> "1", "b" -> "2"))) >>> Lambda.dynamic.toTyped[Map[String, Int]]
            assertZIO(p.evaluate)(equalTo(None))
          },
        ),
        suite("asInt")(
          test("some") {
            val p = Lambda(DynamicValue(1)) >>> Lambda.dynamic.toTyped[Int]
            assertZIO(p.evaluate)(equalTo(Some(1)))
          },
          test("none") {
            val p = Lambda(DynamicValue("1")) >>> Lambda.dynamic.toTyped[Int]
            assertZIO(p.evaluate)(equalTo(None))
          },
        ),
        suite("asBoolean")(
          test("some") {
            val p = Lambda(DynamicValue(true)) >>> Lambda.dynamic.toTyped[Boolean]
            assertZIO(p.evaluate)(equalTo(Some(true)))
          },
          test("none") {
            val p = Lambda(DynamicValue(1)) >>> Lambda.dynamic.toTyped[Boolean]
            assertZIO(p.evaluate)(equalTo(None))
          },
        ),
        suite("asString")(
          test("some") {
            val p = Lambda(DynamicValue("1")) >>> Lambda.dynamic.toTyped[String]
            assertZIO(p.evaluate)(equalTo(Some("1")))
          },
          test("none") {
            val p = Lambda(DynamicValue(1)) >>> Lambda.dynamic.toTyped[String]
            assertZIO(p.evaluate)(equalTo(None))
          },
        ),
        suite("toDynamic")(
          test("int") {
            val p = Lambda(1) >>> Lambda.dynamic.toDynamic
            assertZIO(p.evaluate)(equalTo(DynamicValue(1)))
          },
          test("string") {
            val p = Lambda("1") >>> Lambda.dynamic.toDynamic
            assertZIO(p.evaluate)(equalTo(DynamicValue("1")))
          },
          test("boolean") {
            val p = Lambda(true) >>> Lambda.dynamic.toDynamic
            assertZIO(p.evaluate)(equalTo(DynamicValue(true)))
          },
          test("map") {
            val p = Lambda(Map("a" -> 1, "b" -> 2)) >>> Lambda.dynamic.toDynamic
            assertZIO(p.evaluate)(equalTo(DynamicValue(Map("a" -> 1, "b" -> 2))))
          },
          test("seq") {
            val p = Lambda(Seq(1, 2, 3)) >>> Lambda.dynamic.toDynamic
            assertZIO(p.evaluate)(equalTo(DynamicValue(Seq(1, 2, 3))))
          },
          test("option") {
            val p = Lambda(Option(100)) >>> Lambda.dynamic.toDynamic
            assertZIO(p.evaluate)(equalTo(DynamicValue(Option(100))))
          },
        ),
        suite("path")(
          test("one level") {
            val context  = Context(DynamicValue("Tailcall"), Map("foo" -> DynamicValue(1)), None)
            val p        = Lambda(DynamicValue(context)) >>> Lambda.dynamic.path("value")
            val expected = DynamicValue("Tailcall")
            assertZIO(p.evaluate)(equalTo(Some(expected)))
          },
          test("with option") {
            val parent   = Context(value = DynamicValue("Parent"))
            val context  = Context(value = DynamicValue("Child"), parent = Option(parent))
            val p        = Lambda(DynamicValue(context)) >>> Lambda.dynamic.path("parent", "value")
            val expected = DynamicValue("Parent")
            assertZIO(p.evaluate)(equalTo(Some(expected)))
          },
          test("with map") {
            val input    = Map("a" -> 100)
            val p        = Lambda(DynamicValue(input)) >>> Lambda.dynamic.path("a")
            val expected = DynamicValue(100)
            assertZIO(p.evaluate)(equalTo(Some(expected)))
          },
        ),
      ),
      suite("option")(
        test("isSome") {
          val program = Lambda(Option(1)) >>> Lambda.option.isSome
          assertZIO(program.evaluate)(isTrue)
        },
        test("isNone") {
          val program = Lambda(Option.empty[Int]) >>> Lambda.option.isNone
          assertZIO(program.evaluate)(isTrue)
        },
        test("fold some") {
          val program = Lambda.option.fold(
            Lambda(Option(0)),
            ifNone = Lambda.math.inc(Lambda.identity[Int]),
            ifSome = Lambda.math.inc(Lambda.identity[Int]),
          )
          assertZIO(program.evaluateWith(100))(equalTo(1))
        },
        test("fold none") {
          val program = Lambda.option.fold(
            Lambda(Option.empty[Int]),
            ifNone = Lambda.math.inc(Lambda.identity[Int]),
            ifSome = Lambda.math.inc(Lambda.identity[Int]),
          )
          assertZIO(program.evaluateWith(100))(equalTo(101))
        },
        test("apply some") {
          val program = Lambda.option(Option(Lambda(0)))
          assertZIO(program.evaluate)(equalTo(Some(0)))
        },
        test("apply none") {
          val program = Lambda.option(Option.empty[Int ~> Int])
          assertZIO(program.evaluateWith(0))(equalTo(None))
        },
      ),
      suite("unsafe")(
        test("endpoint /users/1") {
          val endpoint = Endpoint.make("jsonplaceholder.typicode.com").withPath("/users/{{id}}")
            .withOutput(Option(TSchema.obj("id" -> TSchema.num, "name" -> TSchema.string)))
          val input    = DynamicValue(Map("id" -> 1))

          for {
            dynamic <- Lambda.unsafe.fromEndpoint(endpoint).evaluateWith(input)
          } yield assertTrue(
            DynamicValueUtil.getPath(dynamic, "id").contains(DynamicValue(BigDecimal(1))),
            DynamicValueUtil.getPath(dynamic, "name").contains(DynamicValue("Leanne Graham")),
          )
        },
        test("error") {
          val endpoint = Endpoint.make("jsonplaceholder.typicode.com").withPath("/users/{{id}}")
            .withOutput(Option(TSchema.obj("id" -> TSchema.num, "name" -> TSchema.string)))
          val program  = Lambda.unsafe.fromEndpoint(endpoint).evaluateWith(DynamicValue(Map("id" -> 100))).flip
            .map(_.getMessage)

          assertZIO(program)(equalTo("HTTP Error: 404 body: {}"))
        },
      ),
    ).provide(EvaluationRuntime.default, HttpClient.live, Client.default, DataLoader.http) @@ timeout(5 seconds)
}
