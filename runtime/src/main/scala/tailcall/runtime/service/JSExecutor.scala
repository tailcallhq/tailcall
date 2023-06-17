package tailcall.runtime.service

import org.graalvm.polyglot._
import zio.json.{DecoderOps, EncoderOps, JsonCodec}
import zio.{Scope, Task, ZIO, ZLayer}

trait JSExecutor {
  def execute(script: String, input: String): Task[String]
  final def execute[A: JsonCodec](script: String, input: A): Task[A] =
    execute(script, input.toJson)
      .flatMap(json => ZIO.fromEither(json.fromJson[A]).mapError(error => new RuntimeException(error)))
}

object JSExecutor {
  def execute[A: JsonCodec](script: String, input: A): ZIO[JSExecutor, Throwable, A] =
    ZIO.serviceWithZIO[JSExecutor](_.execute(script, input))

  def live: ZLayer[Scope, Nothing, JSExecutor] =
    ZLayer {
      for {
        ctx <- ZIO.succeed(Context.create())
        _   <- ZIO.addFinalizer(ZIO.attemptBlocking(ctx.close(true)).orDie)
      } yield Live(ctx)
    }

  final case class Live(ctx: Context) extends JSExecutor {
    override def execute(script: String, input: String): Task[String] = {
      ZIO.attemptBlocking {
        val source = s"""(function (input) {return JSON.stringify(($script)(JSON.parse(input))); })"""
        ctx.eval("js", source).execute(input).asString()
      }
    }
  }
}
