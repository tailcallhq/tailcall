package tailcall.registry

import org.flywaydb.core.Flyway
import zio.cli.{CliApp, Command}
import zio.{Scope, ZIO, ZIOAppArgs, ZIOAppDefault}

object DBMigration extends ZIOAppDefault {
  val command: Command[DBCommand] = Command("db")
    .subcommands(Command("migrate").as(Migrate), Command("repair").as(Repair), Command("clean").as(Clean))

  override def run: ZIO[Any with ZIOAppArgs with Scope, Any, Any] = {
    for {
      args       <- ZIOAppArgs.getArgs
      dataSource <- MySQLRegistry.dataSource("localhost", 3306, Option("root"), Option("password"))
      flyway     <- ZIO.attemptBlocking(Flyway.configure().dataSource(dataSource).cleanDisabled(false).load())
      _ <- CliApp.make("tailcall", "0.0.1", command.helpDoc.getSpan, command)(_.execute(flyway)).run(args.toList)
    } yield ()
  }

  sealed trait DBCommand {
    def execute(flyway: Flyway): ZIO[Any, Throwable, Unit]
  }

  case object Migrate extends DBCommand {
    override def execute(flyway: Flyway): ZIO[Any, Throwable, Unit] = ZIO.attempt(flyway.migrate())
  }

  case object Repair extends DBCommand {
    override def execute(flyway: Flyway): ZIO[Any, Throwable, Unit] = ZIO.attempt(flyway.repair())
  }

  case object Clean extends DBCommand {
    override def execute(flyway: Flyway): ZIO[Any, Throwable, Unit] = ZIO.attempt(flyway.clean())
  }
}
