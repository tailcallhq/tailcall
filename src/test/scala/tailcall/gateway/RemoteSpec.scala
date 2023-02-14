package tailcall.gateway

import tailcall.gateway.ast.{Context, Endpoint}
import tailcall.gateway.internal.{JsonPlaceholder, RemoteAssertion}
import tailcall.gateway.remote.{EvaluationError, Remote}
import zio.Chunk
import zio.schema.{DynamicValue, Schema, TypeId}
import zio.test.Assertion._
import zio.test.TestAspect.failing
import zio.test.{ZIOSpecDefault, assertZIO}

import scala.collection.immutable.ListMap

object RemoteSpec extends ZIOSpecDefault with RemoteAssertion {
  import tailcall.gateway.remote.Equatable._
  import tailcall.gateway.remote.Numeric._
  import tailcall.gateway.remote.Remote._

  implicit def seqSchema[A: Schema]: Schema[Seq[A]] =
    Schema.chunk[A].transform(_.toSeq, Chunk.from(_))

  def spec =
    suite("Remote")(
      suite("math")(
        test("add") {
          val program = Remote(1) + Remote(2)
          assertRemote(program)(equalTo(3))
        },
        test("subtract") {
          val program = Remote(1) - Remote(2)
          assertRemote(program)(equalTo(-1))
        },
        test("multiply") {
          val program = Remote(2) * Remote(3)
          assertRemote(program)(equalTo(6))
        },
        test("divide") {
          val program = Remote(6) / Remote(3)
          assertRemote(program)(equalTo(2))
        },
        test("modulo") {
          val program = Remote(7) % Remote(3)
          assertRemote(program)(equalTo(1))
        },
        test("greater than") {
          val program = Remote(2) > Remote(1)
          assertRemote(program)(isTrue)
        }
      ),
      suite("logical")(
        test("and") {
          val program = Remote(true) && Remote(true)
          assertRemote(program)(isTrue)
        },
        test("or") {
          val program = Remote(true) || Remote(false)
          assertRemote(program)(isTrue)
        },
        test("not") {
          val program = !Remote(true)
          assertRemote(program)(isFalse)
        }
      ),
      suite("equals")(
        test("equal") {
          val program = Remote(1) =:= Remote(1)
          assertRemote(program)(isTrue)
        },
        test("not equal") {
          val program = Remote(1) =:= Remote(2)
          assertRemote(program)(isFalse)
        }
      ),
      suite("diverge")(
        test("isTrue") {
          val program = Remote(true).diverge(Remote("Yes"), Remote("No"))
          assertRemote(program)(equalTo("Yes"))
        },
        test("isFalse") {
          val program = Remote(false).diverge(Remote("Yes"), Remote("No"))
          assertRemote(program)(equalTo("No"))
        }
      ),
      suite("string")(
        test("concat") {
          val program = Remote("Hello") ++ Remote(" ") ++ Remote("World!")
          assertRemote(program)(equalTo("Hello World!"))
        },
        test("template string") {
          val program = rs"Hello ${Remote("World")}!"
          assertRemote(program)(equalTo("Hello World!"))
        }
      ),
      suite("seq")(
        test("concat") {
          val program = Remote(Seq(1, 2)) ++ Remote(Seq(3, 4))
          assertRemote(program)(equalTo(Seq(1, 2, 3, 4)))
        },
        test("reverse") {
          val program = Remote(Seq(1, 2, 3)).reverse
          assertRemote(program)(equalTo(Seq(3, 2, 1)))
        },
        test("length") {
          val program = Remote(Seq(1, 2, 3)).length
          assertRemote(program)(equalTo(3))
        },
        test("indexOf") {
          val program = Remote(Seq(1, 2, 3)).indexOf(Remote(2))
          assertRemote(program)(equalTo(1))
        },
        test("filter") {
          val program =
            Remote(Seq(1, 2, 3, 4)).filter(r => r % Remote(2) =:= Remote(0))
          assertRemote(program)(equalTo(Seq(2, 4)))
        },
        test("map") {
          val program = Remote(Seq(1, 2, 3, 4)).map(r => r * Remote(2))
          assertRemote(program)(equalTo(Seq(2, 4, 6, 8)))
        },
        test("flatMap") {
          val program = for {
            r   <- Remote(Seq(1, 2, 3, 4))
            seq <- Remote.fromSeq(Seq(r, r * Remote(2)))
          } yield seq
          assertRemote(program)(equalTo(Seq(1, 2, 2, 4, 3, 6, 4, 8)))
        },
        test("groupBy") {
          val program = Remote(Seq(1, 2, 3, 4)).groupBy(r => r % Remote(2))
          assertRemote(program)(equalTo(
            Seq((1, Seq(1, 3)), (0, Seq(2, 4))).sortBy(_._1)
          ))
        },
        test("slice") {
          val program = Remote(Seq(1, 2, 3, 4)).slice(1, 3)
          assertRemote(program)(equalTo(Seq(2, 3)))
        },
        test("take") {
          val program = Remote(Seq(1, 2, 3, 4)).take(2)
          assertRemote(program)(equalTo(Seq(1, 2)))
        }
      ),
      suite("function")(
        test("apply") {
          val function = Remote.fromFunction[Int, Int](_.increment)
          val program  = function(Remote(1))
          assertRemote(program)(equalTo(2))
        },
        test("toFunction") {
          val function = Remote.fromFunction[Int, Int](_.increment)
          val program  = function.toFunction(Remote(1))
          assertRemote(program)(equalTo(2))
        },
        test("pipe") {
          val f       = Remote.fromFunction[Int, Int](_.increment)
          val g       = Remote.fromFunction[Int, Int](_.increment)
          val fg      = f >>> g
          val program = fg(Remote(1))
          assertRemote(program)(equalTo(3))
        },
        test("compose") {
          val f       = Remote.fromFunction[Int, Int](_.increment)
          val g       = Remote.fromFunction[Int, Int](_.increment)
          val fg      = f <<< g
          val program = fg(Remote(1))
          assertRemote(program)(equalTo(3))
        }
      ),
      suite("either")(
        test("left") {
          val program = Remote.fromEither(Left(Remote("Error")))
          assertRemote(program)(equalTo(Left("Error")))
        },
        test("right") {
          val program = Remote.fromEither(Right(Remote(1)))
          assertRemote(program)(equalTo(Right(1)))
        },
        test("fold right") {
          val program = Remote
            .fromEither(Right(Remote(1)))
            .fold((l: Remote[Nothing]) => l.length, r => r * Remote(2))
          assertRemote(program)(equalTo(2))
        },
        test("fold left") {
          val program = Remote
            .fromEither(Left(Remote("Error")))
            .fold(l => rs"Some ${l}", (r: Remote[Nothing]) => r * Remote(2))
          assertRemote(program)(equalTo("Some Error"))
        }
      ),
      suite("option")(
        test("some") {
          val program = Remote.fromOption(Some(Remote(1)))
          assertRemote(program)(equalTo(Some(1)))
        },
        test("none") {
          val program = Remote.fromOption(None)
          assertRemote(program)(equalTo(None))
        },
        test("isSome") {
          val program = Remote.fromOption(Some(Remote(1))).isSome
          assertRemote(program)(isTrue)
        },
        test("isNone") {
          val program = Remote.fromOption(None).isNone
          assertRemote(program)(isTrue)
        },
        test("fold some") {
          val program =
            Remote.fromOption(Some(Remote(1))).fold(Remote(0))(_ * Remote(2))
          assertRemote(program)(equalTo(2))
        },
        test("fold none") {
          val program = Remote.fromOption(None).fold(Remote(0))(_ * Remote(2))
          assertRemote(program)(equalTo(0))
        }
      ),
      test("record") {
        val program = Remote.record(
          "a" -> Remote(DynamicValue(1)),
          "b" -> Remote(DynamicValue(2))
        )
        assertRemote(program)(equalTo(DynamicValue.Record(
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
            val program = Remote(context).parent.map(_.value)
            assertRemote(program)(equalTo(Some(DynamicValue(2))))
          },
          test("not present") {
            val context = Context(
              DynamicValue(1),
              parent = Option(Context(DynamicValue(2)))
            )
            val program = Remote(context).parent.flatMap(_.parent)
            assertRemote(program)(equalTo(None))
          },
          test("nested") {
            val context = Context(
              DynamicValue(1),
              parent = Option(Context(
                DynamicValue(2),
                parent = Option(Context(DynamicValue(3)))
              ))
            )
            val program = Remote(context).parent.flatMap(_.parent).map(_.value)
            assertRemote(program)(equalTo(Some(DynamicValue(3))))
          }
        ),
        test("value") {
          val program = Remote(Context(DynamicValue(1))).value
          assertRemote(program)(equalTo(DynamicValue(1)))
        },
        test("arg") {
          val context = Context(
            DynamicValue(1),
            args = ListMap.from(List("a" -> DynamicValue(2)))
          )
          val program = Remote(context).arg("a")
          assertRemote(program)(equalTo(Option(DynamicValue(2))))
        }
      ),
      suite("die")(
        test("literal") {
          val program = Remote.die("Error")
          assertZIO(program.toZIO.exit)(fails(
            equalTo(EvaluationError.Death("Error"))
          ))
        },
        test("remote") {
          val program = Remote.die(Remote("Error"))
          assertZIO(program.toZIO.exit)(fails(
            equalTo(EvaluationError.Death("Error"))
          ))
        }
      ),
      suite("dynamicValue")(
        suite("path")(
          test("path not found") {
            val program = Remote(DynamicValue(1)).path("a")
            assertRemote(program)(equalTo(Option.empty[DynamicValue]))
          },
          test("path found") {
            val program =
              Remote.record("a" -> Remote(DynamicValue(1))).path("a")
            assertRemote(program)(equalTo(Option(DynamicValue(1))))
          }
        ),
        suite("asString")(
          test("string") {
            val program = Remote(DynamicValue("a")).asString
            assertRemote(program)(equalTo(Option("a")))
          },
          test("not string") {
            val program = Remote(DynamicValue(1)).asString
            assertRemote(program)(equalTo(Option.empty[String]))
          }
        ),
        suite("asBoolean")(
          test("boolean") {
            val program = Remote(DynamicValue(true)).asBoolean
            assertRemote(program)(equalTo(Option(true)))
          },
          test("not boolean") {
            val program = Remote(DynamicValue(1)).asBoolean
            assertRemote(program)(equalTo(Option.empty[Boolean]))
          }
        ),
        suite("asInt")(
          test("int") {
            val program = Remote(DynamicValue(1)).asInt
            assertRemote(program)(equalTo(Option(1)))
          },
          test("not int") {
            val program = Remote(DynamicValue("a")).asInt
            assertRemote(program)(equalTo(Option.empty[Int]))
          }
        ),
        suite("asLong")(
          test("long") {
            val program = Remote(DynamicValue(1L)).asLong
            assertRemote(program)(equalTo(Option(1L)))
          },
          test("not long") {
            val program = Remote(DynamicValue("a")).asLong
            assertRemote(program)(equalTo(Option.empty[Long]))
          }
        ),
        suite("asDouble")(
          test("double") {
            val program = Remote(DynamicValue(1.0)).asDouble
            assertRemote(program)(equalTo(Option(1.0)))
          },
          test("not double") {
            val program = Remote(DynamicValue("a")).asDouble
            assertRemote(program)(equalTo(Option.empty[Double]))
          }
        ),
        suite("asFloat")(
          test("float") {
            val program = Remote(DynamicValue(1.0f)).asFloat
            assertRemote(program)(equalTo(Option(1.0f)))
          },
          test("not float") {
            val program = Remote(DynamicValue("a")).asFloat
            assertRemote(program)(equalTo(Option.empty[Float]))
          }
        ),
        suite("asList")(
          test("list") {
            val program  = Remote(DynamicValue(List(1, 2, 3))).asList
            val expected =
              Option(List(DynamicValue(1), DynamicValue(2), DynamicValue(3)))
            assertRemote(program)(equalTo(expected))
          },
          test("not list") {
            val program = Remote(DynamicValue("a")).asList
            assertRemote(program)(equalTo(Option.empty[List[DynamicValue]]))
          }
        ),
        suite("asMap")(
          test("map") {
            val program  = Remote(DynamicValue(Map("a" -> 1, "b" -> 2))).asMap
            val expected = Option(Map(
              DynamicValue("a") -> DynamicValue(1),
              DynamicValue("b") -> DynamicValue(2)
            ))
            assertRemote(program)(equalTo(expected))
          },
          test("not map") {
            val program = Remote(DynamicValue("a")).asMap
            assertRemote(program)(equalTo(
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
        val program  = Remote
          .fromEndpoint(endpoint)(Remote(DynamicValue(Map("id" -> 1))))
        val expected = DynamicValue(JsonPlaceholder.User(1, "Leanne Graham"))
        assertRemote(program)(equalTo(expected))
      }),
      suite("tuple")(
        test("_1") {
          val program = Remote((1, 2))._1
          assertRemote(program)(equalTo(1))
        },
        test("_2") {
          val program = Remote((1, 2))._2
          assertRemote(program)(equalTo(2))
        },
        test("fromTuple 2") {
          val program = Remote.fromTuple((Remote(1), Remote(2)))
          assertRemote(program)(equalTo((1, 2)))
        },
        test("fromTuple 3") {
          val program = Remote.fromTuple((Remote(1), Remote(2), Remote(3)))
          assertRemote(program)(equalTo((1, 2, 3)))
        }
      ),
      suite("batch")(test("option") {
        val from    = Remote(Seq((1, "john"), (2, "richard"), (3, "paul")))
        val to      =
          (_: Any) => Remote(Seq((1, "london"), (2, "paris"), (3, "new york")))
        val program = Remote.batch(
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
        assertRemote(program)(equalTo(expected))
      })
    )
}
