package tailcall.runtime.service

import org.graalvm.polyglot._
import zio.json.{DecoderOps, EncoderOps, JsonCodec}
import zio.{Duration, Task, ZIO, ZLayer}

trait JSExecutor {
  def execute(script: CharSequence, input: CharSequence): Task[CharSequence]
  final def execute[A: JsonCodec](script: CharSequence, input: A): Task[A] =
    execute(script, input.toJson)
      .flatMap(json => ZIO.fromEither(json.fromJson[A]).mapError(error => new RuntimeException(error)))
}

object JSExecutor {
  def execute[A: JsonCodec](script: CharSequence, input: A): ZIO[JSExecutor, Throwable, A] =
    ZIO.serviceWithZIO[JSExecutor](_.execute(script, input))

  def live(timeout: Duration): ZLayer[Any, Nothing, JSExecutor] = ZLayer.succeed(new Live(timeout))

  final class Live(timeout: Duration) extends JSExecutor {

    override def execute(script: CharSequence, input: CharSequence): Task[CharSequence] = {
      ZIO.scoped {
        for {
          ctx    <- ZIO.succeed(Context.create())
          f      <- runScript(script, input, ctx).fork
          _      <- interrupt(ctx).fork
          result <- f.join
        } yield result
      }
    }

    private def interrupt(ctx: Context): Task[Unit] = {
      ZIO.attemptBlocking {
        Thread.sleep(timeout.toMillis)
        ctx.interrupt(timeout)
      }
    }

    private def runScript(script: CharSequence, input: CharSequence, ctx: Context): Task[CharSequence] = {
      ZIO.attemptBlocking {
        val source = s"""(function (input) {return JSON.stringify(($script)(JSON.parse(input))); })"""
        ctx.eval("js", source).execute(input).asString()
      }
    }
  }
}
