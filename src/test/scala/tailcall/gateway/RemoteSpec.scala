package tailcall.gateway

import tailcall.gateway.ast.{Context, Endpoint}
import tailcall.gateway.internal.JsonPlaceholder
import tailcall.gateway.remote._
import zio.Chunk
import zio.schema.{DynamicValue, Schema, TypeId}
import zio.test.Assertion._
import zio.test.TestAspect.failing
import zio.test._

import scala.collection.immutable.ListMap

object RemoteSpec extends ZIOSpecDefault {
  import tailcall.gateway.remote.Equatable._
  import tailcall.gateway.remote.Numeric._

  implicit def seqSchema[A: Schema]: Schema[Seq[A]] =
    Schema.chunk[A].transform(_.toSeq, Chunk.from(_))

  def spec =
    suite("Remote")(
      suite("math")(
        test("add") {
          val program = Lambda(1) + Lambda(2)
          assertZIO(program.evaluateWith(()))(equalTo(3))
        },
        test("subtract") {
          val program = Lambda(1) - Lambda(2)
          assertZIO(program.evaluateWith(()))(equalTo(-1))
        },
        test("multiply") {
          val program = Lambda(2) * Lambda(3)
          assertZIO(program.evaluateWith(()))(equalTo(6))
        },
        test("divide") {
          val program = Lambda(6) / Lambda(3)
          assertZIO(program.evaluateWith(()))(equalTo(2))
        },
        test("modulo") {
          val program = Lambda(7) % Lambda(3)
          assertZIO(program.evaluateWith(()))(equalTo(1))
        },
        test("greater than") {
          val program = Lambda(2) > Lambda(1)
          assertZIO(program.evaluateWith(()))(isTrue)
        }
      ),
      suite("logical")(
        test("and") {
          val program = Lambda(true) && Lambda(true)
          assertZIO(program.evaluateWith(()))(isTrue)
        },
        test("or") {
          val program = Lambda(true) || Lambda(false)
          assertZIO(program.evaluateWith(()))(isTrue)
        },
        test("not") {
          val program = !Lambda(true)
          assertZIO(program.evaluateWith(()))(isFalse)
        }
      ),
      suite("equals")(
        test("equal") {
          val program = Lambda(1) =:= Lambda(1)
          assertZIO(program.evaluateWith(()))(isTrue)
        },
        test("not equal") {
          val program = Lambda(1) =:= Lambda(2)
          assertZIO(program.evaluateWith(()))(isFalse)
        }
      ),
      suite("diverge")(
        test("isTrue") {
          val program = Lambda(true).diverge(Lambda("Yes"), Lambda("No"))
          assertZIO(program.evaluateWith(()))(equalTo("Yes"))
        },
        test("isFalse") {
          val program = Lambda(false).diverge(Lambda("Yes"), Lambda("No"))
          assertZIO(program.evaluateWith(()))(equalTo("No"))
        }
      ),
      suite("string")(
        test("concat") {
          val program = Lambda("Hello") ++ Lambda(" ") ++ Lambda("World!")
          assertZIO(program.evaluateWith(()))(equalTo("Hello World!"))
        },
        test("template string") {
          val program = rs"Hello ${Lambda("World")}!"
          assertZIO(program.evaluateWith(()))(equalTo("Hello World!"))
        }
      ),
      suite("seq")(
        test("concat") {
          val program = Lambda(Seq(1, 2)) ++ Lambda(Seq(3, 4))
          assertZIO(program.evaluateWith(()))(equalTo(Seq(1, 2, 3, 4)))
        },
        test("reverse") {
          val program = Lambda(Seq(1, 2, 3)).reverse
          assertZIO(program.evaluateWith(()))(equalTo(Seq(3, 2, 1)))
        },
        test("length") {
          val program = Lambda(Seq(1, 2, 3)).length
          assertZIO(program.evaluateWith(()))(equalTo(3))
        },
        test("indexOf") {
          val program = Lambda(Seq(1, 2, 3)).indexOf(Lambda(2))
          assertZIO(program.evaluateWith(()))(equalTo(1))
        },
        test("filter") {
          val program =
            Lambda(Seq(1, 2, 3, 4)).filter(r => r % Lambda(2) =:= Lambda(0))
          assertZIO(program.evaluateWith(()))(equalTo(Seq(2, 4)))
        },
        test("filter empty") {
          val program =
            Lambda(Seq(1, 5, 3, 7)).filter(r => r % Lambda(2) =:= Lambda(0))
          assertZIO(program.evaluateWith(()))(equalTo(Seq.empty[Int]))
        },
        test("map") {
          val program = Lambda(Seq(1, 2, 3, 4)).map(r => r * Lambda(2))
          assertZIO(program.evaluateWith(()))(equalTo(Seq(2, 4, 6, 8)))
        },
        test("flatMap") {
          val program = for {
            r   <- Lambda(Seq(1, 2, 3, 4))
            seq <- Lambda.fromSeq(Seq(r, r * Lambda(2)))
          } yield seq
          assertZIO(program.evaluateWith(()))(equalTo(Seq(1, 2, 2, 4, 3, 6, 4,
            8)))
        },
        test("groupBy") {
          val program = Lambda(Seq(1, 2, 3, 4)).groupBy(r => r % Lambda(2))
          assertZIO(program.evaluateWith(()))(equalTo(
            Map(1 -> Seq(1, 3), 0 -> Seq(2, 4))
          ))
        },
        test("slice") {
          val program = Lambda(Seq(1, 2, 3, 4)).slice(1, 3)
          assertZIO(program.evaluateWith(()))(equalTo(Seq(2, 3)))
        },
        test("take") {
          val program = Lambda(Seq(1, 2, 3, 4)).take(2)
          assertZIO(program.evaluateWith(()))(equalTo(Seq(1, 2)))
        },
        test("head") {
          val program = Lambda(Seq(1, 2, 3, 4)).head
          assertZIO(program.evaluateWith(()))(equalTo(Option(1)))
        },
        test("head empty") {
          val program = Lambda(Seq.empty[Int]).head
          assertZIO(program.evaluateWith(()))(equalTo(Option.empty[Int]))
        }
      ),
      suite("function")(
        test("apply") {
          val function = Lambda.fromFunction[Int, Int](_.increment)
          val program  = function(Lambda(1))
          assertZIO(program.evaluateWith(()))(equalTo(2))
        },
        test("toFunction") {
          val function = Lambda.fromFunction[Int, Int](_.increment)
          val program  = function.toFunction(Lambda(1))
          assertZIO(program.evaluateWith(()))(equalTo(2))
        },
        test("pipe") {
          val f       = Lambda.fromFunction[Int, Int](_.increment)
          val g       = Lambda.fromFunction[Int, Int](_.increment)
          val fg      = f >>> g
          val program = fg(Lambda(1))
          assertZIO(program.evaluateWith(()))(equalTo(3))
        },
        test("compose") {
          val f       = Lambda.fromFunction[Int, Int](_.increment)
          val g       = Lambda.fromFunction[Int, Int](_.increment)
          val fg      = f <<< g
          val program = fg(Lambda(1))
          assertZIO(program.evaluateWith(()))(equalTo(3))
        },
        test("multilevel") {
          val f1      = Lambda.fromFunction[Int, Int] { a =>
            val f2 = Lambda.fromFunction[Int, Int] { b =>
              val f3 = Lambda.fromFunction[Int, Int](c => a + b + c)
              f3(b)
            }

            f2(a)
          }
          val program = f1(Lambda(1))

          assertZIO(program.evaluateWith(()))(equalTo(3))
        },
        test("higher order function") {
          val f1      = Lambda
            .fromFunction[Int ~> Int, Int](f => Lambda.flatten(f)(Lambda(100)))
          val program = f1(Lambda.fromFunction[Int, Int](_.increment))
          assertZIO(program.evaluateWith(()))(equalTo(101))
        } @@ failing
      ),
      suite("either")(
        test("left") {
          val program = Lambda.fromEither(Left(Lambda("Error")))
          assertZIO(program.evaluateWith(()))(equalTo(Left("Error")))
        },
        test("right") {
          val program = Lambda.fromEither(Right(Lambda(1)))
          assertZIO(program.evaluateWith(()))(equalTo(Right(1)))
        },
        test("fold right") {
          val program = Lambda
            .fromEither(Right(Lambda(1)))
            .fold((l: Remote[Nothing]) => l.length, r => r * Lambda(2))
          assertZIO(program.evaluateWith(()))(equalTo(2))
        },
        test("fold left") {
          val program = Lambda
            .fromEither(Left(Lambda("Error")))
            .fold(l => rs"Some ${l}", (r: Remote[Nothing]) => r * Lambda(2))
          assertZIO(program.evaluateWith(()))(equalTo("Some Error"))
        }
      ),
      suite("option")(
        test("some") {
          val program = Lambda.fromOption(Some(Lambda(1)))
          assertZIO(program.evaluateWith(()))(equalTo(Some(1)))
        },
        test("none") {
          val program = Lambda.fromOption(None)
          assertZIO(program.evaluateWith(()))(equalTo(None))
        },
        test("isSome") {
          val program = Lambda.fromOption(Some(Lambda(1))).isSome
          assertZIO(program.evaluateWith(()))(isTrue)
        },
        test("isNone") {
          val program = Lambda.fromOption(None).isNone
          assertZIO(program.evaluateWith(()))(isTrue)
        },
        test("fold some") {
          val program =
            Lambda.fromOption(Some(Lambda(1))).fold(Lambda(0))(_ * Lambda(2))
          assertZIO(program.evaluateWith(()))(equalTo(2))
        },
        test("fold none") {
          val program = Lambda.fromOption(None).fold(Lambda(0))(_ * Lambda(2))
          assertZIO(program.evaluateWith(()))(equalTo(0))
        }
      ),
      test("record") {
        val program = Lambda.record(
          "a" -> Lambda(DynamicValue(1)),
          "b" -> Lambda(DynamicValue(2))
        )
        assertZIO(program.evaluateWith(()))(equalTo(DynamicValue.Record(
          TypeId.Structural,
          ListMap.from(List("a" -> DynamicValue(1), "b" -> DynamicValue(2)))
        )))
      },
      suite("context")(
        suite("parent")(
          test("present") {
            val context = Context(
              DynamicValue(1),
              parent = Option(Context(DynamicValue(2)))
            )
            val program = Lambda(context).parent.map(_.value)
            assertZIO(program.evaluateWith(()))(equalTo(Some(DynamicValue(2))))
          },
          test("not present") {
            val context = Context(
              DynamicValue(1),
              parent = Option(Context(DynamicValue(2)))
            )
            val program = Lambda(context).parent.flatMap(_.parent)
            assertZIO(program.evaluateWith(()))(equalTo(None))
          },
          test("nested") {
            val context = Context(
              DynamicValue(1),
              parent = Option(Context(
                DynamicValue(2),
                parent = Option(Context(DynamicValue(3)))
              ))
            )
            val program = Lambda(context).parent.flatMap(_.parent).map(_.value)
            assertZIO(program.evaluateWith(()))(equalTo(Some(DynamicValue(3))))
          }
        ),
        test("value") {
          val program = Lambda(Context(DynamicValue(1))).value
          assertZIO(program.evaluateWith(()))(equalTo(DynamicValue(1)))
        },
        test("arg") {
          val context = Context(
            DynamicValue(1),
            args = ListMap.from(List("a" -> DynamicValue(2)))
          )
          val program = Lambda(context).arg("a")
          assertZIO(program.evaluateWith(()))(equalTo(Option(DynamicValue(2))))
        },
        test("evaluate") {
          val program = Lambda(Context(DynamicValue(1)))
          assertZIO(program.evaluateWith(()))(equalTo(Context(DynamicValue(1))))
        }
      ),
      suite("die")(
        test("literal") {
          val program = Lambda.die("Error")
          assertZIO(program.evaluateWith(()).exit)(fails(
            equalTo(EvaluationError.Death("Error"))
          ))
        },
        test("remote") {
          val program = Lambda.die(Lambda("Error"))
          assertZIO(program.evaluateWith(()).exit)(fails(
            equalTo(EvaluationError.Death("Error"))
          ))
        }
      ),
      suite("dynamicValue")(
        suite("path")(
          test("path not found") {
            val program = Lambda(DynamicValue(1)).path("a")
            assertZIO(program.evaluateWith(()))(equalTo(
              Option.empty[DynamicValue]
            ))
          },
          test("path found") {
            val program =
              Lambda.record("a" -> Lambda(DynamicValue(1))).path("a")
            assertZIO(program.evaluateWith(()))(equalTo(
              Option(DynamicValue(1))
            ))
          }
        ),
        suite("asString")(
          test("string") {
            val program = Lambda(DynamicValue("a")).asString
            assertZIO(program.evaluateWith(()))(equalTo(Option("a")))
          },
          test("not string") {
            val program = Lambda(DynamicValue(1)).asString
            assertZIO(program.evaluateWith(()))(equalTo(Option.empty[String]))
          }
        ),
        suite("asBoolean")(
          test("boolean") {
            val program = Lambda(DynamicValue(true)).asBoolean
            assertZIO(program.evaluateWith(()))(equalTo(Option(true)))
          },
          test("not boolean") {
            val program = Lambda(DynamicValue(1)).asBoolean
            assertZIO(program.evaluateWith(()))(equalTo(Option.empty[Boolean]))
          }
        ),
        suite("asInt")(
          test("int") {
            val program = Lambda(DynamicValue(1)).asInt
            assertZIO(program.evaluateWith(()))(equalTo(Option(1)))
          },
          test("not int") {
            val program = Lambda(DynamicValue("a")).asInt
            assertZIO(program.evaluateWith(()))(equalTo(Option.empty[Int]))
          }
        ),
        suite("asLong")(
          test("long") {
            val program = Lambda(DynamicValue(1L)).asLong
            assertZIO(program.evaluateWith(()))(equalTo(Option(1L)))
          },
          test("not long") {
            val program = Lambda(DynamicValue("a")).asLong
            assertZIO(program.evaluateWith(()))(equalTo(Option.empty[Long]))
          }
        ),
        suite("asDouble")(
          test("double") {
            val program = Lambda(DynamicValue(1.0)).asDouble
            assertZIO(program.evaluateWith(()))(equalTo(Option(1.0)))
          },
          test("not double") {
            val program = Lambda(DynamicValue("a")).asDouble
            assertZIO(program.evaluateWith(()))(equalTo(Option.empty[Double]))
          }
        ),
        suite("asFloat")(
          test("float") {
            val program = Lambda(DynamicValue(1.0f)).asFloat
            assertZIO(program.evaluateWith(()))(equalTo(Option(1.0f)))
          },
          test("not float") {
            val program = Lambda(DynamicValue("a")).asFloat
            assertZIO(program.evaluateWith(()))(equalTo(Option.empty[Float]))
          }
        ),
        suite("asList")(
          test("list") {
            val program  = Lambda(DynamicValue(List(1, 2, 3))).asList
            val expected =
              Option(List(DynamicValue(1), DynamicValue(2), DynamicValue(3)))
            assertZIO(program.evaluateWith(()))(equalTo(expected))
          },
          test("not list") {
            val program = Lambda(DynamicValue("a")).asList
            assertZIO(program.evaluateWith(()))(equalTo(
              Option.empty[List[DynamicValue]]
            ))
          }
        ),
        suite("asMap")(
          test("map") {
            val program  = Lambda(DynamicValue(Map("a" -> 1, "b" -> 2))).asMap
            val expected = Option(Map(
              DynamicValue("a") -> DynamicValue(1),
              DynamicValue("b") -> DynamicValue(2)
            ))
            assertZIO(program.evaluateWith(()))(equalTo(expected))
          },
          test("not map") {
            val program = Lambda(DynamicValue("a")).asMap
            assertZIO(program.evaluateWith(()))(equalTo(
              Option.empty[Map[DynamicValue, DynamicValue]]
            ))
          }
        )
      ) @@ failing,
      suite("endpoint")(test("/users/{{id}}") {
        val endpoint = Endpoint
          .make("jsonplaceholder.typicode.com")
          .withPath("/users/{{id}}")
          .withOutput[JsonPlaceholder.User]
        val program  = Lambda
          .fromEndpoint(endpoint)(Lambda(DynamicValue(Map("id" -> 1))))
        val expected = DynamicValue(JsonPlaceholder.User(1, "Leanne Graham"))
        assertZIO(program.evaluateWith(()))(equalTo(expected))
      }),
      suite("tuple")(
        test("_1") {
          val program = Lambda((1, 2))._1
          assertZIO(program.evaluateWith(()))(equalTo(1))
        },
        test("_2") {
          val program = Lambda((1, 2))._2
          assertZIO(program.evaluateWith(()))(equalTo(2))
        },
        test("fromTuple 2") {
          val program = Lambda.fromTuple((Lambda(1), Lambda(2)))
          assertZIO(program.evaluateWith(()))(equalTo((1, 2)))
        },
        test("fromTuple 3") {
          val program = Lambda.fromTuple((Lambda(1), Lambda(2), Lambda(3)))
          assertZIO(program.evaluateWith(()))(equalTo((1, 2, 3)))
        }
      ),
      suite("batch")(
        test("option") {
          val from    = Lambda(Seq((1, "john"), (2, "richard"), (3, "paul")))
          val to      = (_: Any) =>
            Lambda(Seq((1, "london"), (2, "paris"), (3, "new york")))
          val program = Lambda.batch(
            from,
            to,
            (x: Remote[(Int, String)]) => x._1,
            (b: Remote[Int]) => from.filter(x => x._1 =:= b).head.getOrDie,
            (y: Remote[(Int, String)]) => y._1
          )

          val expected = List(
            ((1, "john"), Some(1, "london")),
            ((2, "richard"), Some(2, "paris")),
            ((3, "paul"), Some(3, "new york"))
          )
          assertZIO(program.evaluateWith(()))(equalTo(expected))
        },
        test("option order") {
          val from    = Lambda(Seq((1, "john"), (2, "richard"), (3, "paul")))
          val to      = (_: Any) =>
            Lambda(Seq((3, "london"), (2, "paris"), (1, "new york")))
          val program = Lambda.batch(
            from,
            to,
            (x: Remote[(Int, String)]) => x._1,
            (b: Remote[Int]) => from.filter(x => x._1 =:= b).head.getOrDie,
            (y: Remote[(Int, String)]) => y._1
          )

          val expected = List(
            ((1, "john"), Some(1, "new york")),
            ((2, "richard"), Some(2, "paris")),
            ((3, "paul"), Some(3, "london"))
          )
          assertZIO(program.evaluateWith(()))(equalTo(expected))
        },
        test("empty") {
          val from    = Lambda(Seq((1, "john"), (2, "richard"), (3, "paul")))
          val to      = (_: Any) => Lambda(Seq((1, "london"), (2, "paris")))
          val program = Lambda.batch(
            from,
            to,
            (x: Remote[(Int, String)]) => x._1,
            (b: Remote[Int]) => from.find(x => x._1 =:= b).getOrDie,
            (y: Remote[(Int, String)]) => y._1
          )

          val expected = List(
            ((1, "john"), Some(1, "london")),
            ((2, "richard"), Some(2, "paris")),
            ((3, "paul"), None)
          )
          assertZIO(program.evaluateWith(()))(equalTo(expected))
        }
      ),
      suite("map")(
        test("get some") {
          val program = Lambda(Map("a" -> 1, "b" -> 2)).get(Lambda("a"))
          assertZIO(program.evaluateWith(()))(equalTo(Option(1)))
        },
        test("get none") {
          val program = Lambda(Map("a" -> 1, "b" -> 2)).get(Lambda("c"))
          assertZIO(program.evaluateWith(()))(equalTo(Option.empty[Int]))
        }
      ),
      test("flatten") {
        val program = Lambda.flatten(Lambda(Lambda(1)))
        assertZIO(program.evaluateWith(()))(equalTo(1))
      }
    ).provide(LambdaRuntime.live, EvaluationContext.live)
}
