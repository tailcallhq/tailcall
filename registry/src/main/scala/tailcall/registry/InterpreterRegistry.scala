package tailcall.registry

import zio.{Task, Ref, ZIO}
import caliban.GraphQLInterpreter
import tailcall.runtime.service.HttpContext
import tailcall.runtime.service.GraphQLGenerator
import caliban.CalibanError
import zio.ZLayer

trait InterpreterRegistry {
  def get(id: String): Task[Option[InterpreterRegistry.Interpreter]]
}

object InterpreterRegistry {
  type Interpreter = GraphQLInterpreter[HttpContext, CalibanError]

  final class Live(reg: SchemaRegistry, gql: GraphQLGenerator, cache: Ref[Map[String, Interpreter]])
      extends InterpreterRegistry {
    def get(hex: String): Task[Option[Interpreter]] =
      for {
        option <- cache.get.map(_.get(hex))
        _      <- option match {
          case None => for {
              option <- reg.get(hex)
              option <- option match {
                case None     => ZIO.succeed(None)
                case Some(bp) => for {
                    int <- gql.toGraphQL(bp).interpreter
                    _   <- cache.update(_ + (hex -> int))
                  } yield int
              }
            } yield option
          case int  => ZIO.succeed(Some(int))
        }
      } yield ???
  }

  def live: ZLayer[SchemaRegistry with GraphQLGenerator, Nothing, Live] =
    ZLayer.fromZIO(for {
      reg   <- ZIO.service[SchemaRegistry]
      gql   <- ZIO.service[GraphQLGenerator]
      cache <- Ref.make(Map.empty[String, Interpreter])
    } yield new Live(reg, gql, cache))
}
