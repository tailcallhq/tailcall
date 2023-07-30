package tailcall.cli

import tailcall.cli.service.CommandExecutor
import zio.Console.print
import zio.cli.ValidationError
import zio.{ExitCode, Scope, ZIO, ZIOAppArgs, ZIOAppDefault}

object Main extends ZIOAppDefault {
  self =>
  override def run: ZIO[Any with ZIOAppArgs with Scope, Any, Any] =
    ZIOAppArgs.getArgs.flatMap { args =>
      CommandDoc.app.run(args.toList).catchSome { case ValidationError(_, _) =>
        for {
          _ <- print(CommandDoc.command.helpDoc.toPlaintext())
          _ <- exit(ExitCode.failure)
        } yield ()
      }.provide(CommandExecutor.default)
    }
}
