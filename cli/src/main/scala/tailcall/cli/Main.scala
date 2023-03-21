package tailcall.cli

import tailcall.cli.service.CommandExecutor
import zio.{Scope, ZIO, ZIOAppArgs, ZIOAppDefault}

object Main extends ZIOAppDefault {
  self =>
  override def run: ZIO[Any with ZIOAppArgs with Scope, Any, Any] =
    ZIOAppArgs.getArgs.flatMap(args => CommandDoc.app.run(args.toList).provide(CommandExecutor.default))
}
