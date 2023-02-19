package tailcall.gateway

import tailcall.gateway.RemoteSpec.seqSchema
import tailcall.gateway.remote.Remote.ComposeStringInterpolator
import tailcall.gateway.remote.{Remote, RemoteSchemaInferer}
import zio.schema.Schema
import zio.test.{ZIOSpecDefault, assertTrue}

object RemoteSchemaInfererSpec extends ZIOSpecDefault {
  def spec =
    suite("RemoteSchemaInfer")(
      suite("literal")(
        test("int") {
          val program = RemoteSchemaInferer.inferSchema(Remote(1))
          assertTrue(program == Schema[Int])
        },
        test("string") {
          val program = RemoteSchemaInferer.inferSchema(Remote("hello"))
          assertTrue(program == Schema[String])
        },
        test("boolean") {
          val program = RemoteSchemaInferer.inferSchema(Remote(true))
          assertTrue(program == Schema[Boolean])
        }
      ),
      suite("math")(
        test("add") {
          val program = RemoteSchemaInferer.inferSchema(Remote(1) + Remote(2))
          assertTrue(program == Schema[Int])
        },
        test("multiply") {
          val program = RemoteSchemaInferer.inferSchema(Remote(1) * Remote(2))
          assertTrue(program == Schema[Int])
        },
        test("divide") {
          val program = RemoteSchemaInferer.inferSchema(Remote(1) / Remote(2))
          assertTrue(program == Schema[Int])
        },
        test("modulo") {
          val program = RemoteSchemaInferer.inferSchema(Remote(1) % Remote(2))
          assertTrue(program == Schema[Int])
        },
        test("greaterThan") {
          val program = RemoteSchemaInferer.inferSchema(Remote(1) > Remote(2))
          assertTrue(program == Schema[Boolean])
        },
        test("negate") {
          val program = RemoteSchemaInferer.inferSchema(Remote(1).negate)
          assertTrue(program == Schema[Int])
        }
      ),
      suite("logical")(
        test("and") {
          val program =
            RemoteSchemaInferer.inferSchema(Remote(true) && Remote(false))
          assertTrue(program == Schema[Boolean])
        },
        test("or") {
          val program =
            RemoteSchemaInferer.inferSchema(Remote(true) || Remote(false))
          assertTrue(program == Schema[Boolean])
        },
        test("not") {
          val program = RemoteSchemaInferer.inferSchema(!Remote(true))
          assertTrue(program == Schema[Boolean])
        }
      ),
      suite("string")(test("concat") {
        val program =
          RemoteSchemaInferer.inferSchema(Remote("hello") ++ Remote("world"))
        assertTrue(program == Schema[String])
      }),
      suite("tuple")(
        test("get Index") {
          val program = RemoteSchemaInferer
            .inferSchema(Remote.fromTuple((Remote(1), Remote("hello")))._1)
          assertTrue(program == Schema[Int])
        },
        test("tuple 2") {
          val program = RemoteSchemaInferer
            .inferSchema(Remote.fromTuple((Remote(1), Remote("hello"))))
          assertTrue(program == Schema[(Int, String)])
        }
      ),
      suite("sequence")(
        test("fromSeq") {
          val program = RemoteSchemaInferer
            .inferSchema(Remote.fromSeq(Seq(Remote(1), Remote(2))))
          assertTrue(program.ast == Schema[Seq[Int]].ast)
        },
        test("concat") {
          val program = RemoteSchemaInferer.inferSchema(
            Remote.fromSeq(Seq(Remote(1), Remote(2))) ++ Remote
              .fromSeq(Seq(Remote(3), Remote(4)))
          )
          assertTrue(program.ast == Schema[Seq[Int]].ast)
        },
        test("reverse") {
          val program = RemoteSchemaInferer
            .inferSchema(Remote.fromSeq(Seq(Remote(1), Remote(2))).reverse)
          assertTrue(program.ast == Schema[Seq[Int]].ast)
        },
        test("map") {
          val program = RemoteSchemaInferer
            .inferSchema(Remote(Seq(1, 2, 3)).map(_ + Remote(1)))
          assertTrue(program.ast == seqSchema[Int].ast)
        },
        test("groupBy") {
          val program = RemoteSchemaInferer
            .inferSchema(Remote(Seq(1, 2, 3)).groupBy(_ % Remote(2)))
          assertTrue(program.ast == Schema[Map[Int, Seq[Int]]].ast)
        }
      ),
      suite("either")(
        test("right") {
          val program =
            RemoteSchemaInferer.inferSchema(Remote.fromEither(Right(Remote(1))))
          assertTrue(program.ast == Schema[Either[Unit, Int]].ast)
        },
        test("left") {
          val program =
            RemoteSchemaInferer.inferSchema(Remote.fromEither(Left(Remote(1))))
          assertTrue(program.ast == Schema[Either[Int, Unit]].ast)
        },
        test("fold right") {
          val program = RemoteSchemaInferer.inferSchema(
            Remote
              .fromEither(Right(Remote(1)))
              .fold((l: Remote[Nothing]) => l.length, r => r * Remote(2))
          )
          assertTrue(program.ast == Schema[Int].ast)
        },
        test("fold left") {
          val program = RemoteSchemaInferer.inferSchema(
            Remote
              .fromEither(Left(Remote("Error")))
              .fold(l => rs"Some ${l}", (r: Remote[Nothing]) => r * Remote(2))
          )
          assertTrue(program.ast == Schema[String].ast)
        }
      )
    )
}
