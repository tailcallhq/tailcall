package tailcall.runtime.internal

import better.files.File
import tailcall.runtime.internal.GraphQLTestSpec.GraphQLSpec
import tailcall.runtime.model.{Config, ConfigFormat}
import zio.test.Gen
import zio.{IO, UIO, ZIO}

trait GraphQLTestSpec {
  def getFiles(dir: String): UIO[List[File]] =
    ZIO.succeedBlocking(File(getClass.getResource(dir)).glob("*.graphql").toList)

  private def extractComponent(file: File, components: List[String], token: String): String = {
    components.find(_.contains(token)).map(_.replace(token, "")).map(_.trim)
      .getOrElse(throw new Exception(s"${token} not found: ${file.path}")).trim
  }

  def loadSpecs(dir: String): ZIO[Any, Nothing, List[GraphQLSpec]] =
    for {
      files       <- getFiles(dir)
      contentList <- ZIO.foreach(files)(file => ZIO.succeedBlocking(file -> file.contentAsString))
    } yield contentList.map { case (file, content) =>
      val components = content.split("#>").map(_.trim)
      GraphQLSpec(
        extractComponent(file, components.toList, "server-sdl"),
        extractComponent(file, components.toList, "client-sdl"),
      )
    }

  def graphQLSpecGen(dir: String): Gen[Any, GraphQLSpec] = Gen.fromZIO(loadSpecs(dir).map(Gen.fromIterable(_))).flatten
}

object GraphQLTestSpec {
  final case class GraphQLSpec(serverSDL: String, clientSDL: String) {
    def config: IO[String, Config] = ConfigFormat.GRAPHQL.decode(serverSDL)
  }
}
