package tailcall.runtime.internal

import better.files.File
import caliban.parsing.Parser
import caliban.parsing.adt.Definition.ExecutableDefinition.OperationDefinition
import tailcall.runtime.DirectiveCodec.DecoderSyntax
import tailcall.runtime.JsonT
import tailcall.runtime.internal.GraphQLSpec._
import tailcall.runtime.model.ExpectType
import zio.json._
import zio.json.yaml._
import zio.{NonEmptyChunk, ZIO}

import java.io.{File => JFile}
import scala.util.Properties

trait GraphQLTestSpec {

  def removeCommentPrefix(input: String): String = {
    input.split(Properties.lineSeparator).map(_.replace("# ", "")).mkString(Properties.lineSeparator)
  }

  private def load(dir: String): ZIO[Any, Nothing, List[File]] = {
    for { files <- ZIO.succeedBlocking(File(getClass.getResource(dir)).glob("*.graphql").toList) } yield files
  }

  def loadTests(dir: String): ZIO[Any, Nothing, List[(File, ZIO[Any, Any, GraphQLSpec])]] = {
    for {
      sdlSpecFiles        <- load(dir + JFile.separator + "sdl")
      validationSpecFiles <- load(dir + JFile.separator + "validation")
      executionSpecFiles  <- load(dir + JFile.separator + "execution")
      files               <- ZIO.succeed(sdlSpecFiles ++ validationSpecFiles ++ executionSpecFiles)
    } yield files.map(file => (file, GraphQLSpec.fromFileContent(file)))
  }

}

case class GraphQLSpec(serverSDL: ValidSDL, client: Option[Client])

object GraphQLSpec {

  case class ValidSDL private (sdl: String)

  object ValidSDL {
    def fromSDL(sdl: String, fileName: String) = {
      Parser.parseQuery(sdl).map(_ => new ValidSDL(sdl))
        .fold(error => throw new Exception(s"${error.toString()}: ${fileName}"), value => value);
    }
  }

  case class ValidQuery private (query: String, expectedOutput: JsonT.Constant)

  object ValidQuery {
    def fromQuery(query: String, fileName: String) = {
      for {
        document   <- Parser.parseQuery(query)
          .fold(error => throw new Exception(s"${error.toString()}: ${fileName}"), value => value);
        definition <- ZIO.fromOption(document.definitions.collectFirst { case op: OperationDefinition => op })
        expect     <- ZIO.fromOption {
          val directiveOption = definition.directives.flatMap(_.fromDirective[ExpectType].toOption).headOption
          if (directiveOption.isEmpty) { throw new Exception(s"@expect directive missing on query: ${fileName}") }
          directiveOption
        }
      } yield new ValidQuery(query, expect.output)
    }
  }

  sealed trait Client {}
  object Client       {
    case class SDL(clientSDL: ValidSDL, queries: List[ValidQuery]) extends Client

    case class ValidationError(error: NonEmptyChunk[TValid.Cause[String]]) extends Client

    object ValidationError {
      def removeCommentPrefix(input: String): String = {
        input.split(Properties.lineSeparator).map(_.replace("# ", "")).mkString(Properties.lineSeparator)
      }

      def fromValidationMessages(validationMessages: String) = {
        val yaml = removeCommentPrefix(validationMessages)
        yaml.fromYaml[ValidationError]
      }
    }

    implicit val causeCodec: JsonCodec[TValid.Cause[String]] = DeriveJsonCodec.gen[TValid.Cause[String]]
    implicit val jsonCodec: JsonCodec[ValidationError]       = DeriveJsonCodec.gen[ValidationError]

  }

  def extractComponent(components: List[String], token: String): String = {
    components.find(_.contains(token)).map(_.replace(token, "")).map(_.trim).getOrElse("").trim
  }

  def extractSpecComponents(file: File): (String, String, String, String) = {
    val components         = file.contentAsString.split("#>").map(_.trim).toList
    val serverSDLStr       = extractComponent(components, "server-sdl")
    val clientSDLStr       = extractComponent(components, "client-sdl")
    val validationMessages = extractComponent(components, "validation-messages")
    val queryString        = extractComponent(components, "client-query")
    (serverSDLStr, clientSDLStr, validationMessages, queryString)
  }

  def validateSpecFileContents(
    serverSDLStr: String,
    clientSDLStr: String,
    validationMessages: String,
    queryString: String,
    file: File,
  ) = {
    if (serverSDLStr.isBlank()) throw new Exception(s"server-sdl not found: ${file.path}")
    if (!clientSDLStr.isBlank() && !validationMessages.isBlank())
      throw new Exception(s"Only one of client-sdl or validation-messages must be present: ${file.path}")
    if (!queryString.isBlank() && clientSDLStr.isBlank())
      throw new Exception(s"client-sdl not found but client-query is present: ${file.path}")
  }

  def fromFileContent(file: File) = {

    val (serverSDLStr, clientSDLStr, validationMessages, queryString) = extractSpecComponents(file)

    validateSpecFileContents(serverSDLStr, clientSDLStr, validationMessages, queryString, file)

    val serverSDLZio = ValidSDL.fromSDL(serverSDLStr, file.name)

    val clientSDLZio = ValidSDL.fromSDL(clientSDLStr, file.name)

    val queriesZio =
      if (!queryString.isBlank()) {
        ZIO.collectAll {
          val qs = queryString.split("(?=query)").map(_.trim).toList.map(q => ValidQuery.fromQuery(q, file.name))
          qs
        }
      } else { ZIO.succeed(Nil) }

    val sdlZio = for {
      c  <- clientSDLZio
      qs <- queriesZio
    } yield Client.SDL(c, qs)

    val validationError       = Client.ValidationError.fromValidationMessages(validationMessages)
    val validationErrorOption = validationError match {
      case Left(error)  => if (!validationMessages.isBlank()) throw new Exception(s"${error}: ${file.name}") else None
      case Right(value) => Some(value)
    }

    for {
      serverSDL <- serverSDLZio
      sdlOption <- sdlZio.option
    } yield {
      val clientOption =
        if (!clientSDLStr.isBlank()) sdlOption else if (!validationMessages.isBlank()) validationErrorOption else None
      GraphQLSpec(serverSDL, clientOption)
    }

  }
}
