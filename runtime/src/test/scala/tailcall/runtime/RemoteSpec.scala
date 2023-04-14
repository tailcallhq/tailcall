package tailcall.runtime

import tailcall.runtime.http.HttpClient
import tailcall.runtime.internal.DynamicValueUtil
import tailcall.runtime.model.{Context, Endpoint, TSchema}
import tailcall.runtime.remote.Remote.{logic, math}
import tailcall.runtime.remote._
import tailcall.runtime.service.{DataLoader, EvaluationRuntime}
import zio.durationInt
import zio.http.Client
import zio.schema.DynamicValue
import zio.test.Assertion._
import zio.test.TestAspect.timeout
import zio.test._

object RemoteSpec extends ZIOSpecDefault {
  import tailcall.runtime.remote.Numeric._

  def spec =
    suite("Remote")(
      suite("math")(
        test("add") {
          val program = Remote(1) + Remote(2)
          assertZIO(program.evaluate)(equalTo(3))
        },
        test("subtract") {
          val program = Remote(1) - Remote(2)
          assertZIO(program.evaluate)(equalTo(-1))
        },
        test("multiply") {
          val program = Remote(2) * Remote(3)
          assertZIO(program.evaluate)(equalTo(6))
        },
        test("divide") {
          val program = Remote(6) / Remote(3)
          assertZIO(program.evaluate)(equalTo(2))
        },
        test("modulo") {
          val program = Remote(7) % Remote(3)
          assertZIO(program.evaluate)(equalTo(1))
        },
        test("greater than") {
          val program = Remote(2) > Remote(1)
          assertZIO(program.evaluate)(isTrue)
        },
      ),
      suite("logical")(
        test("and") {
          val program = Remote(true) && Remote(true)
          assertZIO(program.evaluate)(isTrue)
        },
        test("or") {
          val program = Remote(true) || Remote(false)
          assertZIO(program.evaluate)(isTrue)
        },
        test("not") {
          val program = !Remote(true)
          assertZIO(program.evaluate)(isFalse)
        },
      ),
//      suite("equals")(
//        test("equal") {
//          val program = Remote(1) =:= Remote(1)
//          assertZIO(program.evaluate)(isTrue)
//        },
//        test("not equal") {
//          val program = Remote(1) =:= Remote(2)
//          assertZIO(program.evaluate)(isFalse)
//        }
//      ),
      suite("diverge")(
        test("isTrue") {
          val program = Remote(true).diverge(Remote("Yes"), Remote("No"))
          assertZIO(program.evaluate)(equalTo("Yes"))
        },
        test("isFalse") {
          val program = Remote(false).diverge(Remote("Yes"), Remote("No"))
          assertZIO(program.evaluate)(equalTo("No"))
        },
      ),
      suite("fromFunction")(
        test("one level") {
          val program = Remote.fromFunction[Int, Int](i => i + Remote(1))
          assertZIO(program.evaluateWith(1))(equalTo(2))
        },
        test("two level") {
          val program = Remote.fromFunction[Int, Int] { i =>
            val f1 = Remote.fromFunction[Int, Int](j => i * j)
            f1(i + Remote(1))
          }(Remote(10))
          assertZIO(program.evaluate)(equalTo(110))
        },
        test("three level") {
          val program = Remote.fromFunction[Int, Int] { i =>
            val f1 = Remote.fromFunction[Int, Int] { j =>
              val f2 = Remote.fromFunction[Int, Int](k => i * j * k)
              f2(j + Remote(1))
            }
            f1(i + Remote(1))
          }(Remote(10))
          assertZIO(program.evaluate)(equalTo(10 * 11 * 12))
        },
        test("three level") {
          val program = Remote.fromFunction[Int, Int] { i =>
            val f1 = Remote.fromFunction[Int, Int](j => j * i)
            val f2 = Remote.fromFunction[Int, Int](j => i * j)
            f1(i + Remote(1)) + f2(i - Remote(1))
          }(Remote(10))
          assertZIO(program.evaluate)(equalTo(200))
        },
      ),
//      suite("string")(
//        test("concat") {
//          val program = Remote("Hello") ++ Remote(" ") ++ Remote("World!")
//          assertZIO(program.evaluate)(equalTo("Hello World!"))
//        },
//        test("template string") {
//          val program = rs"Hello ${Remote("World")}!"
//          assertZIO(program.evaluate)(equalTo("Hello World!"))
//        }
//      ),
//      suite("seq")(
//        test("concat") {
//          val program = Remote(Seq(1, 2)) ++ Remote(Seq(3, 4))
//          assertZIO(program.evaluate)(equalTo(Seq(1, 2, 3, 4)))
//        },
//        test("reverse") {
//          val program = Remote(Seq(1, 2, 3)).reverse
//          assertZIO(program.evaluate)(equalTo(Seq(3, 2, 1)))
//        },
//        test("length") {
//          val program = Remote(Seq(1, 2, 3)).length
//          assertZIO(program.evaluate)(equalTo(3))
//        },
//        test("indexOf") {
//          val program = Remote(Seq(1, 2, 3)).indexOf(Remote(2))
//          assertZIO(program.evaluate)(equalTo(1))
//        },
//        test("filter") {
//          val program =
//            Remote(Seq(1, 2, 3, 4)).filter(r => r % Remote(2) =:= Remote(0))
//          assertZIO(program.evaluate)(equalTo(Seq(2, 4)))
//        },
//        test("filter empty") {
//          val program =
//            Remote(Seq(1, 5, 3, 7)).filter(r => r % Remote(2) =:= Remote(0))
//          assertZIO(program.evaluate)(equalTo(Seq.empty[Int]))
//        },
//        test("map") {
//          val program = Remote(Seq(1, 2, 3, 4)).map(r => r * Remote(2))
//          assertZIO(program.evaluate)(equalTo(Seq(2, 4, 6, 8)))
//        },
//        test("flatMap") {
//          val program = for {
//            r   <- Remote(Seq(1, 2, 3, 4))
//            seq <- Remote.fromSeq(Seq(r, r * Remote(2)))
//          } yield seq
//          assertZIO(program.evaluate)(equalTo(Seq(1, 2, 2, 4, 3, 6, 4, 8)))
//        },
//        test("groupBy") {
//          val program = Remote(Seq(1, 2, 3, 4)).groupBy(r => r % Remote(2))
//          assertZIO(program.evaluate)(equalTo(
//            Map(1 -> Seq(1, 3), 0 -> Seq(2, 4))
//          ))
//        },
//        test("slice") {
//          val program = Remote(Seq(1, 2, 3, 4)).slice(1, 3)
//          assertZIO(program.evaluate)(equalTo(Seq(2, 3)))
//        },
//        test("take") {
//          val program = Remote(Seq(1, 2, 3, 4)).take(2)
//          assertZIO(program.evaluate)(equalTo(Seq(1, 2)))
//        },
//        test("head") {
//          val program = Remote(Seq(1, 2, 3, 4)).head
//          assertZIO(program.evaluate)(equalTo(Option(1)))
//        },
//        test("head empty") {
//          val program = Remote(Seq.empty[Int]).head
//          assertZIO(program.evaluate)(equalTo(Option.empty[Int]))
//        }
//      ),
//      suite("function")(
//        test("apply") {
//          val function = Remote.fromFunction[Int, Int](_.increment)
//          val program  = function(Remote(1))
//          assertZIO(program.evaluate)(equalTo(2))
//        },
//        test("toFunction") {
//          val function = Remote.fromFunction[Int, Int](_.increment)
//          val program  = function.toFunction(Remote(1))
//          assertZIO(program.evaluate)(equalTo(2))
//        },
//        test("pipe") {
//          val f       = Remote.fromFunction[Int, Int](_.increment)
//          val g       = Remote.fromFunction[Int, Int](_.increment)
//          val fg      = f >>> g
//          val program = fg(Remote(1))
//          assertZIO(program.evaluate)(equalTo(3))
//        },
//        test("compose") {
//          val f       = Remote.fromFunction[Int, Int](_.increment)
//          val g       = Remote.fromFunction[Int, Int](_.increment)
//          val fg      = f <<< g
//          val program = fg(Remote(1))
//          assertZIO(program.evaluate)(equalTo(3))
//        },
//        test("multilevel") {
//          val f1      = Remote.fromFunction[Int, Int] { a =>
//            val f2 = Remote.fromFunction[Int, Int] { b =>
//              val f3 = Remote.fromFunction[Int, Int](c => a + b + c)
//              f3(b)
//            }
//
//            f2(a)
//          }
//          val program = f1(Remote(1))
//
//          assertZIO(program.evaluate)(equalTo(3))
//        },
//        test("higher order function") {
//          val f1 = Remote.fromFunction[Int => Int, Int](f => f(Remote(100)))
//          val program = f1(Remote.fromFunction[Int, Int](_.increment))
//          assertZIO(program.evaluate)(equalTo(101))
//        } @@ failing
//      ),
//      suite("either")(
//        test("left") {
//          val program = Remote.fromEither(Left(Remote("Error")))
//          assertZIO(program.evaluate)(equalTo(Left("Error")))
//        },
//        test("right") {
//          val program = Remote.fromEither(Right(Remote(1)))
//          assertZIO(program.evaluate)(equalTo(Right(1)))
//        },
//        test("fold right") {
//          val program = Remote
//            .fromEither(Right(Remote(1)))
//            .fold((l: Remote[Nothing]) => l.length, r => r * Remote(2))
//          assertZIO(program.evaluate)(equalTo(2))
//        },
//        test("fold left") {
//          val program = Remote
//            .fromEither(Left(Remote("Error")))
//            .fold(l => rs"Some ${l}", (r: Remote[Nothing]) => r * Remote(2))
//          assertZIO(program.evaluate)(equalTo("Some Error"))
//        }
//      ),
      suite("option")(
        test("some") {
          val program = Remote.option(Some(Remote(1)))
          assertZIO(program.evaluate)(equalTo(Some(1)))
        },
        test("none") {
          val program = Remote.option(None)
          assertZIO(program.evaluate)(equalTo(None))
        },
        test("isSome") {
          val program = Remote.option(Some(Remote(1))).isSome
          assertZIO(program.evaluate)(isTrue)
        },
        test("isNone") {
          val program = Remote.option(None).isNone
          assertZIO(program.evaluate)(isTrue)
        },
        test("fold some") {
          val program = Remote.option(Some(Remote(1))).fold(Remote(0), _ * Remote(2))
          assertZIO(program.evaluate)(equalTo(2))
        },
        test("fold none") {
          val program = Remote.option(Option.empty[Remote[Any, Int]]).fold(Remote(0), _ * Remote(2))
          assertZIO(program.evaluate)(equalTo(0))
        },
      ),
      suite("dynamicValue")(
        test("int") {
          val program = Remote(1).toDynamic
          assertZIO(program.evaluate)(equalTo(DynamicValue(1)))
        },
        test("some") {
          val program = Remote(Option(1)).toDynamic
          assertZIO(program.evaluate)(equalTo(DynamicValue(Option(1))))
        },
        test("none") {
          val program = Remote(Option.empty[Int]).toDynamic
          assertZIO(program.evaluate)(equalTo(DynamicValue(Option.empty[Int])))
        },
      ),
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
          val program = Remote.fromFunction[Int, Int](i => math.add(i, Remote(1)))
          assertZIO(program.evaluateWith(1))(equalTo(2))
        },
        test("two level") {
          val program = Remote.fromFunction[Int, Int] { i =>
            val f1 = Remote.fromFunction[Int, Int](j => math.mul(i, j))
            math.add(i, Remote(1)) >>> f1
          }
          assertZIO(program.evaluateWith(10))(equalTo(110))
        },
        test("three level") {
          val program = Remote.fromFunction[Int, Int] { i =>
            val f1 = Remote.fromFunction[Int, Int] { j =>
              val f2 = Remote.fromFunction[Int, Int](k => math.mul(math.mul(i, j), k))
              math.add(j, Remote(1)) >>> f2
            }
            math.add(i, Remote(1)) >>> f1
          }
          assertZIO(program.evaluateWith(10))(equalTo(10 * 11 * 12))
        },
        test("nested siblings") {
          val program = Remote.fromFunction[Int, Int] { i =>
            val f1 = Remote.fromFunction[Int, Int](j => math.mul(i, j))
            val f2 = Remote.fromFunction[Int, Int](j => math.mul(i, j))
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
      ),
//      test("record") {
//        val program = Remote.record(
//          "a" -> Remote(DynamicValue(1)),
//          "b" -> Remote(DynamicValue(2))
//        )
//        assertZIO(program.evaluate)(equalTo(DynamicValue.Record(
//          TypeId.Structural,
//          ListMap.from(List("a" -> DynamicValue(1), "b" -> DynamicValue(2)))
//        )))
//      },
//      suite("context")(
//        suite("parent")(
//          test("present") {
//            val context = Context(
//              DynamicValue(1),
//              parent = Option(Context(DynamicValue(2)))
//            )
//            val program = Remote(context).parent.map(_.value)
//            assertZIO(program.evaluate)(equalTo(Some(DynamicValue(2))))
//          },
//          test("not present") {
//            val context = Context(
//              DynamicValue(1),
//              parent = Option(Context(DynamicValue(2)))
//            )
//            val program = Remote(context).parent.flatMap(_.parent)
//            assertZIO(program.evaluate)(equalTo(None))
//          },
//          test("nested") {
//            val context = Context(
//              DynamicValue(1),
//              parent = Option(Context(
//                DynamicValue(2),
//                parent = Option(Context(DynamicValue(3)))
//              ))
//            )
//            val program = Remote(context).parent.flatMap(_.parent).map(_.value)
//            assertZIO(program.evaluate)(equalTo(Some(DynamicValue(3))))
//          }
//        ),
//        test("value") {
//          val program = Remote(Context(DynamicValue(1))).value
//          assertZIO(program.evaluate)(equalTo(DynamicValue(1)))
//        },
//        test("arg") {
//          val context = Context(
//            DynamicValue(1),
//            args = ListMap.from(List("a" -> DynamicValue(2)))
//          )
//          val program = Remote(context).arg("a")
//          assertZIO(program.evaluate)(equalTo(Option(DynamicValue(2))))
//        },
//        test("evaluate") {
//          val program = Remote(Context(DynamicValue(1)))
//          assertZIO(program.evaluate)(equalTo(Context(DynamicValue(1))))
//        }
//      ),
//      suite("die")(
//        test("literal") {
//          val program = Remote.die("Error")
//          assertZIO(program.evaluate.exit)(fails(
//            equalTo(EvaluationError.Death("Error"))
//          ))
//        },
//        test("remote") {
//          val program = Remote.die(Remote("Error"))
//          assertZIO(program.evaluate.exit)(fails(
//            equalTo(EvaluationError.Death("Error"))
//          ))
//        }
//      ),
//      suite("dynamicValue")(
//        suite("path")(
//          test("path not found") {
//            val program = Remote(DynamicValue(1)).path("a")
//            assertZIO(program.evaluate)(equalTo(Option.empty[DynamicValue]))
//          },
//          test("path found") {
//            val program =
//              Remote.record("a" -> Remote(DynamicValue(1))).path("a")
//            assertZIO(program.evaluate)(equalTo(Option(DynamicValue(1))))
//          }
//        ),
//        suite("asString")(
//          test("string") {
//            val program = Remote(DynamicValue("a")).asString
//            assertZIO(program.evaluate)(equalTo(Option("a")))
//          },
//          test("not string") {
//            val program = Remote(DynamicValue(1)).asString
//            assertZIO(program.evaluate)(equalTo(Option.empty[String]))
//          }
//        ),
//        suite("asBoolean")(
//          test("boolean") {
//            val program = Remote(DynamicValue(true)).asBoolean
//            assertZIO(program.evaluate)(equalTo(Option(true)))
//          },
//          test("not boolean") {
//            val program = Remote(DynamicValue(1)).asBoolean
//            assertZIO(program.evaluate)(equalTo(Option.empty[Boolean]))
//          }
//        ),
//        suite("asInt")(
//          test("int") {
//            val program = Remote(DynamicValue(1)).asInt
//            assertZIO(program.evaluate)(equalTo(Option(1)))
//          },
//          test("not int") {
//            val program = Remote(DynamicValue("a")).asInt
//            assertZIO(program.evaluate)(equalTo(Option.empty[Int]))
//          }
//        ),
//        suite("asLong")(
//          test("long") {
//            val program = Remote(DynamicValue(1L)).asLong
//            assertZIO(program.evaluate)(equalTo(Option(1L)))
//          },
//          test("not long") {
//            val program = Remote(DynamicValue("a")).asLong
//            assertZIO(program.evaluate)(equalTo(Option.empty[Long]))
//          }
//        ),
//        suite("asDouble")(
//          test("double") {
//            val program = Remote(DynamicValue(1.0)).asDouble
//            assertZIO(program.evaluate)(equalTo(Option(1.0)))
//          },
//          test("not double") {
//            val program = Remote(DynamicValue("a")).asDouble
//            assertZIO(program.evaluate)(equalTo(Option.empty[Double]))
//          }
//        ),
//        suite("asFloat")(
//          test("float") {
//            val program = Remote(DynamicValue(1.0f)).asFloat
//            assertZIO(program.evaluate)(equalTo(Option(1.0f)))
//          },
//          test("not float") {
//            val program = Remote(DynamicValue("a")).asFloat
//            assertZIO(program.evaluate)(equalTo(Option.empty[Float]))
//          }
//        ),
//        suite("asList")(
//          test("list") {
//            val program  = Remote(DynamicValue(List(1, 2, 3))).asList
//            val expected =
//              Option(List(DynamicValue(1), DynamicValue(2), DynamicValue(3)))
//            assertZIO(program.evaluate)(equalTo(expected))
//          },
//          test("not list") {
//            val program = Remote(DynamicValue("a")).asList
//            assertZIO(program.evaluate)(equalTo(
//              Option.empty[List[DynamicValue]]
//            ))
//          }
//        ),
//        suite("asMap")(
//          test("map") {
//            val program  = Remote(DynamicValue(Map("a" -> 1, "b" -> 2))).asMap
//            val expected = Option(Map(
//              DynamicValue("a") -> DynamicValue(1),
//              DynamicValue("b") -> DynamicValue(2)
//            ))
//            assertZIO(program.evaluate)(equalTo(expected))
//          },
//          test("not map") {
//            val program = Remote(DynamicValue("a")).asMap
//            assertZIO(program.evaluate)(equalTo(
//              Option.empty[Map[DynamicValue, DynamicValue]]
//            ))
//          }
//        )
//      ) @@ failing,
//      suite("endpoint")(test("/users/{{id}}") {
//        val endpoint = Endpoint
//          .make("jsonplaceholder.typicode.com")
//          .withPath("/users/{{id}}")
//          .withOutput[JsonPlaceholder.User]
//        val program  = Remote
//          .fromEndpoint(endpoint)(Remote(DynamicValue(Map("id" -> 1))))
//        val expected = DynamicValue(JsonPlaceholder.User(1, "Leanne Graham"))
//        assertZIO(program.evaluate)(equalTo(expected))
//      }),
//      suite("tuple")(
//        test("_1") {
//          val program = Remote((1, 2))._1
//          assertZIO(program.evaluate)(equalTo(1))
//        },
//        test("_2") {
//          val program = Remote((1, 2))._2
//          assertZIO(program.evaluate)(equalTo(2))
//        },
//        test("fromTuple 2") {
//          val program = Remote.fromTuple((Remote(1), Remote(2)))
//          assertZIO(program.evaluate)(equalTo((1, 2)))
//        },
//        test("fromTuple 3") {
//          val program = Remote.fromTuple((Remote(1), Remote(2), Remote(3)))
//          assertZIO(program.evaluate)(equalTo((1, 2, 3)))
//        }
//      ),
//      suite("batch")(
//        test("option") {
//          val from    = Remote(Seq((1, "john"), (2, "richard"), (3, "paul")))
//          val to      = (_: Any) =>
//            Remote(Seq((1, "london"), (2, "paris"), (3, "new york")))
//          val program = Remote.batch(
//            from,
//            to,
//            (x: Remote[(Int, String)]) => x._1,
//            (b: Remote[Int]) => from.filter(x => x._1 =:= b).head.getOrDie,
//            (y: Remote[(Int, String)]) => y._1
//          )
//
//          val expected = List(
//            ((1, "john"), Some(1, "london")),
//            ((2, "richard"), Some(2, "paris")),
//            ((3, "paul"), Some(3, "new york"))
//          )
//          assertZIO(program.evaluate)(equalTo(expected))
//        },
//        test("option order") {
//          val from    = Remote(Seq((1, "john"), (2, "richard"), (3, "paul")))
//          val to      = (_: Any) =>
//            Remote(Seq((3, "london"), (2, "paris"), (1, "new york")))
//          val program = Remote.batch(
//            from,
//            to,
//            (x: Remote[(Int, String)]) => x._1,
//            (b: Remote[Int]) => from.filter(x => x._1 =:= b).head.getOrDie,
//            (y: Remote[(Int, String)]) => y._1
//          )
//
//          val expected = List(
//            ((1, "john"), Some(1, "new york")),
//            ((2, "richard"), Some(2, "paris")),
//            ((3, "paul"), Some(3, "london"))
//          )
//          assertZIO(program.evaluate)(equalTo(expected))
//        },
//        test("empty") {
//          val from    = Remote(Seq((1, "john"), (2, "richard"), (3, "paul")))
//          val to      = (_: Any) => Remote(Seq((1, "london"), (2, "paris")))
//          val program = Remote.batch(
//            from,
//            to,
//            (x: Remote[(Int, String)]) => x._1,
//            (b: Remote[Int]) => from.find(x => x._1 =:= b).getOrDie,
//            (y: Remote[(Int, String)]) => y._1
//          )
//
//          val expected = List(
//            ((1, "john"), Some(1, "london")),
//            ((2, "richard"), Some(2, "paris")),
//            ((3, "paul"), None)
//          )
//          assertZIO(program.evaluate)(equalTo(expected))
//        }
//      ),
//      suite("map")(
//        test("get some") {
//          val program = Remote(Map("a" -> 1, "b" -> 2)).get(Remote("a"))
//          assertZIO(program.evaluate)(equalTo(Option(1)))
//        },
//        test("get none") {
//          val program = Remote(Map("a" -> 1, "b" -> 2)).get(Remote("c"))
//          assertZIO(program.evaluate)(equalTo(Option.empty[Int]))
//        }
//      ),
//      test("flatten") {
//        val program = Remote.flatten(Remote(Remote(1)))
//        assertZIO(program.evaluate)(equalTo(1))
//      }
    ).provide(EvaluationRuntime.default, HttpClient.live, Client.default, DataLoader.http) @@ timeout(5 seconds)
}
