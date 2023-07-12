package tailcall.runtime.internal

import better.files.File
import zio.ZIO

import java.io.{File => JFile}
import scala.util.Properties

trait GraphQLTestSpec {

  def removeCommentPrefix(input: String): String = {
    input.split(Properties.lineSeparator).map(_.replace("# ", "")).mkString(Properties.lineSeparator)
  }

  private def load(dir: String): ZIO[Any, Nothing, List[File]] = {
    for { files <- ZIO.succeedBlocking(File(getClass.getResource(dir)).glob("*.graphql").toList) } yield files
  }

  def loadTests(dir: String): ZIO[Any, Nothing, List[(File, List[GraphQLSpec])]] = {
    for {
      sdlSpecFiles        <- load(dir + JFile.separator + "sdl")
      validationSpecFiles <- load(dir + JFile.separator + "validation")
      executionSpecFiles  <- load(dir + JFile.separator + "execution")
      files               <- ZIO.succeed(sdlSpecFiles ++ validationSpecFiles ++ executionSpecFiles)
    } yield files.map(file => {
      val components         = file.contentAsString.split("#>").map(_.trim).toList
      val serverSDL          = extractComponent(components, "server-sdl")
      val clientSDL          = extractComponent(components, "client-sdl")
      val validationMessages = extractComponent(components, "validation-messages")
      val query              = extractComponent(components, "client-query")

      validateSpecFileContents(serverSDL, clientSDL, validationMessages, file)

      val specs = (List(GraphQLConfig2DocumentSpec(serverSDL)) :+
        (if (clientSDL.nonEmpty) GraphQLConfig2ClientSDLSpec(serverSDL, clientSDL) else None) :+
        (if (validationMessages.nonEmpty) GraphQLValidationSpec(serverSDL, validationMessages) else None) :+
        (if (query.nonEmpty) GraphQLExecutionSpec(serverSDL, query) else None)).collect { case spec: GraphQLSpec =>
        spec
      }
      (file, specs)
    })
  }

  def extractComponent(components: List[String], token: String): String = {
    components.find(_.contains(token)).map(_.replace(token, "")).map(_.trim).getOrElse("").trim
  }

  def validateSpecFileContents(serverSDL: String, clientSDL: String, validationMessages: String, file: File) = {
    if (serverSDL.isBlank()) throw new Exception(s"server-sdl not found: ${file.path}")
    if (!clientSDL.isBlank() && !validationMessages.isBlank())
      throw new Exception(s"Only one of client-SDL or validation-messages must be present: ${file.path}")
  }

}

sealed trait GraphQLSpec
case class GraphQLConfig2DocumentSpec(serverSDL: String)                        extends GraphQLSpec
case class GraphQLConfig2ClientSDLSpec(serverSDL: String, clientSDL: String)    extends GraphQLSpec
case class GraphQLValidationSpec(serverSDL: String, validationMessages: String) extends GraphQLSpec
case class GraphQLExecutionSpec(serverSDL: String, query: String)               extends GraphQLSpec
