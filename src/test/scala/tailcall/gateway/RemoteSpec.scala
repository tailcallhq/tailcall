package tailcall.gateway

import tailcall.gateway.ast.Context
import tailcall.gateway.internal.RemoteAssertion
import tailcall.gateway.remote.{Remote, UnsafeEvaluator}
import zio.Chunk
import zio.schema.{DynamicValue, Schema, TypeId}
import zio.test.Assertion.{equalTo, fails, isFalse, isTrue}
import zio.test.{ZIOSpecDefault, assertZIO}

import scala.collection.immutable.ListMap

object RemoteSpec extends ZIOSpecDefault with RemoteAssertion {
  import tailcall.gateway.remote.Numeric._
  import tailcall.gateway.remote.Equatable._
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
          val program = Remote(Seq(1, 2, 3, 4)).filter(r => r % Remote(2) =:= Remote(0))
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
          val program = Remote.fromOption(Some(Remote(1))).fold(Remote(0))(_ * Remote(2))
          assertRemote(program)(equalTo(2))
        },
        test("fold none") {
          val program = Remote.fromOption(None).fold(Remote(0))(_ * Remote(2))
          assertRemote(program)(equalTo(0))
        }
      ),
      test("record") {
        val program = Remote.record("a" -> Remote(DynamicValue(1)), "b" -> Remote(DynamicValue(2)))
        assertRemote(program)(equalTo(DynamicValue.Record(
          TypeId.Structural,
          ListMap.from(List("a" -> DynamicValue(1), "b" -> DynamicValue(2)))
        )))
      },
      suite("context")(
        test("value") {
          val program = Remote(Context(DynamicValue(1))).value
          assertRemote(program)(equalTo(DynamicValue(1)))
        },
        test("parent") {
          val context = Context(DynamicValue(1), parent = Option(Context(DynamicValue(2))))
          val program = Remote(context).parent
          assertRemote(program)(equalTo(Option(Context(DynamicValue(2)))))
        },
        test("arg") {
          val context = Context(DynamicValue(1), args = ListMap.from(List("a" -> DynamicValue(2))))
          val program = Remote(context).arg("a")
          assertRemote(program)(equalTo(Option(DynamicValue(2))))
        }
      ),
      suite("die")(
        test("literal") {
          val program = Remote.die("Error")
          assertZIO(program.toZIO.exit)(fails(equalTo(UnsafeEvaluator.Error.Died("Error"))))
        },
        test("remote") {
          val program = Remote.die(Remote("Error"))
          assertZIO(program.toZIO.exit)(fails(equalTo(UnsafeEvaluator.Error.Died("Error"))))
        }
      )
    )
}
