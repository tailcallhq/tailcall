package tailcall.cli

import tailcall.cli.service.CommandExecutor
import zio.{ExitCode, Scope, ZIO, ZIOAppArgs, ZIOAppDefault}

object Main extends ZIOAppDefault {
  self =>
  override def run: ZIO[Any with ZIOAppArgs with Scope, Any, Any] =
    ZIOAppArgs.getArgs
      .flatMap(args => (CommandSpec.app.run(args.toList) <> self.exit(ExitCode.failure)).provide(CommandExecutor.live))
}
