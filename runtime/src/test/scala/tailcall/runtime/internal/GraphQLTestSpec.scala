package tailcall.runtime.internal

import better.files.File
import tailcall.runtime.internal.GraphQLTestSpec.{GraphQLExecutionSpec, GraphQLSDLSpec, GraphQLValidationSpec}
import zio.ZIO
import zio.test.Gen

import java.io.{File => JFile}
import scala.util.Properties

trait GraphQLTestSpec {
  def graphQLSpecGen[A](dir: String)(implicit specCreator: SpecCreator[A]): Gen[Any, A] =
    Gen.fromZIO(load[A](dir, specCreator).map(Gen.fromIterable(_))).flatten

  def removeCommentPrefix(input: String): String = {
    input.split(Properties.lineSeparator).map(_.replace("# ", "")).mkString(Properties.lineSeparator)
  }

  def loadTests[A](dir: String)(implicit specCreator: SpecCreator[A]): ZIO[Any, Nothing, List[A]] =
    load[A](dir, specCreator)

  private def load[A](dir: String, specCreator: SpecCreator[A]): ZIO[Any, Nothing, List[A]] = {
    for {
      files       <- ZIO
        .succeedBlocking(File(getClass.getResource(dir + JFile.separator + specCreator.dir)).glob("*.graphql").toList)
      contentList <- ZIO.foreach(files)(file => ZIO.succeedBlocking(file -> file.contentAsString))
    } yield contentList.map { case (file, content) => specCreator.construct((file, content.split("#>").map(_.trim))) }
  }
}

object GraphQLTestSpec {
  final case class GraphQLSDLSpec(name: String, serverSDL: String, clientSDL: String)
  final case class GraphQLValidationSpec(name: String, serverSDL: String, validationMessage: String)
  final case class GraphQLExecutionSpec(name: String, serverSDL: String, query: String)
}

trait SpecCreator[A] {
  def construct(fileComponents: (File, Array[String])): A
  def dir = ""

  def extractComponent(file: File, components: List[String], token: String): String = {
    components.find(_.contains(token)).map(_.replace(token, "")).map(_.trim)
      .getOrElse(throw new Exception(s"${token} not found: ${file.path}")).trim
  }
}

object SpecCreator {
  def apply[A](implicit instance: SpecCreator[A]): SpecCreator[A] = instance

  implicit val sdlSpecCreator: SpecCreator[GraphQLSDLSpec] = new SpecCreator[GraphQLSDLSpec] {
    override def dir                                     = "sdl"
    def construct(fileComponents: (File, Array[String])) = {
      fileComponents match {
        case (file, components) => GraphQLSDLSpec(
            file.name,
            extractComponent(file, components.toList, "server-sdl"),
            extractComponent(file, components.toList, "client-sdl"),
          )
      }
    }
  }

  implicit val validationSpecCreator: SpecCreator[GraphQLValidationSpec] = new SpecCreator[GraphQLValidationSpec] {
    override def dir                                     = "validation"
    def construct(fileComponents: (File, Array[String])) = {
      fileComponents match {
        case (file, components) => GraphQLValidationSpec(
            file.name,
            extractComponent(file, components.toList, "server-sdl"),
            extractComponent(file, components.toList, "validation-messages"),
          )
      }
    }
  }

  implicit val executionSpecCreator: SpecCreator[GraphQLExecutionSpec] = new SpecCreator[GraphQLExecutionSpec] {
    override def dir                                     = "execution"
    def construct(fileComponents: (File, Array[String])) = {
      fileComponents match {
        case (file, components) => GraphQLExecutionSpec(
            file.name,
            extractComponent(file, components.toList, "server-sdl"),
            extractComponent(file, components.toList, "client-query"),
          )
      }
    }

  }

}
