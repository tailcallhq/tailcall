package tailcall.runtime.internal

import better.files.File
import tailcall.runtime.internal.GraphQLTestSpec.GraphQLSpec
import zio.ZIO
import zio.test.Gen

trait GraphQLTestSpec {
  def graphQLSpecGen(dir: String): Gen[Any, GraphQLSpec] = Gen.fromZIO(load(dir).map(Gen.fromIterable(_))).flatten

  private def extractComponent(file: File, components: List[String], token: String): String = {
    components.find(_.contains(token)).map(_.replace(token, "")).map(_.trim)
      .getOrElse(throw new Exception(s"${token} not found: ${file.path}")).trim
  }

  private def load(dir: String): ZIO[Any, Nothing, List[GraphQLSpec]] =
    for {
      files       <- ZIO.succeedBlocking(File(getClass.getResource(dir)).glob("*.graphql").toList.filter(file =>
        file.name.contains("test-query")
      ))
      contentList <- ZIO.foreach(files)(file => ZIO.succeedBlocking(file -> file.contentAsString))
    } yield contentList.map { case (file, content) =>
      val components = content.split("#>").map(_.trim)
      GraphQLSpec(
        extractComponent(file, components.toList, "server-sdl"),
        extractComponent(file, components.toList, "client-sdl"),
        extractComponent(file, components.toList, "client-query"),
      )
    }
}

object GraphQLTestSpec {
  final case class GraphQLSpec(serverSDL: String, clientSDL: String, clientQuery: String)
}
