package tailcall.cli

import tailcall.cli.CommandADT.BlueprintOptions
import tailcall.runtime.ast.Digest
import zio.cli._
import zio.http.URL

import java.nio.file.Path

object CustomOptions {
  def digestOption(name: String): Options[Digest] =
    Options.text(name).mapOrFail { digest =>
      if ("^[a-fA-F0-9]{64}$".r.matches(digest)) Right(Digest.fromHex(digest))
      else Left(ValidationError(ValidationErrorType.InvalidArgument, HelpDoc.p("Digest must be a SHA-256 hash.")))
    }

  def integerOption(name: String): Options[Int] =
    Options.text(name).mapOrFail { int =>
      if ("^[0-9]+$".r.matches(int)) Right(int.toInt)
      else Left(ValidationError(ValidationErrorType.InvalidArgument, HelpDoc.p("Integer must be a positive number.")))
    }

  def urlOption(name: String): Options[URL] =
    Options.text(name).mapOrFail { string =>
      URL.fromString(string) match {
        case Left(_) => Left(ValidationError(ValidationErrorType.InvalidArgument, HelpDoc.p(s"Invalid URL: ${string}")))
        case Right(value) => Right(value)
      }
    }

  val remoteOption: Options[URL] = CustomOptions.urlOption("remote").alias("r")

  val digestOption: Options[Digest] = CustomOptions.digestOption("digest")

  val configFileOption: Options[Path] = Options.file("config").alias("c")

  val blueprintOptions = (Options.boolean("blueprint").withDefault(false) ++ Options.boolean("endpoints")
    .withDefault(false) ++ Options.boolean("schema").alias("s").withDefault(false)).map(BlueprintOptions.tupled)

}
