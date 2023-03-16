package tailcall.cli

import zio.{Scope, ZIO, ZIOAppArgs, ZIOAppDefault}

object Main extends ZIOAppDefault {
  override def run: ZIO[Any with ZIOAppArgs with Scope, Any, Any] =
    ZIOAppArgs.getArgs.flatMap(args => CommandSpec.app.run(args.toList).provide(CommandExecutor.live))
}
