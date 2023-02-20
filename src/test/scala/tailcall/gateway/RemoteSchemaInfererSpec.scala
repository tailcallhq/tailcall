package tailcall.gateway

import tailcall.gateway.RemoteSpec.seqSchema
import tailcall.gateway.remote._
import zio.schema.Schema
import zio.test.{ZIOSpecDefault, assertTrue}

object RemoteSchemaInfererSpec extends ZIOSpecDefault {
  def spec =
    suite("RemoteSchemaInfer")(
      suite("literal")(
        test("int") {
          val program = RemoteSchemaInferer.inferSchema(Lambda(1))
          assertTrue(program == Schema[Int])
        },
        test("string") {
          val program = RemoteSchemaInferer.inferSchema(Lambda("hello"))
          assertTrue(program == Schema[String])
        },
        test("boolean") {
          val program = RemoteSchemaInferer.inferSchema(Lambda(true))
          assertTrue(program == Schema[Boolean])
        }
      ),
      suite("math")(
        test("add") {
          val program = RemoteSchemaInferer.inferSchema(Lambda(1) + Lambda(2))
          assertTrue(program == Schema[Int])
        },
        test("multiply") {
          val program = RemoteSchemaInferer.inferSchema(Lambda(1) * Lambda(2))
          assertTrue(program == Schema[Int])
        },
        test("divide") {
          val program = RemoteSchemaInferer.inferSchema(Lambda(1) / Lambda(2))
          assertTrue(program == Schema[Int])
        },
        test("modulo") {
          val program = RemoteSchemaInferer.inferSchema(Lambda(1) % Lambda(2))
          assertTrue(program == Schema[Int])
        },
        test("greaterThan") {
          val program = RemoteSchemaInferer.inferSchema(Lambda(1) > Lambda(2))
          assertTrue(program == Schema[Boolean])
        },
        test("negate") {
          val program = RemoteSchemaInferer.inferSchema(Lambda(1).negate)
          assertTrue(program == Schema[Int])
        }
      ),
      suite("logical")(
        test("and") {
          val program =
            RemoteSchemaInferer.inferSchema(Lambda(true) && Lambda(false))
          assertTrue(program == Schema[Boolean])
        },
        test("or") {
          val program =
            RemoteSchemaInferer.inferSchema(Lambda(true) || Lambda(false))
          assertTrue(program == Schema[Boolean])
        },
        test("not") {
          val program = RemoteSchemaInferer.inferSchema(!Lambda(true))
          assertTrue(program == Schema[Boolean])
        }
      ),
      suite("string")(test("concat") {
        val program =
          RemoteSchemaInferer.inferSchema(Lambda("hello") ++ Lambda("world"))
        assertTrue(program == Schema[String])
      }),
      suite("tuple")(
        test("get Index") {
          val program = RemoteSchemaInferer
            .inferSchema(Lambda.fromTuple((Lambda(1), Lambda("hello")))._1)
          assertTrue(program == Schema[Int])
        },
        test("tuple 2") {
          val program = RemoteSchemaInferer
            .inferSchema(Lambda.fromTuple((Lambda(1), Lambda("hello"))))
          assertTrue(program == Schema[(Int, String)])
        }
      ),
      suite("sequence")(
        test("fromSeq") {
          val program = RemoteSchemaInferer
            .inferSchema(Lambda.fromSeq(Seq(Lambda(1), Lambda(2))))
          assertTrue(program.ast == Schema[Seq[Int]].ast)
        },
        test("concat") {
          val program = RemoteSchemaInferer.inferSchema(
            Lambda.fromSeq(Seq(Lambda(1), Lambda(2))) ++ Lambda
              .fromSeq(Seq(Lambda(3), Lambda(4)))
          )
          assertTrue(program.ast == Schema[Seq[Int]].ast)
        },
        test("reverse") {
          val program = RemoteSchemaInferer
            .inferSchema(Lambda.fromSeq(Seq(Lambda(1), Lambda(2))).reverse)
          assertTrue(program.ast == Schema[Seq[Int]].ast)
        },
        test("map") {
          val program = RemoteSchemaInferer
            .inferSchema(Lambda(Seq(1, 2, 3)).map(_ + Lambda(1)))
          assertTrue(program.ast == seqSchema[Int].ast)
        },
        test("groupBy") {
          val program = RemoteSchemaInferer
            .inferSchema(Lambda(Seq(1, 2, 3)).groupBy(_ % Lambda(2)))
          assertTrue(program.ast == Schema[Map[Int, Seq[Int]]].ast)
        }
      ),
      suite("either")(
        test("right") {
          val program =
            RemoteSchemaInferer.inferSchema(Lambda.fromEither(Right(Lambda(1))))
          assertTrue(program.ast == Schema[Either[Unit, Int]].ast)
        },
        test("left") {
          val program =
            RemoteSchemaInferer.inferSchema(Lambda.fromEither(Left(Lambda(1))))
          assertTrue(program.ast == Schema[Either[Int, Unit]].ast)
        },
        test("fold right") {
          val program = RemoteSchemaInferer.inferSchema(
            Lambda
              .fromEither(Right(Lambda(1)))
              .fold((l: Remote[Nothing]) => l.length, r => r * Lambda(2))
          )
          assertTrue(program.ast == Schema[Int].ast)
        },
        test("fold left") {
          val program = RemoteSchemaInferer.inferSchema(
            Lambda
              .fromEither(Left(Lambda("Error")))
              .fold(l => rs"Some ${l}", (r: Remote[Nothing]) => r * Lambda(2))
          )
          assertTrue(program.ast == Schema[String].ast)
        }
      )
    )
}
