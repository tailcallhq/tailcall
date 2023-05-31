package tailcall.cli

import tailcall.cli.CommandADT.{BlueprintOptions, SourceFormat, TargetFormat}
import tailcall.runtime.model.ConfigFormat
import zio.cli._
import zio.http.URL

object CustomOptions {

  val remoteOption: Options[URL] = CustomOptions.urlOption("remote").alias("r")

  val blueprintOptions: Options[BlueprintOptions] = {
    Options.boolean("blueprint").withDefault(false) ++
      Options.boolean("endpoints").withDefault(false) ++
      Options.boolean("schema").alias("s").withDefault(false)
  }.map(BlueprintOptions(_, _, _))

  val sourceFormat: Options[SourceFormat] = Options
    .enumeration("source")(SourceFormat.Postman.named, SourceFormat.SchemaDefinitionLanguage.named)

  val targetFormat: Options[TargetFormat] = Options.enumeration("target")(
    TargetFormat.Config(ConfigFormat.JSON).named,
    TargetFormat.Config(ConfigFormat.YML).named,
    TargetFormat.Config(ConfigFormat.GRAPHQL).named,
    TargetFormat.JsonLines.named,
  )

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
}
