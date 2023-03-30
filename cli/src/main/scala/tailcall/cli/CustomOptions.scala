package tailcall.cli

import tailcall.cli.CommandADT.{BlueprintOptions, SourceFormat}
import tailcall.runtime.service.DSLFormat
import zio.cli._
import zio.http.URL

object CustomOptions {

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

  val blueprintOptions = (Options.boolean("blueprint").withDefault(false) ++ Options.boolean("endpoints")
    .withDefault(false) ++ Options.boolean("schema").alias("s").withDefault(false)).map(BlueprintOptions.tupled)

  val sourceFormat: Options[SourceFormat] = Options.enumeration("source")("postman" -> SourceFormat.POSTMAN).alias("s")

  val configFormat: Options[DSLFormat] = Options
    .enumeration("format")("json" -> DSLFormat.JSON, "yaml" -> DSLFormat.YML, "graphql" -> DSLFormat.GRAPHQL)
    .withDefault(DSLFormat.GRAPHQL)
}
