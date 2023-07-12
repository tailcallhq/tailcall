package tailcall.runtime.internal

import better.files.File

import java.io.{File => JFile}
import scala.util.Properties

trait GraphQLTestSpec {

  def removeCommentPrefix(input: String): String = {
    input.split(Properties.lineSeparator).map(_.replace("# ", "")).mkString(Properties.lineSeparator)
  }

  private def load(dir: String): List[File] = { File(getClass.getResource(dir)).glob("*.graphql").toList }

  def loadTests(dir: String): List[(File, List[GraphQLSpec])] = {
    val sdlSpecFiles        = load(dir + JFile.separator + "sdl")
    val validationSpecFiles = load(dir + JFile.separator + "validation")
    val executionSpecFiles  = load(dir + JFile.separator + "execution")
    val files               = sdlSpecFiles ++ validationSpecFiles ++ executionSpecFiles
    files.map(file => {
      val components         = file.contentAsString.split("#>").map(_.trim).toList
      val serverSDL          = extractComponent(components, "server-sdl")
      val clientSDL          = extractComponent(components, "client-sdl")
      val validationMessages = extractComponent(components, "validation-messages")
      val query              = extractComponent(components, "client-query")

      validateSpecFileContents(serverSDL, clientSDL, validationMessages, file)

      var specs: List[GraphQLSpec] = List.empty
      specs = GraphQLConfig2DocumentSpec(serverSDL) :: specs
      if (clientSDL.nonEmpty) specs = GraphQLConfig2ClientSDLSpec(serverSDL, clientSDL) :: specs
      if (validationMessages.nonEmpty) specs = GraphQLValidationSpec(serverSDL, validationMessages) :: specs
      if (query.nonEmpty) specs = GraphQLExecutionSpec(serverSDL, query) :: specs
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
